use std::path::{Path, PathBuf};

use nahuatl_transcoder::evaluation::{GoldStatus, evaluate_corpus, validate_corpus_dir};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/evaluation")
        .join(name)
}

#[test]
fn evaluates_valid_fixture_and_reports_reconstruction_candidates() {
    let corpus = validate_corpus_dir(fixture("valid_corpus")).expect("fixture should validate");
    let report = evaluate_corpus(&corpus, 2);

    assert_eq!(report.corpus_id, "validator-fixture");
    assert_eq!(report.corpus_version, "0.1.0");
    assert_eq!(report.configured_top_k, 2);
    assert_eq!(report.total_record_count, 1);
    assert_eq!(report.metrics.record_count, 1);
    assert_eq!(report.excluded_record_count, 0);
    assert_eq!(report.record_results.len(), 1);
    assert!(
        report.record_results[0]
            .candidates
            .iter()
            .all(|candidate| { candidate.projection_matches_observed })
    );
    assert!(
        report.record_results[0]
            .candidates
            .iter()
            .any(|candidate| candidate.matches_accepted_analysis)
    );
    assert!(report.ambiguity_class_metrics.contains_key("final_n_vs_h"));

    let json = serde_json::to_string(&report).expect("report should serialize");
    assert!(json.contains("\"top_k_accuracy\""));
    assert!(report.to_markdown().contains("Ambiguity-class breakdown"));
}

#[test]
fn excludes_insufficient_evidence_and_reports_it_separately() {
    let mut corpus = validate_corpus_dir(fixture("valid_corpus")).expect("fixture should validate");
    corpus.records[0].gold.status = GoldStatus::InsufficientEvidence;
    corpus.records[0].gold.analyses.clear();
    let report = evaluate_corpus(&corpus, 3);

    assert_eq!(report.total_record_count, 1);
    assert_eq!(report.excluded_record_count, 1);
    assert_eq!(report.excluded_records[0].record_id, "fixture-1");
    assert_eq!(report.metrics.record_count, 0);
    assert_eq!(report.metrics.top_1_accuracy, None);
}
