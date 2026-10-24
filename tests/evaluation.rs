use std::path::{Path, PathBuf};

use nahuatl_transcoder::evaluation::validate_corpus_dir;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/evaluation")
        .join(name)
}

#[test]
fn validates_synthetic_corpus_fixture() {
    let corpus = validate_corpus_dir(fixture("valid_corpus")).expect("fixture should validate");
    assert_eq!(corpus.manifest.corpus_id, "validator-fixture");
    assert_eq!(corpus.records.len(), 1);
    assert_eq!(corpus.records[0].gold.analyses.len(), 2);
}

#[test]
fn rejects_invalid_corpus_fixtures_with_field_paths() {
    let cases = [
        ("duplicate_ids", "record_id", "duplicate record ID"),
        ("bad_status", "gold.status", "invalid value"),
        (
            "bad_ambiguity",
            "gold.ambiguity_classes[0]",
            "invalid value",
        ),
        ("missing_analysis", "gold.analyses", "requires exactly one"),
        ("missing_required", "license", "required field is missing"),
        ("invalid_split", "provenance.split", "invalid value"),
        (
            "missing_source",
            "source.source_id",
            "unknown source reference",
        ),
        (
            "manifest_version",
            "schema_version",
            "unsupported manifest schema version",
        ),
        (
            "duplicate_analysis_ids",
            "gold.analyses[1].analysis_id",
            "duplicate analysis ID",
        ),
    ];

    for (directory, field_path, message) in cases {
        let errors = validate_corpus_dir(fixture(directory))
            .expect_err("invalid fixture should be rejected");
        assert!(
            errors
                .iter()
                .any(|error| error.field_path == field_path && error.message.contains(message)),
            "expected {field_path:?} containing {message:?}, got {errors:?}"
        );
    }
}

#[test]
fn reports_record_id_for_record_validation_errors() {
    let errors =
        validate_corpus_dir(fixture("bad_status")).expect_err("invalid status should be rejected");
    assert!(
        errors.iter().any(|error| {
            error.record_id == "bad-status-1" && error.field_path == "gold.status"
        })
    );
}

#[test]
fn reports_line_markers_for_invalid_json_and_utf8() {
    let temp = std::env::temp_dir().join(format!(
        "nahuatl-evaluation-validator-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after UNIX epoch")
            .as_nanos()
    ));
    std::fs::create_dir(&temp).expect("temporary corpus directory should be created");
    std::fs::copy(
        fixture("valid_corpus").join("manifest.json"),
        temp.join("manifest.json"),
    )
    .expect("fixture manifest should be copied");

    std::fs::write(temp.join("records.jsonl"), b"\n\xff")
        .expect("invalid UTF-8 test record should be written");
    let utf8_errors = validate_corpus_dir(&temp).expect_err("invalid UTF-8 must fail");
    assert!(
        utf8_errors
            .iter()
            .any(|error| error.record_id == "<line:2>" && error.field_path == "$utf8")
    );

    std::fs::write(temp.join("records.jsonl"), b"\n{not-json}\n")
        .expect("invalid JSON test record should be written");
    let json_errors = validate_corpus_dir(&temp).expect_err("invalid JSON must fail");
    assert!(
        json_errors
            .iter()
            .any(|error| error.record_id == "<line:2>" && error.field_path == "$")
    );

    std::fs::remove_dir_all(temp).expect("temporary corpus directory should be removed");
}
