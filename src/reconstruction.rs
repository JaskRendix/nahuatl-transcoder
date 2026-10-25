//! Reconstruction of morphophonemic candidates from SEP-style
//! phonetic spellings.
//!
//! Because phonetic orthographies collapse distinctions present
//! in morphophonemic systems, reconstruction is inherently
//! ambiguous. This module generates and ranks candidate
//! morphophonemic forms by applying a collection of reverse
//! transformation heuristics.

use crate::atomic::{is_atomic_consonant, is_atomic_vowel, to_atomic};
use crate::geminate::geminate;
use crate::orthography::atomic_to_ack;
use crate::phonetics::degeminate;
use crate::{apply_f2m, apply_m2f};

/// Records the transformation path used to generate a candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Derivation {
    Unchanged,
    FToM,
    FinalNRecovery,
    FinalWRecovery,
    CodaWRecovery,
    KLenitionRecovery,
    KwRecovery,
    LlHlRecovery,
    Gemination,
    FToMThenGemination,
    GeminationThenFToM,
}

/// A reconstructed morphophonemic candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// Candidate form in ATOMIC orthography.
    pub atomic: String,

    /// Candidate rendered in ACK orthography.
    pub ack: String,

    /// Transformations that produced this candidate.
    pub derivations: Vec<Derivation>,
}

impl Candidate {
    /// Assigns a heuristic confidence score.
    ///
    /// Higher scores represent analyses that require fewer
    /// reconstruction assumptions.
    pub fn score(&self) -> i32 {
        let mut score = 0;

        if self.derivations.contains(&Derivation::Unchanged) {
            score += 10;
        }

        if self.derivations.contains(&Derivation::FinalNRecovery) {
            score -= 1;
        }

        if self.derivations.contains(&Derivation::FinalWRecovery) {
            score -= 1;
        }

        if self.derivations.contains(&Derivation::KLenitionRecovery) {
            score -= 2;
        }

        if self.derivations.contains(&Derivation::CodaWRecovery) {
            score -= 2;
        }

        score
    }
}

/// Indicates whether reconstruction produced zero, one,
/// or multiple valid candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateStatus {
    NoCandidates,
    Unique,
    Ambiguous,
}

/// Collection of reconstruction candidates together with
/// ambiguity information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateSet {
    /// Reconstructed candidates ordered by score.
    pub candidates: Vec<Candidate>,

    /// Whether reconstruction was unique or ambiguous.
    pub status: CandidateStatus,
}

/// Returns true if a forward projection reproduces the
/// observed form, allowing for gemination differences.
fn projections_match(projected: &str, observed: &str) -> bool {
    projected == observed || degeminate(projected) == degeminate(observed)
}

/// Adds a candidate if it projects back to the observed form.
///
/// Duplicate candidates are merged and their derivation paths
/// are accumulated.
fn add_candidate(
    candidates: &mut Vec<Candidate>,
    atomic: String,
    derivation: Derivation,
    observed: &str,
) {
    if !projections_match(&apply_m2f(&atomic), observed) {
        return;
    }

    if let Some(existing) = candidates
        .iter_mut()
        .find(|candidate| candidate.atomic == atomic)
    {
        if !existing.derivations.contains(&derivation) {
            existing.derivations.push(derivation);
        }
        return;
    }

    candidates.push(Candidate {
        ack: atomic_to_ack(&atomic),
        atomic,
        derivations: vec![derivation],
    });
}

