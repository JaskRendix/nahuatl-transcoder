use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::atomic::to_atomic;
use crate::evaluation::metrics::{MetricSummary, RecordEvaluation, summarize};
use crate::reconstruction::{Candidate, reconstruct_sep};
use crate::{convert_all, evaluation::ValidatedCorpus, evaluation::schema::GoldStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateFeatures {
    pub lexical_attestation: bool,
    pub exact_lexicon_match: bool,
    pub normalized_lexicon_match: bool,
    pub derivation_count: usize,
    pub heuristic_score: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RankedCandidate {
    pub atomic: String,
    pub ack: String,
    pub features: CandidateFeatures,
    pub baseline_score: i32,
    pub lexical_score: i32,
    pub final_score: i32,
    pub final_rank: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RankerSelection {
    Baseline,
    LexicalPrior,
    HeuristicOnly,
    LexicalOnly,
    Combined,
}

#[derive(Debug, Clone, Default)]
pub struct LexicalInventory {
    exact_atomic: HashSet<String>,
    normalized_atomic: HashSet<String>,
}

impl LexicalInventory {
    pub fn from_atomic_forms(forms: impl IntoIterator<Item = String>) -> Self {
        let exact_atomic: HashSet<String> = forms.into_iter().collect();
        let normalized_atomic = exact_atomic.iter().map(|form| to_atomic(form)).collect();
        Self {
            exact_atomic,
            normalized_atomic,
        }
    }

    pub fn from_headwords(headwords: impl IntoIterator<Item = String>) -> Self {
        let exact_atomic: Vec<String> = headwords
            .into_iter()
            .map(|word| convert_all(&word).atom)
            .collect();
        Self::from_atomic_forms(exact_atomic)
    }

    pub fn load_g2(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let reader = BufReader::new(File::open(path)?);
        let mut headwords = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let Some(headword) = line.trim().split('.').next() else {
                continue;
            };
            let headword = headword.trim();
            if !headword.is_empty() && !headword.contains(' ') {
                headwords.push(headword.to_owned());
            }
        }
        Ok(Self::from_headwords(headwords))
    }

    pub fn len(&self) -> usize {
        self.exact_atomic.len()
    }

    pub fn is_empty(&self) -> bool {
        self.exact_atomic.is_empty()
    }

    fn features(&self, candidate: &Candidate) -> CandidateFeatures {
        let normalized = to_atomic(&candidate.atomic);
        let exact_lexicon_match = self.exact_atomic.contains(&candidate.atomic);
        let normalized_lexicon_match = self.normalized_atomic.contains(&normalized);
        CandidateFeatures {
            lexical_attestation: exact_lexicon_match || normalized_lexicon_match,
            exact_lexicon_match,
            normalized_lexicon_match,
            derivation_count: candidate.derivations.len(),
            heuristic_score: candidate.score(),
        }
    }
}

pub trait Ranker {
    fn rank(&self, candidates: &[Candidate], inventory: &LexicalInventory) -> Vec<RankedCandidate>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BaselineRanker;

impl Ranker for BaselineRanker {
    fn rank(&self, candidates: &[Candidate], inventory: &LexicalInventory) -> Vec<RankedCandidate> {
        rank_with(candidates, inventory, RankerSelection::Baseline)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LexicalPriorRanker;

impl Ranker for LexicalPriorRanker {
    fn rank(&self, candidates: &[Candidate], inventory: &LexicalInventory) -> Vec<RankedCandidate> {
        rank_with(candidates, inventory, RankerSelection::Combined)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AblationResult {
    pub name: String,
    pub metrics: MetricSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordRankingComparison {
    pub record_id: String,
    pub accepted_atomic: Vec<String>,
    pub candidates: Vec<CandidateRankingComparison>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateRankingComparison {
    pub atomic: String,
    pub ack: String,
    pub lexical_attestation: bool,
    pub exact_lexicon_match: bool,
    pub normalized_lexicon_match: bool,
    pub derivation_count: usize,
    pub baseline_score: i32,
    pub lexical_score: i32,
    pub baseline_rank: usize,
    pub lexical_prior_rank: usize,
    pub heuristic_only_rank: usize,
    pub lexical_only_rank: usize,
    pub combined_rank: usize,
    pub final_rank: usize,
    pub accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RankingComparisonReport {
    pub lexical_entry_count: usize,
    pub total_record_count: usize,
    pub excluded_record_count: usize,
    pub excluded_record_ids: Vec<String>,
    pub top_k: usize,
    pub selected_ranker: RankerSelection,
    pub baseline_metrics: MetricSummary,
    pub lexical_prior_metrics: MetricSummary,
    pub ablations: Vec<AblationResult>,
    pub side_by_side: Vec<RecordRankingComparison>,
}

pub fn rank_with(
    candidates: &[Candidate],
    inventory: &LexicalInventory,
    selection: RankerSelection,
) -> Vec<RankedCandidate> {
    let mut ranked: Vec<(usize, CandidateFeatures, i32)> = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            let features = inventory.features(candidate);
            let lexical_score = i32::from(features.exact_lexicon_match) * 2
                + i32::from(features.normalized_lexicon_match);
            (index, features, lexical_score)
        })
        .collect();

    match selection {
        RankerSelection::Baseline | RankerSelection::HeuristicOnly => {
            ranked.sort_by(|left, right| {
                right
                    .1
                    .heuristic_score
                    .cmp(&left.1.heuristic_score)
                    .then_with(|| left.0.cmp(&right.0))
            });
        }
        RankerSelection::LexicalPrior | RankerSelection::Combined => {
            ranked.sort_by(|left, right| {
                combined_score(right.2, right.1.heuristic_score)
                    .cmp(&combined_score(left.2, left.1.heuristic_score))
                    .then_with(|| right.1.heuristic_score.cmp(&left.1.heuristic_score))
                    .then_with(|| left.0.cmp(&right.0))
            });
        }
        RankerSelection::LexicalOnly => {
            ranked.sort_by(|left, right| {
                right
                    .2
                    .cmp(&left.2)
                    .then_with(|| candidates[left.0].atomic.cmp(&candidates[right.0].atomic))
            });
        }
    }

    ranked
        .into_iter()
        .enumerate()
        .map(|(rank, (index, features, lexical_score))| {
            let baseline_score = features.heuristic_score;
            let final_score = match selection {
                RankerSelection::Baseline | RankerSelection::HeuristicOnly => baseline_score,
                RankerSelection::LexicalPrior | RankerSelection::Combined => {
                    combined_score(lexical_score, baseline_score)
                }
                RankerSelection::LexicalOnly => lexical_score,
            };
            RankedCandidate {
                atomic: candidates[index].atomic.clone(),
                ack: candidates[index].ack.clone(),
                features,
                baseline_score,
                lexical_score,
                final_score,
                final_rank: rank + 1,
            }
        })
        .collect()
}

fn combined_score(lexical_score: i32, heuristic_score: i32) -> i32 {
    lexical_score * 100 + heuristic_score
}

pub fn compare_corpus_rankers(
    corpus: &ValidatedCorpus,
    inventory: &LexicalInventory,
    top_k: usize,
) -> RankingComparisonReport {
    compare_corpus_rankers_with_selection(corpus, inventory, top_k, RankerSelection::Combined)
}

pub fn compare_corpus_rankers_with_selection(
    corpus: &ValidatedCorpus,
    inventory: &LexicalInventory,
    top_k: usize,
    selected_ranker: RankerSelection,
) -> RankingComparisonReport {
    let mut excluded_record_ids = Vec::new();
    let mut side_by_side = Vec::new();

    for record in &corpus.records {
        if record.gold.status == GoldStatus::InsufficientEvidence {
            excluded_record_ids.push(record.record_id.clone());
            continue;
        }
        let accepted: BTreeSet<String> = record
            .gold
            .analyses
            .iter()
            .map(|analysis| to_atomic(&analysis.atomic))
            .collect();
        let candidates = reconstruct_sep(&record.sep.normalized).candidates;
        let candidate_orders: BTreeMap<RankerSelection, Vec<RankedCandidate>> = [
            RankerSelection::Baseline,
            RankerSelection::LexicalPrior,
            RankerSelection::HeuristicOnly,
            RankerSelection::LexicalOnly,
            RankerSelection::Combined,
        ]
        .into_iter()
        .map(|selection| (selection, rank_with(&candidates, inventory, selection)))
        .collect();
        let baseline = ranks(&candidate_orders[&RankerSelection::Baseline]);
        let lexical = ranks(&candidate_orders[&RankerSelection::LexicalPrior]);
        let heuristic = ranks(&candidate_orders[&RankerSelection::HeuristicOnly]);
        let lexical_only = ranks(&candidate_orders[&RankerSelection::LexicalOnly]);
        let combined = ranks(&candidate_orders[&RankerSelection::Combined]);
        let candidates = baseline
            .keys()
            .map(|atomic| {
                let baseline_candidate = baseline[atomic];
                CandidateRankingComparison {
                    atomic: atomic.clone(),
                    ack: baseline_candidate.ack.clone(),
                    lexical_attestation: baseline_candidate.features.lexical_attestation,
                    exact_lexicon_match: baseline_candidate.features.exact_lexicon_match,
                    normalized_lexicon_match: baseline_candidate.features.normalized_lexicon_match,
                    derivation_count: baseline_candidate.features.derivation_count,
                    baseline_score: baseline_candidate.baseline_score,
                    lexical_score: baseline_candidate.lexical_score,
                    baseline_rank: baseline_candidate.final_rank,
                    lexical_prior_rank: lexical[atomic].final_rank,
                    heuristic_only_rank: heuristic[atomic].final_rank,
                    lexical_only_rank: lexical_only[atomic].final_rank,
                    combined_rank: combined[atomic].final_rank,
                    final_rank: candidate_orders[&selected_ranker]
                        .iter()
                        .find(|candidate| candidate.atomic == *atomic)
                        .map_or(0, |candidate| candidate.final_rank),
                    accepted: accepted.contains(atomic),
                }
            })
            .collect();
        side_by_side.push(RecordRankingComparison {
            record_id: record.record_id.clone(),
            accepted_atomic: accepted.into_iter().collect(),
            candidates,
        });
    }

    let selections = [
        ("heuristic_only", RankerSelection::HeuristicOnly),
        ("lexical_prior_only", RankerSelection::LexicalOnly),
        ("combined", RankerSelection::Combined),
    ];
    let ablations = selections
        .into_iter()
        .map(|(name, selection)| {
            let ranked_examples: Vec<RecordEvaluation> = corpus
                .records
                .iter()
                .filter(|record| record.gold.status != GoldStatus::InsufficientEvidence)
                .map(|record| {
                    let accepted: BTreeSet<String> = record
                        .gold
                        .analyses
                        .iter()
                        .map(|analysis| to_atomic(&analysis.atomic))
                        .collect();
                    let ranked = rank_with(
                        &reconstruct_sep(&record.sep.normalized).candidates,
                        inventory,
                        selection,
                    );
                    RecordEvaluation {
                        record_id: record.record_id.clone(),
                        ambiguity_classes: record
                            .gold
                            .ambiguity_classes
                            .iter()
                            .map(|class| class.as_str().to_owned())
                            .collect(),
                        accepted_analyses: accepted.into_iter().collect(),
                        candidates: ranked
                            .into_iter()
                            .map(|candidate| candidate.atomic)
                            .collect(),
                    }
                })
                .collect();
            AblationResult {
                name: name.to_owned(),
                metrics: summarize(&ranked_examples, top_k),
            }
        })
        .collect();

    RankingComparisonReport {
        lexical_entry_count: inventory.len(),
        total_record_count: corpus.records.len(),
        excluded_record_count: excluded_record_ids.len(),
        excluded_record_ids,
        top_k,
        selected_ranker,
        baseline_metrics: metrics_for_selection(
            corpus,
            inventory,
            top_k,
            RankerSelection::Baseline,
        ),
        lexical_prior_metrics: metrics_for_selection(
            corpus,
            inventory,
            top_k,
            RankerSelection::LexicalPrior,
        ),
        ablations,
        side_by_side,
    }
}

fn metrics_for_selection(
    corpus: &ValidatedCorpus,
    inventory: &LexicalInventory,
    top_k: usize,
    selection: RankerSelection,
) -> MetricSummary {
    let records: Vec<RecordEvaluation> = corpus
        .records
        .iter()
        .filter(|record| record.gold.status != GoldStatus::InsufficientEvidence)
        .map(|record| {
            let accepted: BTreeSet<String> = record
                .gold
                .analyses
                .iter()
                .map(|analysis| to_atomic(&analysis.atomic))
                .collect();
            let ranked = rank_with(
                &reconstruct_sep(&record.sep.normalized).candidates,
                inventory,
                selection,
            );
            RecordEvaluation {
                record_id: record.record_id.clone(),
                ambiguity_classes: record
                    .gold
                    .ambiguity_classes
                    .iter()
                    .map(|class| class.as_str().to_owned())
                    .collect(),
                accepted_analyses: accepted.into_iter().collect(),
                candidates: ranked
                    .into_iter()
                    .map(|candidate| candidate.atomic)
                    .collect(),
            }
        })
        .collect();
    summarize(&records, top_k)
}

fn ranks(candidates: &[RankedCandidate]) -> BTreeMap<String, &RankedCandidate> {
    candidates
        .iter()
        .map(|candidate| (candidate.atomic.clone(), candidate))
        .collect()
}

impl RankingComparisonReport {
    pub fn to_markdown(&self) -> String {
        let mut output = format!(
            "# Lexical-prior ranking experiment\n\n\
             - Lexicon Atomic entries: {}\n\
             - Records: {} total, {} excluded\n\
             - Configured top-k: {}\n\
             - Selected ranker: `{:?}`\n\n\
              ## Baseline and lexical-prior results\n\n\
              | Ranker | Top-1 | Top-3 | Top-k | Candidate recall (macro) |\n|---|---:|---:|---:|---:|\n\
              | Baseline (`Candidate::score()`) | {} | {} | {} | {} |\n\
              | Lexical prior (combined) | {} | {} | {} | {} |\n\n\
              ## Ablation metrics\n\n\
             | Ranker | Top-1 | Top-3 | Top-k | Candidate recall (macro) |\n|---|---:|---:|---:|---:|\n",
            self.lexical_entry_count,
            self.total_record_count,
            self.excluded_record_count,
            self.top_k,
            self.selected_ranker,
            percentage(self.baseline_metrics.top_1_accuracy),
            percentage(self.baseline_metrics.top_3_accuracy),
            percentage(self.baseline_metrics.top_k_accuracy),
            percentage(self.baseline_metrics.candidate_recall_macro),
            percentage(self.lexical_prior_metrics.top_1_accuracy),
            percentage(self.lexical_prior_metrics.top_3_accuracy),
            percentage(self.lexical_prior_metrics.top_k_accuracy),
            percentage(self.lexical_prior_metrics.candidate_recall_macro),
        );
        for ablation in &self.ablations {
            output.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                ablation.name,
                percentage(ablation.metrics.top_1_accuracy),
                percentage(ablation.metrics.top_3_accuracy),
                percentage(ablation.metrics.top_k_accuracy),
                percentage(ablation.metrics.candidate_recall_macro),
            ));
        }
        output.push_str(
            "\n## Candidate ranks\n\n\
             The heuristic score is the existing `Candidate::score()`. Lexical score is 2 for an exact Atomic lexicon match plus 1 for a normalized match. Combined score is lexical score × 100 + heuristic score. Ties preserve baseline order for combined rankings and use Atomic lexical order for lexical-only rankings.\n\n\
             | Record | Candidate | Accepted | Attested | Heuristic | Lexical | Baseline rank | Lexical-prior rank | Heuristic-only rank | Lexical-only rank | Combined rank | Selected rank |\n|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n",
        );
        for record in &self.side_by_side {
            for candidate in &record.candidates {
                output.push_str(&format!(
                    "| `{}` | `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                    record.record_id,
                    escape(&candidate.atomic),
                    candidate.accepted,
                    candidate.lexical_attestation,
                    candidate.baseline_score,
                    candidate.lexical_score,
                    candidate.baseline_rank,
                    candidate.lexical_prior_rank,
                    candidate.heuristic_only_rank,
                    candidate.lexical_only_rank,
                    candidate.combined_rank,
                    candidate.final_rank,
                ));
            }
        }
        output.push_str("\n## Excluded records\n\n");
        if self.excluded_record_ids.is_empty() {
            output.push_str("None.\n");
        } else {
            for id in &self.excluded_record_ids {
                output.push_str(&format!("- `{}`\n", escape(id)));
            }
        }
        output
    }
}

fn percentage(value: Option<f64>) -> String {
    value
        .map(|value| format!("{:.2}%", value * 100.0))
        .unwrap_or_else(|| "n/a".to_owned())
}

fn escape(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::{
        BaselineRanker, LexicalInventory, LexicalPriorRanker, Ranker, RankerSelection,
        compare_corpus_rankers, compare_corpus_rankers_with_selection, rank_with,
    };
    use crate::evaluation::validate_corpus_dir;
    use crate::reconstruction::{Candidate, Derivation};
    use std::path::Path;

    fn candidate(atomic: &str, derivations: Vec<Derivation>) -> Candidate {
        Candidate {
            atomic: atomic.to_owned(),
            ack: atomic.to_owned(),
            derivations,
        }
    }

    #[test]
    fn baseline_preserves_score_order_and_lexical_ranker_uses_attestation() {
        let candidates = vec![
            candidate("unattested", vec![Derivation::Unchanged]),
            candidate("attested", vec![Derivation::FinalNRecovery]),
        ];
        let inventory = LexicalInventory::from_atomic_forms(vec!["attested".to_owned()]);

        let baseline = BaselineRanker.rank(&candidates, &inventory);
        let lexical = LexicalPriorRanker.rank(&candidates, &inventory);
        assert_eq!(baseline[0].atomic, "unattested");
        assert_eq!(baseline[0].final_rank, 1);
        assert_eq!(lexical[0].atomic, "attested");
        assert!(lexical[0].features.lexical_attestation);
        assert_eq!(lexical[0].features.derivation_count, 1);
    }

    #[test]
    fn exposes_three_ablation_orders_with_deterministic_ties() {
        let candidates = vec![
            candidate("zeta", vec![Derivation::Unchanged]),
            candidate("alpha", vec![Derivation::Unchanged]),
        ];
        let inventory = LexicalInventory::default();

        let baseline = rank_with(&candidates, &inventory, RankerSelection::Baseline);
        let lexical_only = rank_with(&candidates, &inventory, RankerSelection::LexicalOnly);
        let combined = rank_with(&candidates, &inventory, RankerSelection::Combined);
        assert_eq!(baseline[0].atomic, "zeta");
        assert_eq!(combined[0].atomic, "zeta");
        assert_eq!(lexical_only[0].atomic, "alpha");
    }

    #[test]
    fn compares_ablations_on_synthetic_corpus_fixture() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/evaluation")
            .join("valid_corpus");
        let corpus = validate_corpus_dir(fixture).expect("fixture should validate");
        let inventory =
            LexicalInventory::from_atomic_forms(vec!["nosih".to_owned(), "nosin".to_owned()]);
        let report = compare_corpus_rankers(&corpus, &inventory, 3);

        assert_eq!(report.total_record_count, 1);
        assert_eq!(report.excluded_record_count, 0);
        assert_eq!(report.ablations.len(), 3);
        assert_eq!(report.ablations[0].name, "heuristic_only");
        assert_eq!(report.ablations[1].name, "lexical_prior_only");
        assert_eq!(report.side_by_side.len(), 1);
        assert!(
            report.side_by_side[0]
                .candidates
                .iter()
                .all(|candidate| candidate.baseline_rank > 0 && candidate.combined_rank > 0)
        );
        assert!(report.to_markdown().contains("Ablation metrics"));
        let selected = compare_corpus_rankers_with_selection(
            &corpus,
            &inventory,
            3,
            RankerSelection::Baseline,
        );
        assert_eq!(selected.selected_ranker, RankerSelection::Baseline);
        assert!(
            selected.side_by_side[0]
                .candidates
                .iter()
                .all(|candidate| candidate.final_rank == candidate.baseline_rank)
        );
    }
}
