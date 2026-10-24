use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::atomic::to_atomic;
use crate::phonetics::degeminate;
use crate::reconstruction::{Candidate, Derivation, reconstruct_sep};
use crate::{apply_m2f, evaluation::schema::GoldStatus};

use super::metrics::{RecordEvaluation, summarize};
use super::report::{
    BenchmarkReport, CandidateEvaluation, ExcludedRecord, MetricDefinition, RecordResult,
};
use super::validation::ValidatedCorpus;

pub fn evaluate_corpus(corpus: &ValidatedCorpus, top_k: usize) -> BenchmarkReport {
    let mut evaluations = Vec::new();
    let mut excluded_records = Vec::new();
    let mut record_results = Vec::new();

    for record in &corpus.records {
        if record.gold.status == GoldStatus::InsufficientEvidence {
            excluded_records.push(ExcludedRecord {
                record_id: record.record_id.clone(),
                reason: "insufficient_evidence: no adjudicated Atomic analysis".to_owned(),
            });
            continue;
        }

        let accepted: BTreeSet<String> = record
            .gold
            .analyses
            .iter()
            .map(|analysis| to_atomic(&analysis.atomic))
            .collect();
        let candidate_set = reconstruct_sep(&record.sep.normalized);
        let candidates: Vec<String> = candidate_set
            .candidates
            .iter()
            .map(|candidate| to_atomic(&candidate.atomic))
            .collect();
        let ambiguity_classes: Vec<String> = record
            .gold
            .ambiguity_classes
            .iter()
            .map(|class| class.as_str().to_owned())
            .collect();

        evaluations.push(RecordEvaluation {
            record_id: record.record_id.clone(),
            ambiguity_classes: ambiguity_classes.clone(),
            accepted_analyses: accepted.iter().cloned().collect(),
            candidates: candidates.clone(),
        });
        record_results.push(record_result(
            &record.record_id,
            record.gold.status,
            ambiguity_classes,
            accepted,
            &candidate_set.candidates,
            &record.sep.normalized,
        ));
    }

    let metrics = summarize(&evaluations, top_k);
    let ambiguity_class_metrics = class_metrics(&evaluations, top_k);
    BenchmarkReport {
        corpus_id: corpus.manifest.corpus_id.clone(),
        corpus_version: corpus.manifest.version.clone(),
        schema_version: corpus.manifest.schema_version.clone(),
        normalization_version: corpus.manifest.normalization_version.clone(),
        run_timestamp: run_timestamp(),
        git_commit: git_commit(),
        git_working_tree_clean: git_working_tree_clean(),
        repository_version: env!("CARGO_PKG_VERSION").to_owned(),
        configured_top_k: top_k,
        total_record_count: corpus.records.len(),
        excluded_record_count: excluded_records.len(),
        excluded_records,
        metric_definitions: metric_definitions(top_k),
        metrics,
        ambiguity_class_metrics,
        record_results,
    }
}

fn class_metrics(
    records: &[RecordEvaluation],
    top_k: usize,
) -> BTreeMap<String, super::metrics::MetricSummary> {
    let classes: BTreeSet<String> = records
        .iter()
        .flat_map(|record| record.ambiguity_classes.iter().cloned())
        .collect();
    classes
        .into_iter()
        .map(|class| {
            let class_records: Vec<RecordEvaluation> = records
                .iter()
                .filter(|record| record.ambiguity_classes.contains(&class))
                .cloned()
                .collect();
            (class, summarize(&class_records, top_k))
        })
        .collect()
}

fn record_result(
    record_id: &str,
    status: GoldStatus,
    ambiguity_classes: Vec<String>,
    accepted: BTreeSet<String>,
    candidates: &[Candidate],
    observed: &str,
) -> RecordResult {
    let normalized_observed = to_atomic(observed);
    let candidate_evaluations = candidates
        .iter()
        .map(|candidate| {
            let projection = apply_m2f(&candidate.atomic);
            CandidateEvaluation {
                atomic: candidate.atomic.clone(),
                ack: candidate.ack.clone(),
                score: candidate.score(),
                derivations: candidate
                    .derivations
                    .iter()
                    .copied()
                    .map(derivation_name)
                    .map(str::to_owned)
                    .collect(),
                projection_matches_observed: projection == normalized_observed
                    || degeminate(&projection) == degeminate(&normalized_observed),
                projection,
                matches_accepted_analysis: accepted.contains(&to_atomic(&candidate.atomic)),
            }
        })
        .collect();
    RecordResult {
        record_id: record_id.to_owned(),
        status: status_name(status).to_owned(),
        ambiguity_classes,
        accepted_atomic: accepted.into_iter().collect(),
        candidates: candidate_evaluations,
    }
}

