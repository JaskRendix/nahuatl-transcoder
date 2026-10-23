use nahuatl_transcoder::{
    orthography::atomic_to_ack,
    reconstruction::{Candidate, CandidateSet, CandidateStatus, reconstruct_sep},
};

fn assert_ambiguous_candidate<'a>(set: &'a CandidateSet, expected_atomic: &str) -> &'a Candidate {
    let candidate = set
        .candidates
        .iter()
        .find(|candidate| candidate.atomic == expected_atomic)
        .unwrap_or_else(|| {
            panic!(
                "missing expected Atomic candidate {expected_atomic:?}; got {:?}",
                set.candidates
                    .iter()
                    .map(|candidate| candidate.atomic.as_str())
                    .collect::<Vec<_>>()
            )
        });

    assert_eq!(
        set.status,
        CandidateStatus::Ambiguous,
        "expected multiple underlying forms, got {:?}",
        set.candidates
            .iter()
            .map(|candidate| candidate.atomic.as_str())
            .collect::<Vec<_>>()
    );
    assert_eq!(candidate.ack, atomic_to_ack(expected_atomic));
    candidate
}

fn assert_lacks_candidate(input: &str, unexpected_atomic: &str) {
    let result = reconstruct_sep(input);
    assert!(
        !result
            .candidates
            .iter()
            .any(|candidate| candidate.atomic == unexpected_atomic),
        "unexpected candidate {unexpected_atomic:?} for {input:?}"
    );
}

#[test]
fn final_n_to_h_recovers_nosin_and_preserves_ambiguity() {
    let result = reconstruct_sep("nosij");

    let candidate = assert_ambiguous_candidate(&result, "nosin");
    assert_eq!(candidate.ack, "nocin");
}

#[test]
fn coda_w_to_h_recovers_benchmark_forms() {
    for (sep, expected_atomic) in [("kuajtli", "κawλi"), ("uajtli", "wawλi")] {
        let result = reconstruct_sep(sep);
        assert_ambiguous_candidate(&result, expected_atomic);
    }
}

#[test]
fn kw_and_k_lenition_recovers_tecuhtli() {
    for (sep, expected_atomic) in [("tejtli", "teκλi"), ("mijtlantektli", "mikλanteκλi")] {
        let result = reconstruct_sep(sep);
        assert_ambiguous_candidate(&result, expected_atomic);
    }
}

#[test]
fn k_lenition_recovers_mictlan() {
    let result = reconstruct_sep("mijtlaj");

    assert_ambiguous_candidate(&result, "mikλan");
}

#[test]
fn existing_f2m_rule_recovers_nikneki_without_resolving_ambiguity() {
    let existing_f2m_case = reconstruct_sep("nijneki");
    assert_ambiguous_candidate(&existing_f2m_case, "nikneki");
}

#[test]
fn ll_hl_alternation_recovers_benchmark_forms() {
    for (sep, expected_atomic) in [
        ("uipijli", "wipilli"),
        ("siuapijli", "siwapilli"),
        ("kesadihla", "kesadilla"),
    ] {
        let result = reconstruct_sep(sep);
        assert_ambiguous_candidate(&result, expected_atomic);
    }
}

#[test]
fn gemination_interaction_retains_underlying_and_geminated_forms() {
    let result = reconstruct_sep("tenochtitlaj");

    assert_ambiguous_candidate(&result, "tenoςtiλan");

    let geminated = result
        .candidates
        .iter()
        .find(|candidate| candidate.atomic == "tennoςtiλah")
        .expect("expected geminated candidate");

    assert_eq!(geminated.ack, atomic_to_ack("tennoςtiλah"));
}

#[test]
fn known_gemination_rule_generates_an_ambiguous_candidate() {
    let result = reconstruct_sep("yoli");

    assert_eq!(result.status, CandidateStatus::Ambiguous);
    assert!(
        result
            .candidates
            .iter()
            .any(|candidate| candidate.atomic == "yoli")
    );
    assert!(
        result
            .candidates
            .iter()
            .any(|candidate| candidate.atomic == "yolli")
    );
}

#[test]
fn negative_controls_do_not_create_unlicensed_alternatives() {
    assert_lacks_candidate("nosi", "nosin");
    assert_lacks_candidate("kuatli", "κawλi");
    assert_lacks_candidate("tekali", "teκali");
    assert_lacks_candidate("tehaki", "teκaki");
    assert_lacks_candidate("uipili", "wipilli");
    assert_lacks_candidate("weli", "welli");
    assert_lacks_candidate("piyali", "piyalli");
}
