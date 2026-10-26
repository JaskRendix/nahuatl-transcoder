//! Candidate ranking and lexical priors.
//!
//! This module ranks reconstructed morphophonemic candidates
//! using heuristic scores, lexical attestation data, or
//! combinations of both.

use std::collections::HashSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::atomic::to_atomic;
use crate::convert_all;
use crate::reconstruction::Candidate;

/// Features extracted from a reconstructed candidate that
/// may be used during ranking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateFeatures {
    pub lexical_attestation: bool,
    pub exact_lexicon_match: bool,
    pub normalized_lexicon_match: bool,
    pub derivation_count: usize,
    pub heuristic_score: i32,
}

/// A candidate together with ranking features and scores.
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

/// Ranking strategy used when ordering candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RankerSelection {
    Baseline,
    LexicalPrior,
    HeuristicOnly,
    LexicalOnly,
    Combined,
}

/// Lexical resource used to provide attestation-based
/// ranking signals.
#[derive(Debug, Clone, Default)]
pub struct LexicalInventory {
    exact_atomic: HashSet<String>,
    normalized_atomic: HashSet<String>,
}

impl LexicalInventory {
    /// Builds a lexical inventory from ATOMIC forms.
    pub fn from_atomic_forms(forms: impl IntoIterator<Item = String>) -> Self {
        let exact_atomic: HashSet<String> = forms.into_iter().collect();
        let normalized_atomic = exact_atomic.iter().map(|form| to_atomic(form)).collect();
        Self {
            exact_atomic,
            normalized_atomic,
        }
    }

    /// Builds a lexical inventory from dictionary headwords,
    /// normalizing them to ATOMIC orthography.
    pub fn from_headwords(headwords: impl IntoIterator<Item = String>) -> Self {
        let exact_atomic: Vec<String> = headwords
            .into_iter()
            .map(|word| convert_all(&word).atom)
            .collect();
        Self::from_atomic_forms(exact_atomic)
    }

    /// Loads a lexical inventory from a G2 lexicon file.
    pub fn load_g2(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        Self::from_g2_bytes(&std::fs::read(path)?)
    }

    /// Loads a lexical inventory from G2-formatted content.
    pub fn from_g2_bytes(contents: &[u8]) -> Result<Self, std::io::Error> {
        let contents = std::str::from_utf8(contents)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        let mut headwords = Vec::new();
        for line in contents.lines() {
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

/// Orders candidates according to a ranking strategy.
pub trait Ranker {
    fn rank(&self, candidates: &[Candidate], inventory: &LexicalInventory) -> Vec<RankedCandidate>;
}

/// Ranker that uses heuristic reconstruction scores only.
#[derive(Debug, Clone, Copy, Default)]
pub struct BaselineRanker;

impl Ranker for BaselineRanker {
    fn rank(&self, candidates: &[Candidate], inventory: &LexicalInventory) -> Vec<RankedCandidate> {
        rank_with(candidates, inventory, RankerSelection::Baseline)
    }
}

/// Ranker that combines lexical attestation with heuristic
/// reconstruction scores.
#[derive(Debug, Clone, Copy, Default)]
pub struct LexicalPriorRanker;

impl Ranker for LexicalPriorRanker {
    fn rank(&self, candidates: &[Candidate], inventory: &LexicalInventory) -> Vec<RankedCandidate> {
        rank_with(candidates, inventory, RankerSelection::Combined)
    }
}

/// Ranks candidates using the selected strategy.
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

/// Combines lexical and heuristic scores into a single value.
fn combined_score(lexical_score: i32, heuristic_score: i32) -> i32 {
    lexical_score * 100 + heuristic_score
}

#[cfg(test)]
mod tests {
    use super::{
        BaselineRanker, LexicalInventory, LexicalPriorRanker, Ranker, RankerSelection, rank_with,
    };
    use crate::reconstruction::{Candidate, Derivation};

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
}
