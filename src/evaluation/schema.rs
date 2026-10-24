use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: &str = "sep-reconstruction-eval/1";

pub const VALID_AMBIGUITY_CLASSES: &[&str] = &[
    "final_n_vs_h",
    "final_w_vs_n",
    "coda_w",
    "coda_k_lenition",
    "kw_coda",
    "ll_hl",
    "gemination",
    "orthographic_normalization",
    "morphological",
    "other",
];

pub const VALID_SPLITS: &[&str] = &["development", "test", "challenge"];

pub const VALID_REDISTRIBUTION_VALUES: &[&str] = &[
    "source_text_and_annotations",
    "metadata_and_annotation_only",
    "derived_annotations_only",
    "reference_only",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorpusManifest {
    pub schema_version: String,
    pub corpus_id: String,
    pub version: String,
    pub description: String,
    pub normalization_version: String,
    pub sources: Vec<SourceMetadata>,
    pub licenses: Vec<LicenseMetadata>,
    pub ambiguity_classes: Vec<String>,
    pub splits: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationRecord {
    pub record_id: String,
    pub sep: SepMetadata,
    pub gold: GoldMetadata,
    pub dialect: DialectMetadata,
    pub source: SourceMetadata,
    pub provenance: ProvenanceMetadata,
    pub license: LicenseMetadata,
    #[serde(default)]
    pub notes: Option<EvaluationNotes>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SepMetadata {
    pub observed: String,
    pub normalized: String,
    pub normalization: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoldMetadata {
    pub status: GoldStatus,
    pub ambiguity_classes: Vec<AmbiguityClass>,
    pub analyses: Vec<AcceptedAnalysis>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoldStatus {
    SingleAnalysis,
    MultipleAnalyses,
    Underspecified,
    InsufficientEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmbiguityClass {
    #[serde(rename = "final_n_vs_h")]
    FinalNvsH,
    #[serde(rename = "final_w_vs_n")]
    FinalWvsN,
    #[serde(rename = "coda_w")]
    CodaW,
    #[serde(rename = "coda_k_lenition")]
    CodaKLenition,
    #[serde(rename = "kw_coda")]
    KwCoda,
    #[serde(rename = "ll_hl")]
    LlHl,
    #[serde(rename = "gemination")]
    Gemination,
    #[serde(rename = "orthographic_normalization")]
    OrthographicNormalization,
    #[serde(rename = "morphological")]
    Morphological,
    #[serde(rename = "other")]
    Other,
}

impl AmbiguityClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FinalNvsH => "final_n_vs_h",
            Self::FinalWvsN => "final_w_vs_n",
            Self::CodaW => "coda_w",
            Self::CodaKLenition => "coda_k_lenition",
            Self::KwCoda => "kw_coda",
            Self::LlHl => "ll_hl",
            Self::Gemination => "gemination",
            Self::OrthographicNormalization => "orthographic_normalization",
            Self::Morphological => "morphological",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptedAnalysis {
    pub analysis_id: String,
    pub ack: String,
    pub atomic: String,
    #[serde(default)]
    pub lemma_id: Option<String>,
    #[serde(default)]
    pub morphology: Option<String>,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DialectMetadata {
    pub label: String,
    pub variety: String,
    pub certainty: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceMetadata {
    pub source_id: String,
    pub citation: String,
    #[serde(default)]
    pub location: Option<String>,
    pub url_or_identifier: String,
    #[serde(default)]
    pub license_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceMetadata {
    pub transcription: String,
    pub transcriber: String,
    pub annotation_ids: Vec<String>,
    pub adjudication_id: String,
    pub source_group: String,
    pub split: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseMetadata {
    pub corpus_license_id: String,
    #[serde(default)]
    pub redistribution: Option<String>,
    #[serde(default)]
    pub license_identifier: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub copyright_holder: Option<String>,
    #[serde(default)]
    pub permissions: Option<String>,
    #[serde(default)]
    pub redistributable_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationNotes {
    #[serde(default)]
    pub morphological: Option<String>,
    #[serde(default)]
    pub orthographic: Option<String>,
}