fn derivation_name(derivation: Derivation) -> &'static str {
    match derivation {
        Derivation::Unchanged => "unchanged",
        Derivation::FToM => "f_to_m",
        Derivation::FinalNRecovery => "final_n_recovery",
        Derivation::FinalWRecovery => "final_w_recovery",
        Derivation::CodaWRecovery => "coda_w_recovery",
        Derivation::KLenitionRecovery => "k_lenition_recovery",
        Derivation::KwRecovery => "kw_recovery",
        Derivation::LlHlRecovery => "ll_hl_recovery",
        Derivation::Gemination => "gemination",
        Derivation::FToMThenGemination => "f_to_m_then_gemination",
        Derivation::GeminationThenFToM => "gemination_then_f_to_m",
    }
}

fn status_name(status: GoldStatus) -> &'static str {
    match status {
        GoldStatus::SingleAnalysis => "single_analysis",
        GoldStatus::MultipleAnalyses => "multiple_analyses",
        GoldStatus::Underspecified => "underspecified",
        GoldStatus::InsufficientEvidence => "insufficient_evidence",
    }
}

fn metric_definitions(top_k: usize) -> Vec<MetricDefinition> {
    vec![
        definition(
            "Top-1 accuracy",
            "Eligible records whose first candidate is in the accepted Atomic set; no-candidate records count as incorrect.",
        ),
        definition(
            "Top-3 accuracy",
            "Eligible records with any accepted Atomic analysis among the first three candidates.",
        ),
        definition(
            &format!("Top-{top_k} accuracy"),
            "Eligible records with any accepted Atomic analysis among the first configured k candidates.",
        ),
        definition(
            "Candidate recall",
            "Accepted Atomic analyses present in the candidate set; report macro per-record and pooled micro values. Empty candidate sets contribute zero recall.",
        ),
        definition(
            "Exact-set coverage",
            "Eligible records for which every accepted Atomic analysis appears among candidates.",
        ),
        definition(
            "Candidate precision",
            "Returned candidates present in the accepted Atomic set; macro precision averages records with candidates, while micro precision is pooled.",
        ),
        definition(
            "Unsupported candidate rate",
            "One minus micro candidate precision; absent from this annotation does not imply linguistically false.",
        ),
        definition(
            "Average candidate-set size",
            "Mean number of distinct Atomic candidates per eligible record; empty candidate sets count as zero.",
        ),
        definition(
            "Maximum candidate-set size",
            "Largest distinct candidate set among eligible records.",
        ),
        definition(
            "No-candidate rate",
            "Fraction of eligible records with zero candidates.",
        ),
        definition(
            "System ambiguity rate",
            "Fraction of eligible records with more than one candidate; this is not gold or linguistic ambiguity.",
        ),
        definition(
            "Gold ambiguity rate",
            "Fraction of eligible records with more than one distinct accepted Atomic analysis.",
        ),
    ]
}

fn definition(name: &str, definition: &str) -> MetricDefinition {
    MetricDefinition {
        name: name.to_owned(),
        definition: definition.to_owned(),
    }
}

pub fn run_timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = (seconds / 86_400) as i64;
    let day_seconds = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        day_seconds / 3_600,
        (day_seconds % 3_600) / 60,
        day_seconds % 60,
    )
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month, day)
}

fn git_commit() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let commit = String::from_utf8(output.stdout).ok()?.trim().to_owned();
    (!commit.is_empty()).then_some(commit)
}

fn git_working_tree_clean() -> Option<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()?;
    output.status.success().then_some(output.stdout.is_empty())
}

#[cfg(test)]
mod tests {
    use super::run_timestamp;

    #[test]
    fn timestamp_uses_utc_rfc3339_shape() {
        let timestamp = run_timestamp();
        assert_eq!(timestamp.len(), 20);
        assert!(timestamp.ends_with('Z'));
        assert_eq!(&timestamp[4..5], "-");
        assert_eq!(&timestamp[10..11], "T");
    }
}