/// Reconstructs possible morphophonemic analyses from a
/// SEP-style spelling.
///
/// Multiple candidate forms may be returned because distinct
/// morphophonemic words can project to the same phonetic form.
///
/// Candidates are ranked using heuristic confidence scores.
pub fn reconstruct_sep(input: &str) -> CandidateSet {
    let observed = to_atomic(input);
    let mut candidates = Vec::new();

    if !observed.is_empty() {
        add_candidate(
            &mut candidates,
            observed.clone(),
            Derivation::Unchanged,
            &observed,
        );

        let f_to_m = apply_f2m(&observed);
        add_candidate(&mut candidates, f_to_m.clone(), Derivation::FToM, &observed);

        let final_n_recovery = observed.strip_suffix('h').map(|stem| format!("{stem}n"));
        if let Some(recovered) = &final_n_recovery {
            add_candidate(
                &mut candidates,
                recovered.clone(),
                Derivation::FinalNRecovery,
                &observed,
            );
        }

        if let Some(recovered) = &final_n_recovery {
            for coda_after_final_n in recover_coda_w(recovered) {
                add_candidate(
                    &mut candidates,
                    coda_after_final_n,
                    Derivation::CodaWRecovery,
                    &observed,
                );
            }
        }

        if let Some(final_w_recovery) = recover_final_w(&observed) {
            add_candidate(
                &mut candidates,
                final_w_recovery,
                Derivation::FinalWRecovery,
                &observed,
            );
        }

        let mut k_lenition_seeds = vec![observed.as_str()];
        if let Some(recovered) = final_n_recovery.as_deref() {
            k_lenition_seeds.push(recovered);
        }
        let k_lenition_recoveries: Vec<String> = k_lenition_seeds
            .iter()
            .flat_map(|seed| recover_k_lenition(seed))
            .collect();
        for k_recovery in &k_lenition_recoveries {
            add_candidate(
                &mut candidates,
                k_recovery.clone(),
                Derivation::KLenitionRecovery,
                &observed,
            );

            if let Some(final_w) = recover_final_w(k_recovery) {
                add_candidate(
                    &mut candidates,
                    final_w,
                    Derivation::FinalWRecovery,
                    &observed,
                );
            }

            for coda_w in recover_coda_w(k_recovery) {
                add_candidate(
                    &mut candidates,
                    coda_w,
                    Derivation::CodaWRecovery,
                    &observed,
                );
            }

            for ll in recover_ll_from_hl(k_recovery) {
                add_candidate(&mut candidates, ll, Derivation::LlHlRecovery, &observed);
            }
        }

        let mut kw_recovery_seeds = vec![(observed.clone(), false)];
        kw_recovery_seeds.extend(
            k_lenition_recoveries
                .into_iter()
                .map(|recovered| (recovered, true)),
        );
        for (seed, includes_k_lenition) in kw_recovery_seeds {
            for kw_recovery in recover_kw(&seed) {
                add_candidate(
                    &mut candidates,
                    kw_recovery.clone(),
                    Derivation::KwRecovery,
                    &observed,
                );
                if includes_k_lenition {
                    add_candidate(
                        &mut candidates,
                        kw_recovery,
                        Derivation::KLenitionRecovery,
                        &observed,
                    );
                }
            }
        }

        for coda_w_recovery in recover_coda_w(&observed) {
            add_candidate(
                &mut candidates,
                coda_w_recovery,
                Derivation::CodaWRecovery,
                &observed,
            );
        }

        for ll_hl_recovery in recover_ll_from_hl(&observed) {
            add_candidate(
                &mut candidates,
                ll_hl_recovery,
                Derivation::LlHlRecovery,
                &observed,
            );
        }

        let geminated = geminate(&observed);
        add_candidate(
            &mut candidates,
            geminated.clone(),
            Derivation::Gemination,
            &observed,
        );

        add_candidate(
            &mut candidates,
            geminate(&f_to_m),
            Derivation::FToMThenGemination,
            &observed,
        );

        add_candidate(
            &mut candidates,
            apply_f2m(&geminated),
            Derivation::GeminationThenFToM,
            &observed,
        );
    }

    let status = match candidates.len() {
        0 => CandidateStatus::NoCandidates,
        1 => CandidateStatus::Unique,
        _ => CandidateStatus::Ambiguous,
    };

    candidates.sort_by_key(|c| std::cmp::Reverse(c.score()));

    CandidateSet { candidates, status }
}

