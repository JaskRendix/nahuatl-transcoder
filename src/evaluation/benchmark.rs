use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::apply_m2f;
use crate::atomic::to_atomic;
use crate::evaluation::metrics::{MetricSummary, RecordEvaluation, summarize};
use crate::evaluation::report::{
    BenchmarkMetadata, BenchmarkReport, CandidateEvaluation, ExcludedRecord, MetricDefinition,
    RecordResult,
};
use crate::evaluation::schema::GoldStatus;
use crate::evaluation::validation::ValidatedCorpus;
use crate::phonetics::degeminate;
use crate::reconstruction::{Candidate, Derivation, reconstruct_sep};

#[derive(Debug, Clone)]
pub struct PreparedRecord {
    pub record_id: String,
    pub status: GoldStatus,
    pub normalized_sep: String,
    pub ambiguity_classes: Vec<String>,
    pub accepted_atomic: BTreeSet<String>,
    pub candidates: Vec<Candidate>,
}

#[derive(Debug, Clone)]
pub struct PreparedBenchmark {
    pub metadata: BenchmarkMetadata,
    pub total_record_count: usize,
    pub excluded_records: Vec<ExcludedRecord>,
    pub records: Vec<PreparedRecord>,
}

pub fn prepare_benchmark(
    corpus: &ValidatedCorpus,
    lexical_resource: Option<(&Path, &str)>,
) -> PreparedBenchmark {
    let metadata = benchmark_metadata(corpus, lexical_resource);
    let mut records = Vec::new();
    let mut excluded_records = Vec::new();
    for record in &corpus.records {
        if record.gold.status == GoldStatus::InsufficientEvidence {
            excluded_records.push(ExcludedRecord {
                record_id: record.record_id.clone(),
                reason: "insufficient_evidence: no adjudicated Atomic analysis".to_owned(),
            });
            continue;
        }

        records.push(PreparedRecord {
            record_id: record.record_id.clone(),
            status: record.gold.status,
            normalized_sep: record.sep.normalized.clone(),
            ambiguity_classes: record
                .gold
                .ambiguity_classes
                .iter()
                .map(|class| class.as_str().to_owned())
                .collect(),
            accepted_atomic: record
                .gold
                .analyses
                .iter()
                .map(|analysis| to_atomic(&analysis.atomic))
                .collect(),
            candidates: reconstruct_sep(&record.sep.normalized).candidates,
        });
    }
    PreparedBenchmark {
        metadata,
        total_record_count: corpus.records.len(),
        excluded_records,
        records,
    }
}

impl PreparedBenchmark {
    pub fn evaluations<F>(&self, mut order: F) -> Vec<RecordEvaluation>
    where
        F: FnMut(&PreparedRecord) -> Vec<String>,
    {
        self.records
            .iter()
            .map(|record| RecordEvaluation {
                record_id: record.record_id.clone(),
                ambiguity_classes: record.ambiguity_classes.clone(),
                accepted_analyses: record.accepted_atomic.iter().cloned().collect(),
                candidates: order(record),
            })
            .collect()
    }

    pub fn metrics<F>(&self, top_k: usize, order: F) -> MetricSummary
    where
        F: FnMut(&PreparedRecord) -> Vec<String>,
    {
        summarize(&self.evaluations(order), top_k)
    }
}

pub fn evaluate_corpus(corpus: &ValidatedCorpus, top_k: usize) -> BenchmarkReport {
    let prepared = prepare_benchmark(corpus, None);
    let evaluations = prepared.evaluations(|record| {
        record
            .candidates
            .iter()
            .map(|candidate| candidate.atomic.clone())
            .collect()
    });
    let metrics = summarize(&evaluations, top_k);
    let ambiguity_class_metrics = class_metrics(&evaluations, top_k);
    let record_results = prepared
        .records
        .iter()
        .map(|record| record_result(record, &record.normalized_sep))
        .collect();

    BenchmarkReport {
        metadata: prepared.metadata,
        configured_top_k: top_k,
        total_record_count: prepared.total_record_count,
        excluded_record_count: prepared.excluded_records.len(),
        excluded_records: prepared.excluded_records,
        metric_definitions: metric_definitions(top_k),
        metrics,
        ambiguity_class_metrics,
        record_results,
    }
}

