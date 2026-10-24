use crate::atomic::{is_atomic_consonant, is_atomic_vowel, to_atomic};
use crate::geminate::geminate;
use crate::orthography::atomic_to_ack;
use crate::phonetics::degeminate;
use crate::{apply_f2m, apply_m2f};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub atomic: String,
    pub ack: String,
    pub derivations: Vec<Derivation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateStatus {
    NoCandidates,
    Unique,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateSet {
    pub candidates: Vec<Candidate>,
    pub status: CandidateStatus,
}

fn projections_match(projected: &str, observed: &str) -> bool {
    projected == observed || degeminate(projected) == degeminate(observed)
}

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

        let coda_w_recovery = recover_coda_w(&observed);
        if coda_w_recovery != observed {
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

    CandidateSet { candidates, status }
}

fn recover_coda_w(observed: &str) -> String {
    let chars: Vec<char> = observed.chars().collect();
    let mut recovered = String::with_capacity(observed.len());

    for (index, &current) in chars.iter().enumerate() {
        let is_coda_h = current == 'h'
            && index > 0
            && index + 1 < chars.len()
            && is_atomic_vowel(chars[index - 1])
            && is_atomic_consonant(chars[index + 1]);

        recovered.push(if is_coda_h { 'w' } else { current });
    }

    recovered
}

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
        assert_eq!(recover_coda_w(&observed), observed);
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
}
