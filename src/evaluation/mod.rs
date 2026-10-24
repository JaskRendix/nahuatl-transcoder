pub mod errors;
pub mod schema;
pub mod validation;

pub use errors::ValidationError;
pub use schema::{
    AcceptedAnalysis, AmbiguityClass, CorpusManifest, DialectMetadata, EvaluationRecord,
    GoldStatus, LicenseMetadata, ProvenanceMetadata, SourceMetadata,
};
pub use validation::{ValidatedCorpus, validate_corpus_dir};