fn class_metrics(records: &[RecordEvaluation], top_k: usize) -> BTreeMap<String, MetricSummary> {
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

fn record_result(record: &PreparedRecord, observed: &str) -> RecordResult {
    let normalized_observed = to_atomic(observed);
    let candidate_evaluations = record
        .candidates
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
                matches_accepted_analysis: record
                    .accepted_atomic
                    .contains(&to_atomic(&candidate.atomic)),
            }
        })
        .collect();
    RecordResult {
        record_id: record.record_id.clone(),
        status: status_name(record.status).to_owned(),
        ambiguity_classes: record.ambiguity_classes.clone(),
        accepted_atomic: record.accepted_atomic.iter().cloned().collect(),
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

pub fn benchmark_metadata(
    corpus: &ValidatedCorpus,
    lexical_resource: Option<(&Path, &str)>,
) -> BenchmarkMetadata {
    BenchmarkMetadata {
        corpus_id: corpus.manifest.corpus_id.clone(),
        corpus_version: corpus.manifest.version.clone(),
        schema_version: corpus.manifest.schema_version.clone(),
        normalization_version: corpus.manifest.normalization_version.clone(),
        run_timestamp: run_timestamp(),
        git_commit: git_commit(),
        git_working_tree_clean: git_working_tree_clean(),
        repository_version: env!("CARGO_PKG_VERSION").to_owned(),
        lexical_resource_path: lexical_resource.map(|(path, _)| path.display().to_string()),
        lexical_resource_checksum: lexical_resource.map(|(_, checksum)| checksum.to_owned()),
    }
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let contents = fs::read(path)
        .map_err(|error| format!("could not read {} for checksum: {error}", path.display()))?;
    Ok(format!("sha256:{}", sha256_hex(&contents)))
}

pub fn sha256_bytes(input: &[u8]) -> String {
    format!("sha256:{}", sha256_hex(input))
}

pub fn sha256_hex(input: &[u8]) -> String {
    const INITIAL: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut hash = INITIAL;
    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut padded = input.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    let (chunks, _) = padded.as_chunks::<64>();
    for chunk in chunks {
        let mut words = [0u32; 64];
        for (index, bytes) in chunk.as_chunks::<4>().0.iter().enumerate() {
            words[index] = u32::from_be_bytes(*bytes);
        }
        for index in 16..64 {
            let x = words[index - 15];
            let y = words[index - 2];
            let s0 = x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3);
            let s1 = y.rotate_right(17) ^ y.rotate_right(19) ^ (y >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = hash;
        for index in 0..64 {
            let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ (!e & g);
            let t1 = h
                .wrapping_add(sum1)
                .wrapping_add(choice)
                .wrapping_add(K[index])
                .wrapping_add(words[index]);
            let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let t2 = sum0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (value, addition) in hash.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *value = value.wrapping_add(addition);
        }
    }

    hash.iter().map(|value| format!("{value:08x}")).collect()
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
        .current_dir(env!("CARGO_MANIFEST_DIR"))
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
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .ok()?;
    output.status.success().then_some(output.stdout.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{run_timestamp, sha256_file, sha256_hex};

    #[test]
    fn timestamp_uses_utc_rfc3339_shape() {
        let timestamp = run_timestamp();
        assert_eq!(timestamp.len(), 20);
        assert!(timestamp.ends_with('Z'));
        assert_eq!(&timestamp[4..5], "-");
        assert_eq!(&timestamp[10..11], "T");
    }

    #[test]
    fn hashes_file_contents_with_algorithm_prefix() {
        let path = std::env::temp_dir().join(format!("checksum-{}", std::process::id()));
        std::fs::write(&path, b"abc").expect("temporary content should be written");
        let checksum = sha256_file(&path).expect("checksum should be calculated");
        std::fs::remove_file(path).expect("temporary file should be removed");
        assert_eq!(
            checksum,
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sha256_handles_empty_input_and_multiple_blocks() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(&vec![b'a'; 1000]),
            "41edece42d63e8d9bf515a9ba6932e1c20cbc9f5a5d134645adb5db1b9737ea3"
        );
    }
}
