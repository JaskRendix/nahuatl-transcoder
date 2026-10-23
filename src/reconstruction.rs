use crate::atomic::to_atomic;
use crate::geminate::geminate;
use crate::orthography::atomic_to_ack;
use crate::phonetics::degeminate;
use crate::{apply_f2m, apply_m2f};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Derivation {
    Unchanged,
    FToM,
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
}