/// Recovers possible syllable-coda /w/ values realized as [h].
fn recover_coda_w(observed: &str) -> Vec<String> {
    let chars: Vec<char> = observed.chars().collect();

    let positions: Vec<usize> = chars
        .iter()
        .enumerate()
        .filter_map(|(index, &current)| {
            let is_coda_h = current == 'h'
                && index > 0
                && index + 1 < chars.len()
                && is_atomic_vowel(chars[index - 1])
                && is_atomic_consonant(chars[index + 1]);

            if is_coda_h { Some(index) } else { None }
        })
        .collect();

    let mut results = Vec::new();

    // One recovery per position.
    for &pos in &positions {
        let mut candidate = chars.clone();
        candidate[pos] = 'w';
        results.push(candidate.into_iter().collect());
    }

    // Keep existing behaviour too (all positions).
    if positions.len() > 1 {
        let mut candidate = chars.clone();

        for &pos in &positions {
            candidate[pos] = 'w';
        }

        results.push(candidate.into_iter().collect());
    }

    results
}

/// Recovers word-final /w/ values realized as final [h].
fn recover_final_w(observed: &str) -> Option<String> {
    let chars: Vec<char> = observed.chars().collect();

    if chars.len() >= 2 && chars.last() == Some(&'h') && is_atomic_vowel(chars[chars.len() - 2]) {
        let mut recovered = observed.to_string();
        recovered.pop();
        recovered.push('w');

        Some(recovered)
    } else {
        None
    }
}

/// Recovers /k/ in contexts where it lenited to [h].
fn recover_k_lenition(observed: &str) -> Vec<String> {
    let chars: Vec<char> = observed.chars().collect();

    chars
        .iter()
        .enumerate()
        .filter_map(|(index, &current)| {
            let matches_k_lenition_context = current == 'h'
                && index > 0
                && index + 2 < chars.len()
                && is_atomic_vowel(chars[index - 1])
                && is_atomic_consonant(chars[index + 1])
                && is_atomic_vowel(chars[index + 2]);

            if !matches_k_lenition_context {
                return None;
            }

            let mut candidate = chars.clone();
            candidate[index] = 'k';
            Some(candidate.into_iter().collect())
        })
        .collect()
}

/// Recovers coda /kʷ/ from its phonetic realizations.
fn recover_kw(observed: &str) -> Vec<String> {
    let chars: Vec<char> = observed.chars().collect();

    chars
        .iter()
        .enumerate()
        .filter_map(|(index, &current)| {
            let is_coda_context = index > 0
                && is_atomic_vowel(chars[index - 1])
                && ((current == 'h'
                    && index + 1 < chars.len()
                    && (chars[index + 1] == 'k'
                        || (index + 2 < chars.len()
                            && is_atomic_consonant(chars[index + 1])
                            && is_atomic_vowel(chars[index + 2]))))
                    || (current == 'k'
                        && (index + 1 == chars.len() || is_atomic_consonant(chars[index + 1]))));

            if !is_coda_context {
                return None;
            }

            let mut candidate = chars.clone();
            candidate[index] = 'κ';
            Some(candidate.into_iter().collect())
        })
        .collect()
}

