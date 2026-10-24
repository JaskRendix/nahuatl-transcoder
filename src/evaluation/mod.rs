pub mod benchmark;
pub mod errors;
pub mod metrics;
pub mod report;
pub mod schema;
pub mod validation;

pub use benchmark::{evaluate_corpus, run_timestamp};
pub use errors::ValidationError;
pub use metrics::{MetricSummary, RecordEvaluation, summarize};
pub use report::{BenchmarkReport, CandidateEvaluation, ExcludedRecord, MetricDefinition};
pub use schema::{
    AcceptedAnalysis, AmbiguityClass, CorpusManifest, DialectMetadata, EvaluationRecord,
    GoldStatus, LicenseMetadata, ProvenanceMetadata, SourceMetadata,
};
pub use validation::{ValidatedCorpus, validate_corpus_dir};
