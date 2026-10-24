use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordEvaluation {
    pub record_id: String,
    pub ambiguity_classes: Vec<String>,
    pub accepted_analyses: Vec<String>,
    pub candidates: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricSummary {
    pub record_count: usize,
    pub candidate_count: usize,
    pub matching_candidate_count: usize,
    pub accepted_analysis_count: usize,
    pub top_1_accuracy: Option<f64>,
    pub top_3_accuracy: Option<f64>,
    pub top_k_accuracy: Option<f64>,
    pub candidate_recall_macro: Option<f64>,
    pub candidate_recall_micro: Option<f64>,
    pub exact_set_coverage: Option<f64>,
    pub candidate_precision_macro: Option<f64>,
    pub candidate_precision_micro: Option<f64>,
    pub unsupported_candidate_rate: Option<f64>,
    pub average_candidate_set_size: Option<f64>,
    pub maximum_candidate_set_size: usize,
    pub median_candidate_set_size: Option<f64>,
    pub no_candidate_rate: Option<f64>,
    pub system_ambiguity_rate: Option<f64>,
    pub unique_candidate_rate: Option<f64>,
    pub gold_ambiguity_rate: Option<f64>,
}

pub fn summarize(records: &[RecordEvaluation], top_k: usize) -> MetricSummary {
    let mut candidate_count = 0;
    let mut matching_candidate_count = 0;
    let mut accepted_analysis_count = 0;
    let mut top_1_hits = 0;
    let mut top_3_hits = 0;
    let mut top_k_hits = 0;
    let mut exact_set_hits = 0;
    let mut no_candidate_count = 0;
    let mut system_ambiguity_count = 0;
    let mut unique_candidate_count = 0;
    let mut gold_ambiguity_count = 0;
    let mut macro_recall_total = 0.0;
    let mut macro_precision_total = 0.0;
    let mut precision_record_count = 0;
    let mut candidate_sizes = Vec::with_capacity(records.len());

    for record in records {
        let gold: BTreeSet<&str> = record
            .accepted_analyses
            .iter()
            .map(String::as_str)
            .collect();
        let candidates: Vec<&str> = record.candidates.iter().map(String::as_str).collect();
        let candidate_set: BTreeSet<&str> = candidates.iter().copied().collect();
        let matches = candidate_set.intersection(&gold).count();
        let size = candidate_set.len();

        candidate_count += size;
        matching_candidate_count += matches;
        accepted_analysis_count += gold.len();
        candidate_sizes.push(size);

        let matched_at = candidates
            .iter()
            .position(|candidate| gold.contains(candidate));
        if matched_at.is_some_and(|index| index == 0) {
            top_1_hits += 1;
        }
        if matched_at.is_some_and(|index| index < 3) {
            top_3_hits += 1;
        }
        if matched_at.is_some_and(|index| index < top_k) {
            top_k_hits += 1;
        }
        if gold.is_subset(&candidate_set) {
            exact_set_hits += 1;
        }
        if size == 0 {
            no_candidate_count += 1;
        } else {
            if size > 1 {
                system_ambiguity_count += 1;
            } else {
                unique_candidate_count += 1;
            }
            macro_precision_total += matches as f64 / size as f64;
            precision_record_count += 1;
        }
        if gold.len() > 1 {
            gold_ambiguity_count += 1;
        }
        macro_recall_total += matches as f64 / gold.len() as f64;
    }

    let record_count = records.len();
    let mut sorted_sizes = candidate_sizes.clone();
    sorted_sizes.sort_unstable();
    let median_candidate_set_size = if sorted_sizes.is_empty() {
        None
    } else {
        let mid = sorted_sizes.len() / 2;
        Some(if sorted_sizes.len() % 2 == 0 {
            (sorted_sizes[mid - 1] as f64 + sorted_sizes[mid] as f64) / 2.0
        } else {
            sorted_sizes[mid] as f64
        })
    };

    MetricSummary {
        record_count,
        candidate_count,
        matching_candidate_count,
        accepted_analysis_count,
        top_1_accuracy: ratio(top_1_hits, record_count),
        top_3_accuracy: ratio(top_3_hits, record_count),
        top_k_accuracy: ratio(top_k_hits, record_count),
        candidate_recall_macro: ratio_float(macro_recall_total, record_count),
        candidate_recall_micro: ratio(matching_candidate_count, accepted_analysis_count),
        exact_set_coverage: ratio(exact_set_hits, record_count),
        candidate_precision_macro: ratio_float(macro_precision_total, precision_record_count),
        candidate_precision_micro: ratio(matching_candidate_count, candidate_count),
        unsupported_candidate_rate: ratio(
            candidate_count - matching_candidate_count,
            candidate_count,
        ),
        average_candidate_set_size: ratio(candidate_count, record_count),
        maximum_candidate_set_size: candidate_sizes.iter().copied().max().unwrap_or(0),
        median_candidate_set_size,
        no_candidate_rate: ratio(no_candidate_count, record_count),
        system_ambiguity_rate: ratio(system_ambiguity_count, record_count),
        unique_candidate_rate: ratio(unique_candidate_count, record_count),
        gold_ambiguity_rate: ratio(gold_ambiguity_count, record_count),
    }
}

fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then_some(numerator as f64 / denominator as f64)
}

fn ratio_float(numerator: f64, denominator: usize) -> Option<f64> {
    (denominator > 0).then_some(numerator / denominator as f64)
}

#[cfg(test)]
mod tests {
    use super::{RecordEvaluation, summarize};

    fn record(id: &str, gold: &[&str], candidates: &[&str]) -> RecordEvaluation {
        RecordEvaluation {
            record_id: id.to_owned(),
            ambiguity_classes: vec![],
            accepted_analyses: gold.iter().map(|value| (*value).to_owned()).collect(),
            candidates: candidates.iter().map(|value| (*value).to_owned()).collect(),
        }
    }

    #[test]
    fn scores_multiple_acceptable_analyses_as_a_set() {
        let metrics = summarize(&[record("r1", &["a", "b"], &["c", "b", "a"])], 2);

        assert_eq!(metrics.top_1_accuracy, Some(0.0));
        assert_eq!(metrics.top_3_accuracy, Some(1.0));
        assert_eq!(metrics.top_k_accuracy, Some(1.0));
        assert_eq!(metrics.candidate_recall_macro, Some(1.0));
        assert_eq!(metrics.candidate_precision_macro, Some(2.0 / 3.0));
        assert_eq!(metrics.exact_set_coverage, Some(1.0));
    }

    #[test]
    fn keeps_zero_candidate_records_in_accuracy_and_recall_denominators() {
        let metrics = summarize(
            &[record("r1", &["a"], &["a"]), record("r2", &["b"], &[])],
            3,
        );

        assert_eq!(metrics.top_1_accuracy, Some(0.5));
        assert_eq!(metrics.candidate_recall_macro, Some(0.5));
        assert_eq!(metrics.candidate_precision_macro, Some(1.0));
        assert_eq!(metrics.no_candidate_rate, Some(0.5));
        assert_eq!(metrics.average_candidate_set_size, Some(0.5));
    }

    #[test]
    fn undefined_ratios_are_null_and_empty_inputs_are_safe() {
        let metrics = summarize(&[], 3);

        assert_eq!(metrics.top_1_accuracy, None);
        assert_eq!(metrics.candidate_precision_micro, None);
        assert_eq!(metrics.average_candidate_set_size, None);
        assert_eq!(metrics.maximum_candidate_set_size, 0);
    }
}