/// Recovers /ll/ from [hl] allophonic spellings.
fn recover_ll_from_hl(observed: &str) -> Vec<String> {
    let chars: Vec<char> = observed.chars().collect();

    chars
        .iter()
        .enumerate()
        .filter_map(|(index, &current)| {
            let matches_ll2hl_context = current == 'h'
                && index > 0
                && index + 2 < chars.len()
                && chars[index - 1] == 'i'
                && chars[index + 1] == 'l'
                && matches!(chars[index + 2], 'a' | 'i');

            if !matches_ll2hl_context {
                return None;
            }

            let mut candidate = chars.clone();
            candidate[index] = 'l';
            Some(candidate.into_iter().collect())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate<'a>(set: &'a CandidateSet, atomic: &str) -> &'a Candidate {
        set.candidates
            .iter()
            .find(|candidate| candidate.atomic == atomic)
            .unwrap_or_else(|| panic!("missing candidate {atomic:?}"))
    }

    #[test]
    fn empty_input_has_no_candidates() {
        let result = reconstruct_sep("");

        assert!(result.candidates.is_empty());
        assert_eq!(result.status, CandidateStatus::NoCandidates);
    }

    #[test]
    fn f_to_m_recovery_keeps_compatible_candidates() {
        let result = reconstruct_sep("pehki");

        assert_eq!(result.status, CandidateStatus::Ambiguous);
        assert_eq!(candidate(&result, "pehki").ack, "pehqui");
        assert_eq!(candidate(&result, "pewki").ack, "peuhqui");
        assert!(
            candidate(&result, "pewki")
                .derivations
                .contains(&Derivation::FToM)
        );
    }

    #[test]
    fn f_to_m_recovery_preserves_alternative_analyses() {
        let result = reconstruct_sep("nihneki");

        assert_eq!(result.status, CandidateStatus::Ambiguous);
        assert_eq!(candidate(&result, "nihneki").ack, "nihnequi");
        assert_eq!(candidate(&result, "nikneki").ack, "nicnequi");
        assert!(
            candidate(&result, "nikneki")
                .derivations
                .contains(&Derivation::FToM)
        );
    }

    #[test]
    fn final_n_recovery_preserves_ambiguity() {
        let result = reconstruct_sep("nosij");

        assert_eq!(result.status, CandidateStatus::Ambiguous);
        assert_eq!(candidate(&result, "nosih").ack, "nocih");

        let recovered = candidate(&result, "nosin");
        assert_eq!(recovered.ack, "nocin");
        assert!(recovered.derivations.contains(&Derivation::FinalNRecovery));
        assert_eq!(apply_m2f(&recovered.atomic), "nosih");
    }

    #[test]
    fn final_n_recovery_only_changes_word_final_h() {
        let result = reconstruct_sep("pipiltij");

        assert_eq!(result.status, CandidateStatus::Ambiguous);
        assert!(
            result
                .candidates
                .iter()
                .any(|candidate| candidate.atomic == "pipiltin")
        );
        assert!(
            result
                .candidates
                .iter()
                .any(|candidate| candidate.atomic == "pipiltih")
        );

        let non_final_h = reconstruct_sep("pehki");
        assert!(
            !non_final_h
                .candidates
                .iter()
                .any(|candidate| candidate.atomic == "penki")
        );
    }

    #[test]
    fn coda_w_recovery_preserves_ambiguity() {
        for (input, expected) in [("kuajtli", "κawλi"), ("uajtli", "wawλi")] {
            let result = reconstruct_sep(input);

            assert_eq!(result.status, CandidateStatus::Ambiguous);
            let recovered = candidate(&result, expected);
            assert!(recovered.derivations.contains(&Derivation::CodaWRecovery));
            assert_eq!(apply_m2f(&recovered.atomic), to_atomic(input));
        }
    }

    #[test]
    fn coda_w_recovery_handles_each_matching_position() {
        let result = reconstruct_sep("kuajnauak");
        let recovered = candidate(&result, "κawnawak");

        assert!(recovered.derivations.contains(&Derivation::CodaWRecovery));
        assert_eq!(apply_m2f(&recovered.atomic), to_atomic("kuajnauak"));
    }

    #[test]
    fn coda_w_recovery_does_not_change_non_coda_h() {
        let observed = to_atomic("nahuatl");
        assert!(recover_coda_w(&observed).is_empty());
    }

    #[test]
    fn k_lenition_recovery_generates_mictlan_and_doctor_candidates() {
        for (input, expected) in [("mijtlaj", "mikλan"), ("dojtor", "doktor")] {
            let observed = to_atomic(input);
            let result = reconstruct_sep(input);
            let recovered = candidate(&result, expected);

            assert!(
                recovered
                    .derivations
                    .contains(&Derivation::KLenitionRecovery)
            );
            assert_eq!(apply_m2f(&recovered.atomic), observed);
        }
    }

    #[test]
    fn k_lenition_recovery_is_limited_to_forward_rule_context() {
        assert!(recover_k_lenition("mihλa").contains(&"mikλa".to_string()));
        assert!(recover_k_lenition("maha").is_empty());
        assert!(recover_k_lenition("mahλ").is_empty());
        assert!(recover_k_lenition("maht").is_empty());
    }

    #[test]
    fn kw_recovery_handles_tecuhtli_and_mictlantecuhtli() {
        for (input, expected) in [("tejtli", "teκλi"), ("mijtlantektli", "mikλanteκλi")] {
            let observed = to_atomic(input);
            let result = reconstruct_sep(input);
            let recovered = candidate(&result, expected);

            assert!(recovered.derivations.contains(&Derivation::KwRecovery));
            if expected == "mikλanteκλi" {
                assert!(
                    recovered
                        .derivations
                        .contains(&Derivation::KLenitionRecovery)
                );
            }
            assert_eq!(apply_m2f(&recovered.atomic), observed);
        }
    }

    #[test]
    fn kw_recovery_rejects_non_coda_contexts() {
        assert!(recover_kw("teha").is_empty());
        assert!(recover_kw("teka").is_empty());
        assert!(recover_kw("tehλ").is_empty());

        for (input, unexpected) in [("tehaki", "teκaki"), ("tekali", "teκali")] {
            assert!(
                !reconstruct_sep(input)
                    .candidates
                    .iter()
                    .any(|candidate| candidate.atomic == unexpected)
            );
        }
    }

    #[test]
    fn ll_hl_recovery_preserves_ambiguity_and_projects_forward() {
        for (input, expected) in [
            ("uipijli", "wipilli"),
            ("siuapijli", "siwapilli"),
            ("kesadihla", "kesadilla"),
        ] {
            let observed = to_atomic(input);
            let result = reconstruct_sep(input);

            assert_eq!(result.status, CandidateStatus::Ambiguous);
            let recovered = candidate(&result, expected);
            assert!(recovered.derivations.contains(&Derivation::LlHlRecovery));
            assert_eq!(apply_m2f(&recovered.atomic), observed);
        }
    }

    #[test]
    fn ll_hl_recovery_only_reverses_the_forward_rule_context() {
        assert!(recover_ll_from_hl("wipihli").contains(&"wipilli".to_string()));
        assert!(recover_ll_from_hl("wipihlo").is_empty());
        assert!(recover_ll_from_hl("wiphli").is_empty());
    }

    #[test]
    fn gemination_recovery_preserves_ambiguity() {
        let result = reconstruct_sep("yoli");

        assert_eq!(result.status, CandidateStatus::Ambiguous);
        assert_eq!(candidate(&result, "yoli").ack, "yoli");
        assert_eq!(candidate(&result, "yolli").ack, "yolli");
        assert!(
            candidate(&result, "yolli")
                .derivations
                .contains(&Derivation::Gemination)
        );
    }

    #[test]
    fn candidates_are_deduplicated_and_derivations_merged() {
        let result = reconstruct_sep("pehki");
        let candidate = candidate(&result, "pewki");

        assert_eq!(
            result
                .candidates
                .iter()
                .filter(|candidate| candidate.atomic == "pewki")
                .count(),
            1
        );
        assert!(candidate.derivations.contains(&Derivation::FToM));
        assert!(
            candidate
                .derivations
                .contains(&Derivation::FToMThenGemination)
        );
    }

    #[test]
    fn every_candidate_projects_to_the_observed_form() {
        for input in ["pehki", "nihneki", "yoli", "kalli"] {
            let observed = to_atomic(input);
            let result = reconstruct_sep(input);

            for candidate in result.candidates {
                let projected = apply_m2f(&candidate.atomic);
                assert!(
                    projections_match(&projected, &observed),
                    "{:?} did not project to {:?}",
                    candidate.atomic,
                    observed
                );
            }
        }
    }

    #[test]
    fn final_w_recovery_xiuh() {
        let result = reconstruct_sep("xij");

        assert!(result.candidates.iter().any(|c| c.atomic == "xiw"));
    }

    #[test]
    fn final_w_recovery_yauh() {
        let result = reconstruct_sep("yaj");

        assert!(result.candidates.iter().any(|c| c.atomic == "yaw"));
    }

    #[test]
    fn final_w_recovery_iuhquinin() {
        let result = reconstruct_sep("ijkinīj");
        assert!(result.candidates.iter().any(|c| c.atomic == "iwkinīn"));
    }

    #[test]
    fn final_w_recovery_xiuhtenan() {
        let result = reconstruct_sep("xijtēnāj");
        assert!(result.candidates.iter().any(|c| c.atomic == "xiwtēnān"));
    }

    #[test]
    fn recovery_chain_h_to_n_then_k() {
        let result = reconstruct_sep("ijkātsaj");
        assert!(result.candidates.iter().any(|c| c.atomic == "iwkāτan"));
    }

    #[test]
    fn regression_iuhquinin() {
        let result = reconstruct_sep("ijkinīj");

        assert!(result.candidates.iter().any(|c| c.ack == "iuhquinīn"));
    }

    #[test]
    fn regression_queuhquinon() {
        let result = reconstruct_sep("kejkinoj");

        assert!(result.candidates.iter().any(|c| c.ack == "queuhquinon"));
    }

    #[test]
    fn combined_recoveries_can_stack() {
        let result = reconstruct_sep("ijkinoj");

        assert!(
            result
                .candidates
                .iter()
                .any(|c| c.derivations.contains(&Derivation::FinalNRecovery)
                    || c.derivations.contains(&Derivation::FinalWRecovery))
        );
    }

    #[test]
    fn chui_is_normalized_correctly() {
        assert_eq!(to_atomic("pachuiliā"), "paςwiliā");
    }

    #[test]
    fn chue_is_normalized_correctly() {
        assert_eq!(to_atomic("cochuetsi"), "koςweτi");
    }

    #[test]
    fn chua_is_normalized_correctly() {
        assert_eq!(to_atomic("cochuatāni"), "koςwatāni");
    }

    #[test]
    fn coda_w_can_apply_after_final_n_recovery() {
        assert!(recover_coda_w("ihkinīn").contains(&"iwkinīn".to_string()));
    }

    #[test]
    fn coda_w_recovers_single_position() {
        let recoveries = recover_coda_w("mahmahtiā");

        assert!(recoveries.contains(&"mahmawtiā".to_string()));
    }

    #[test]
    fn coda_w_generates_partial_recoveries() {
        let recoveries = recover_coda_w("mahmahtiā");

        assert!(recoveries.contains(&"mawmahtiā".to_string()));

        assert!(recoveries.contains(&"mahmawtiā".to_string()));

        assert!(recoveries.contains(&"mawmawtiā".to_string()));
    }

    #[test]
    fn regression_mahmauhtia() {
        let result = reconstruct_sep("majmajtiā");

        assert!(result.candidates.iter().any(|c| c.atomic == "mahmawtiā"));
    }

    #[test]
    fn regression_cuauhmahmantli() {
        let result = reconstruct_sep("kuajmajmāntli");

        assert!(result.candidates.iter().any(|c| c.atomic == "κawmahmānλi"));
    }

    #[test]
    fn regression_icxipetziquiuh() {
        let result = reconstruct_sep("ijxipetsikij");

        assert!(result.candidates.iter().any(|c| c.atomic == "ikxipeτikiw"));
    }

    #[test]
    fn regression_icxipetziquiuhtic() {
        let result = reconstruct_sep("ijxipetsikijtik");

        assert!(
            result
                .candidates
                .iter()
                .any(|c| c.atomic == "ikxipeτikiwtik")
        );
    }

    #[test]
    fn regression_icxopilli() {
        let result = reconstruct_sep("ijxopijli");

        assert!(result.candidates.iter().any(|c| c.atomic == "ikxopilli"));
    }
}
