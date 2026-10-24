use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::metrics::MetricSummary;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkReport {
    #[serde(flatten)]
    pub metadata: BenchmarkMetadata,
    pub configured_top_k: usize,
    pub total_record_count: usize,
    pub excluded_record_count: usize,
    pub excluded_records: Vec<ExcludedRecord>,
    pub metric_definitions: Vec<MetricDefinition>,
    pub metrics: MetricSummary,
    pub ambiguity_class_metrics: BTreeMap<String, MetricSummary>,
    pub record_results: Vec<RecordResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkMetadata {
    pub corpus_id: String,
    pub corpus_version: String,
    pub schema_version: String,
    pub normalization_version: String,
    pub run_timestamp: String,
    pub git_commit: Option<String>,
    pub git_working_tree_clean: Option<bool>,
    pub repository_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lexical_resource_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lexical_resource_checksum: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExcludedRecord {
    pub record_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricDefinition {
    pub name: String,
    pub definition: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateEvaluation {
    pub atomic: String,
    pub ack: String,
    pub score: i32,
    pub derivations: Vec<String>,
    pub projection: String,
    pub projection_matches_observed: bool,
    pub matches_accepted_analysis: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordResult {
    pub record_id: String,
    pub status: String,
    pub ambiguity_classes: Vec<String>,
    pub accepted_atomic: Vec<String>,
    pub candidates: Vec<CandidateEvaluation>,
}

impl BenchmarkReport {
    pub fn to_markdown(&self) -> String {
        let mut output = format!(
            "# SEP reconstruction evaluation\n\n\
             - Corpus: `{}` version `{}`\n\
             - Schema: `{}`\n\
             - Normalization: `{}`\n\
             - Run timestamp (UTC): `{}`\n\
             - Git commit: `{}`\n\
             - Git working tree clean: `{}`\n\
             - Repository version: `{}`\n\
             - Configured top-k: {}\n\
             - Records: {} total, {} eligible, {} excluded\n\n\
             ## Aggregate metrics\n\n\
             | Metric | Value |\n|---|---:|\n",
            self.metadata.corpus_id,
            self.metadata.corpus_version,
            self.metadata.schema_version,
            self.metadata.normalization_version,
            self.metadata.run_timestamp,
            self.metadata.git_commit.as_deref().unwrap_or("unavailable"),
            self.metadata
                .git_working_tree_clean
                .map(|clean| clean.to_string())
                .as_deref()
                .unwrap_or("unavailable"),
            self.metadata.repository_version,
            self.configured_top_k,
            self.total_record_count,
            self.metrics.record_count,
            self.excluded_record_count,
        );
        append_metrics(&mut output, &self.metrics, self.configured_top_k);

        output.push_str("\n## Metric definitions\n\n");
        for definition in &self.metric_definitions {
            output.push_str(&format!(
                "- **{}:** {}\n",
                definition.name, definition.definition
            ));
        }

        output.push_str("\n## Excluded records\n\n");
        if self.excluded_records.is_empty() {
            output.push_str("None.\n");
        } else {
            output.push_str("| Record | Reason |\n|---|---|\n");
            for record in &self.excluded_records {
                output.push_str(&format!(
                    "| `{}` | {} |\n",
                    escape_table(&record.record_id),
                    escape_table(&record.reason)
                ));
            }
        }

        output.push_str("\n## Ambiguity-class breakdown\n\n");
        if self.ambiguity_class_metrics.is_empty() {
            output.push_str("No eligible records have ambiguity-class labels.\n");
        } else {
            output.push_str("| Class | Records | Top-1 | Top-3 | Top-k | Recall (macro) | Precision (micro) | No-candidate |\n|---|---:|---:|---:|---:|---:|---:|---:|\n");
            for (class, metrics) in &self.ambiguity_class_metrics {
                output.push_str(&format!(
                    "| `{class}` | {} | {} | {} | {} | {} | {} | {} |\n",
                    metrics.record_count,
                    format_metric(metrics.top_1_accuracy),
                    format_metric(metrics.top_3_accuracy),
                    format_metric(metrics.top_k_accuracy),
                    format_metric(metrics.candidate_recall_macro),
                    format_metric(metrics.candidate_precision_micro),
                    format_metric(metrics.no_candidate_rate),
                ));
            }
        }

        output.push_str("\n## Per-record candidates\n\n");
        for record in &self.record_results {
            output.push_str(&format!(
                "### `{}`\n\nStatus: `{}`; ambiguity classes: `{}`; accepted Atomic: `{}`.\n\n",
                record.record_id,
                record.status,
                record.ambiguity_classes.join(", "),
                record.accepted_atomic.join(", "),
            ));
            if record.candidates.is_empty() {
                output.push_str("No candidates returned.\n\n");
            } else {
                output.push_str("| Rank | Atomic | ACK | Score | Derivations | Projection match | Accepted |\n|---:|---|---|---:|---|---|---|\n");
                for (rank, candidate) in record.candidates.iter().enumerate() {
                    output.push_str(&format!(
                        "| {} | `{}` | `{}` | {} | {} | {} | {} |\n",
                        rank + 1,
                        escape_table(&candidate.atomic),
                        escape_table(&candidate.ack),
                        candidate.score,
                        escape_table(&candidate.derivations.join(", ")),
                        candidate.projection_matches_observed,
                        candidate.matches_accepted_analysis,
                    ));
                }
                output.push('\n');
            }
        }
        output
    }
}

fn append_metrics(output: &mut String, metrics: &MetricSummary, top_k: usize) {
    let rows = [
        ("Top-1 accuracy", metrics.top_1_accuracy),
        ("Top-3 accuracy", metrics.top_3_accuracy),
        ("Top-k accuracy", metrics.top_k_accuracy),
        ("Candidate recall (macro)", metrics.candidate_recall_macro),
        ("Candidate recall (micro)", metrics.candidate_recall_micro),
        ("Exact-set coverage", metrics.exact_set_coverage),
        (
            "Candidate precision (macro, nonempty sets)",
            metrics.candidate_precision_macro,
        ),
        (
            "Candidate precision (micro)",
            metrics.candidate_precision_micro,
        ),
        (
            "Unsupported candidate rate",
            metrics.unsupported_candidate_rate,
        ),
        (
            "Average candidate-set size",
            metrics.average_candidate_set_size,
        ),
        (
            "Median candidate-set size",
            metrics.median_candidate_set_size,
        ),
        ("No-candidate rate", metrics.no_candidate_rate),
        ("System ambiguity rate", metrics.system_ambiguity_rate),
        ("Unique-candidate rate", metrics.unique_candidate_rate),
        ("Gold ambiguity rate", metrics.gold_ambiguity_rate),
    ];
    for (name, value) in rows {
        let name = if name == "Top-k accuracy" {
            format!("Top-{top_k} accuracy")
        } else {
            name.to_owned()
        };
        output.push_str(&format!("| {name} | {} |\n", format_metric(value)));
    }
    output.push_str(&format!(
        "| Candidate count | {} |\n\
         | Matching candidate count | {} |\n\
         | Accepted analysis count | {} |\n\
         | Maximum candidate-set size | {} |\n",
        metrics.candidate_count,
        metrics.matching_candidate_count,
        metrics.accepted_analysis_count,
        metrics.maximum_candidate_set_size,
    ));
}

fn format_metric(value: Option<f64>) -> String {
    value
        .map(|value| format!("{:.4}", value))
        .unwrap_or_else(|| "n/a".to_owned())
}

fn escape_table(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::{BenchmarkReport, MetricDefinition, MetricSummary};
    use std::collections::BTreeMap;

    #[test]
    fn markdown_contains_reproducibility_metadata_and_metric_denominators() {
        let report = BenchmarkReport {
            metadata: super::BenchmarkMetadata {
                corpus_id: "fixture".into(),
                corpus_version: "1".into(),
                schema_version: "schema".into(),
                normalization_version: "norm".into(),
                run_timestamp: "2026-01-01T00:00:00Z".into(),
                git_commit: None,
                git_working_tree_clean: None,
                repository_version: "0.1".into(),
                lexical_resource_path: None,
                lexical_resource_checksum: None,
            },
            configured_top_k: 3,
            total_record_count: 1,
            excluded_record_count: 0,
            excluded_records: vec![],
            metric_definitions: vec![MetricDefinition {
                name: "Top-1".into(),
                definition: "Set-valued correctness.".into(),
            }],
            metrics: MetricSummary {
                record_count: 1,
                candidate_count: 1,
                matching_candidate_count: 1,
                accepted_analysis_count: 1,
                top_1_accuracy: Some(1.0),
                top_3_accuracy: Some(1.0),
                top_k_accuracy: Some(1.0),
                candidate_recall_macro: Some(1.0),
                candidate_recall_micro: Some(1.0),
                exact_set_coverage: Some(1.0),
                candidate_precision_macro: Some(1.0),
                candidate_precision_micro: Some(1.0),
                unsupported_candidate_rate: Some(0.0),
                average_candidate_set_size: Some(1.0),
                maximum_candidate_set_size: 1,
                median_candidate_set_size: Some(1.0),
                no_candidate_rate: Some(0.0),
                system_ambiguity_rate: Some(0.0),
                unique_candidate_rate: Some(1.0),
                gold_ambiguity_rate: Some(0.0),
            },
            ambiguity_class_metrics: BTreeMap::new(),
            record_results: vec![],
        };

        let markdown = report.to_markdown();
        assert!(markdown.contains("Run timestamp (UTC)"));
        assert!(markdown.contains("Candidate count"));
        assert!(markdown.contains("Top-3 accuracy"));
        assert!(markdown.contains("None."));
    }
}
