use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde_json::Value;

use super::errors::ValidationError;
use super::schema::{
    CorpusManifest, EvaluationRecord, GoldStatus, VALID_AMBIGUITY_CLASSES,
    VALID_REDISTRIBUTION_VALUES, VALID_SPLITS,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedCorpus {
    pub manifest: CorpusManifest,
    pub records: Vec<EvaluationRecord>,
    pub directory: PathBuf,
}

pub fn validate_corpus_dir(
    directory: impl AsRef<Path>,
) -> Result<ValidatedCorpus, Vec<ValidationError>> {
    let directory = directory.as_ref();
    let manifest_path = directory.join("manifest.json");
    let records_path = directory.join("records.jsonl");
    let mut errors = Vec::new();

    let Some(manifest_value) = read_json_file(&manifest_path, "<manifest>", &mut errors) else {
        return Err(errors);
    };

    let required_manifest = [
        "schema_version",
        "corpus_id",
        "version",
        "description",
        "normalization_version",
        "sources",
        "licenses",
        "ambiguity_classes",
        "splits",
    ];
    check_required_fields(
        &manifest_value,
        "<manifest>",
        "",
        &required_manifest,
        &mut errors,
    );
    check_manifest_nested_required_fields(&manifest_value, &mut errors);
    if has_errors_for(&errors, "<manifest>") {
        return Err(errors);
    }

    let Some(manifest) =
        deserialize_value::<CorpusManifest>(manifest_value, "<manifest>", &mut errors)
    else {
        return Err(errors);
    };

    validate_manifest(&manifest, &mut errors);
    if !errors.is_empty() {
        return Err(errors);
    }

    let bytes = match fs::read(&records_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            errors.push(ValidationError::new(
                "<records>",
                "$",
                format!("could not read {}: {error}", records_path.display()),
            ));
            return Err(errors);
        }
    };
    let contents = match String::from_utf8(bytes) {
        Ok(contents) => contents,
        Err(error) => {
            let line_number = error.as_bytes()[..error.utf8_error().valid_up_to()]
                .iter()
                .filter(|&&byte| byte == b'\n')
                .count()
                + 1;
            errors.push(ValidationError::new(
                format!("<line:{line_number}>"),
                "$utf8",
                format!("invalid UTF-8 in {}", records_path.display()),
            ));
            return Err(errors);
        }
    };

    let mut records = Vec::new();
    let mut record_ids = HashSet::new();
    for (line_index, line) in contents.lines().enumerate() {
        let line_number = line_index + 1;
        if line.trim().is_empty() {
            continue;
        }

        let line_record_id = format!("<line:{line_number}>");
        let value: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(error) => {
                errors.push(ValidationError::new(
                    &line_record_id,
                    "$",
                    format!("invalid JSON on line {line_number}: {error}"),
                ));
                continue;
            }
        };
        let record_id = value
            .get("record_id")
            .and_then(Value::as_str)
            .filter(|id| !id.trim().is_empty())
            .unwrap_or(&line_record_id)
            .to_owned();
        if !record_ids.insert(record_id.clone()) {
            errors.push(ValidationError::new(
                &record_id,
                "record_id",
                "duplicate record ID",
            ));
        }

        let required_record = [
            "record_id",
            "sep",
            "gold",
            "dialect",
            "source",
            "provenance",
            "license",
        ];
        let start_errors = errors.len();
        check_required_fields(&value, &record_id, "", &required_record, &mut errors);
        check_nested_required_fields(&value, &record_id, &mut errors);
        check_record_enum_values(&value, &record_id, &mut errors);
        if errors.len() != start_errors {
            continue;
        }

        let Some(record) = deserialize_value::<EvaluationRecord>(value, &record_id, &mut errors)
        else {
            continue;
        };

        validate_record(&record, &manifest, &mut errors);
        records.push(record);
    }

    if records.is_empty() && errors.is_empty() {
        errors.push(ValidationError::new(
            "<records>",
            "$",
            "records.jsonl must contain at least one nonblank record",
        ));
    }

    if errors.is_empty() {
        Ok(ValidatedCorpus {
            manifest,
            records,
            directory: directory.to_path_buf(),
        })
    } else {
        Err(errors)
    }
}

fn read_json_file(
    path: &Path,
    record_id: &str,
    errors: &mut Vec<ValidationError>,
) -> Option<Value> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => {
            errors.push(ValidationError::new(
                record_id,
                "$",
                format!("could not read {}: {error}", path.display()),
            ));
            return None;
        }
    };

    match serde_json::from_str(&contents) {
        Ok(value) => Some(value),
        Err(error) => {
            errors.push(ValidationError::new(
                record_id,
                "$",
                format!("invalid JSON in {}: {error}", path.display()),
            ));
            None
        }
    }
}

fn deserialize_value<T: DeserializeOwned>(
    value: Value,
    record_id: &str,
    errors: &mut Vec<ValidationError>,
) -> Option<T> {
    match serde_json::from_value(value) {
        Ok(value) => Some(value),
        Err(error) => {
            errors.push(ValidationError::new(
                record_id,
                "$",
                format!("invalid document structure: {error}"),
            ));
            None
        }
    }
}

fn has_errors_for(errors: &[ValidationError], record_id: &str) -> bool {
    errors.iter().any(|error| error.record_id == record_id)
}

fn check_required_fields(
    value: &Value,
    record_id: &str,
    parent_path: &str,
    required: &[&str],
    errors: &mut Vec<ValidationError>,
) {
    let Some(object) = value.as_object() else {
        errors.push(ValidationError::new(
            record_id,
            if parent_path.is_empty() {
                "$"
            } else {
                parent_path
            },
            "expected a JSON object",
        ));
        return;
    };

    for field in required {
        if !object.contains_key(*field) {
            let path = if parent_path.is_empty() {
                (*field).to_owned()
            } else {
                format!("{parent_path}.{field}")
            };
            errors.push(ValidationError::new(
                record_id,
                path,
                "required field is missing",
            ));
        }
    }
}

fn check_nested_required_fields(value: &Value, record_id: &str, errors: &mut Vec<ValidationError>) {
    for (field, required) in [
        ("sep", &["observed", "normalized", "normalization"][..]),
        ("gold", &["status", "ambiguity_classes", "analyses"][..]),
        ("dialect", &["label", "variety", "certainty"][..]),
        (
            "source",
            &["source_id", "citation", "location", "url_or_identifier"][..],
        ),
        (
            "provenance",
            &[
                "transcription",
                "transcriber",
                "annotation_ids",
                "adjudication_id",
                "source_group",
                "split",
            ][..],
        ),
        ("license", &["corpus_license_id", "redistribution"][..]),
    ] {
        if let Some(nested) = value.get(field) {
            check_required_fields(nested, record_id, field, required, errors);
        }
    }

    if let Some(analyses) = value
        .get("gold")
        .and_then(|gold| gold.get("analyses"))
        .and_then(Value::as_array)
    {
        for (index, analysis) in analyses.iter().enumerate() {
            check_required_fields(
                analysis,
                record_id,
                &format!("gold.analyses[{index}]"),
                &["analysis_id", "ack", "atomic", "evidence"],
                errors,
            );
        }
    }
}

fn check_manifest_nested_required_fields(value: &Value, errors: &mut Vec<ValidationError>) {
    if let Some(sources) = value.get("sources").and_then(Value::as_array) {
        for (index, source) in sources.iter().enumerate() {
            check_required_fields(
                source,
                "<manifest>",
                &format!("sources[{index}]"),
                &["source_id", "citation", "url_or_identifier", "license_id"],
                errors,
            );
        }
    }

    if let Some(licenses) = value.get("licenses").and_then(Value::as_array) {
        for (index, license) in licenses.iter().enumerate() {
            check_required_fields(
                license,
                "<manifest>",
                &format!("licenses[{index}]"),
                &[
                    "corpus_license_id",
                    "license_identifier",
                    "copyright_holder",
                    "permissions",
                    "redistributable_fields",
                ],
                errors,
            );
        }
    }
}

fn check_record_enum_values(value: &Value, record_id: &str, errors: &mut Vec<ValidationError>) {
    let Some(gold) = value.get("gold") else {
        return;
    };

    if let Some(status) = gold.get("status") {
        let valid = [
            "single_analysis",
            "multiple_analyses",
            "underspecified",
            "insufficient_evidence",
        ];
        check_enum_value(status, &valid, record_id, "gold.status", errors);
    }

    if let Some(classes) = gold.get("ambiguity_classes").and_then(Value::as_array) {
        for (index, class) in classes.iter().enumerate() {
            check_enum_value(
                class,
                VALID_AMBIGUITY_CLASSES,
                record_id,
                &format!("gold.ambiguity_classes[{index}]"),
                errors,
            );
        }
    }

    if let Some(split) = value
        .get("provenance")
        .and_then(|provenance| provenance.get("split"))
    {
        check_enum_value(split, VALID_SPLITS, record_id, "provenance.split", errors);
    }

    if let Some(redistribution) = value
        .get("license")
        .and_then(|license| license.get("redistribution"))
    {
        check_enum_value(
            redistribution,
            VALID_REDISTRIBUTION_VALUES,
            record_id,
            "license.redistribution",
            errors,
        );
    }
}

fn check_enum_value(
    value: &Value,
    allowed: &[&str],
    record_id: &str,
    field_path: &str,
    errors: &mut Vec<ValidationError>,
) {
    match value.as_str() {
        Some(value) if allowed.contains(&value) => {}
        Some(value) => errors.push(ValidationError::new(
            record_id,
            field_path,
            format!(
                "invalid value {value:?}; expected one of {}",
                allowed.join(", ")
            ),
        )),
        None => errors.push(ValidationError::new(
            record_id,
            field_path,
            "expected a string value",
        )),
    }
}

fn validate_manifest(manifest: &CorpusManifest, errors: &mut Vec<ValidationError>) {
    let id = "<manifest>";
    if manifest.schema_version != super::schema::SCHEMA_VERSION {
        errors.push(ValidationError::new(
            id,
            "schema_version",
            format!(
                "unsupported manifest schema version {:?}; expected {:?}",
                manifest.schema_version,
                super::schema::SCHEMA_VERSION
            ),
        ));
    }

    for (path, value) in [
        ("corpus_id", manifest.corpus_id.as_str()),
        ("version", manifest.version.as_str()),
        ("description", manifest.description.as_str()),
        (
            "normalization_version",
            manifest.normalization_version.as_str(),
        ),
    ] {
        check_nonempty(value, id, path, errors);
    }

    if manifest.sources.is_empty() {
        errors.push(ValidationError::new(
            id,
            "sources",
            "at least one source entry is required",
        ));
    }
    if manifest.licenses.is_empty() {
        errors.push(ValidationError::new(
            id,
            "licenses",
            "at least one license entry is required",
        ));
    }

    let mut source_ids = HashSet::new();
    for (index, source) in manifest.sources.iter().enumerate() {
        let path = format!("sources[{index}]");
        check_nonempty(&source.source_id, id, &format!("{path}.source_id"), errors);
        check_nonempty(&source.citation, id, &format!("{path}.citation"), errors);
        check_nonempty(
            &source.url_or_identifier,
            id,
            &format!("{path}.url_or_identifier"),
            errors,
        );
        check_nonempty(
            source.license_id.as_deref().unwrap_or(""),
            id,
            &format!("{path}.license_id"),
            errors,
        );
        if !source_ids.insert(&source.source_id) {
            errors.push(ValidationError::new(
                id,
                format!("{path}.source_id"),
                "duplicate source ID",
            ));
        }
        if let Some(license_id) = &source.license_id
            && !manifest
                .licenses
                .iter()
                .any(|license| license.corpus_license_id == *license_id)
        {
            errors.push(ValidationError::new(
                id,
                format!("{path}.license_id"),
                format!("unknown license reference {license_id:?}"),
            ));
        }
    }

    let mut license_ids = HashSet::new();
    for (index, license) in manifest.licenses.iter().enumerate() {
        let path = format!("licenses[{index}]");
        check_nonempty(
            &license.corpus_license_id,
            id,
            &format!("{path}.corpus_license_id"),
            errors,
        );
        check_nonempty(
            license.license_identifier.as_deref().unwrap_or(""),
            id,
            &format!("{path}.license_identifier"),
            errors,
        );
        check_nonempty(
            license.copyright_holder.as_deref().unwrap_or(""),
            id,
            &format!("{path}.copyright_holder"),
            errors,
        );
        check_nonempty(
            license.permissions.as_deref().unwrap_or(""),
            id,
            &format!("{path}.permissions"),
            errors,
        );
        if license.redistributable_fields.is_empty() {
            errors.push(ValidationError::new(
                id,
                format!("{path}.redistributable_fields"),
                "at least one redistributable field or restriction must be documented",
            ));
        }
        if !license_ids.insert(&license.corpus_license_id) {
            errors.push(ValidationError::new(
                id,
                format!("{path}.corpus_license_id"),
                "duplicate license ID",
            ));
        }
    }

    let mut classes = HashSet::new();
    for (index, class) in manifest.ambiguity_classes.iter().enumerate() {
        if !VALID_AMBIGUITY_CLASSES.contains(&class.as_str()) {
            errors.push(ValidationError::new(
                id,
                format!("ambiguity_classes[{index}]"),
                format!("unknown ambiguity class {class:?}"),
            ));
        }
        if !classes.insert(class) {
            errors.push(ValidationError::new(
                id,
                format!("ambiguity_classes[{index}]"),
                "duplicate ambiguity class",
            ));
        }
    }

    let mut splits = HashSet::new();
    for (index, split) in manifest.splits.iter().enumerate() {
        if !VALID_SPLITS.contains(&split.as_str()) {
            errors.push(ValidationError::new(
                id,
                format!("splits[{index}]"),
                format!("invalid split name {split:?}"),
            ));
        }
        if !splits.insert(split) {
            errors.push(ValidationError::new(
                id,
                format!("splits[{index}]"),
                "duplicate split name",
            ));
        }
    }
    if manifest.splits.is_empty() {
        errors.push(ValidationError::new(
            id,
            "splits",
            "at least one split is required",
        ));
    }
}

fn validate_record(
    record: &EvaluationRecord,
    manifest: &CorpusManifest,
    errors: &mut Vec<ValidationError>,
) {
    let id = record.record_id.as_str();
    for (path, value) in [
        ("record_id", record.record_id.as_str()),
        ("sep.observed", record.sep.observed.as_str()),
        ("sep.normalized", record.sep.normalized.as_str()),
        ("sep.normalization", record.sep.normalization.as_str()),
        ("dialect.label", record.dialect.label.as_str()),
        ("dialect.variety", record.dialect.variety.as_str()),
        ("dialect.certainty", record.dialect.certainty.as_str()),
        ("source.source_id", record.source.source_id.as_str()),
        ("source.citation", record.source.citation.as_str()),
        (
            "source.url_or_identifier",
            record.source.url_or_identifier.as_str(),
        ),
        (
            "provenance.transcription",
            record.provenance.transcription.as_str(),
        ),
        (
            "provenance.transcriber",
            record.provenance.transcriber.as_str(),
        ),
        (
            "provenance.adjudication_id",
            record.provenance.adjudication_id.as_str(),
        ),
        (
            "provenance.source_group",
            record.provenance.source_group.as_str(),
        ),
        (
            "license.corpus_license_id",
            record.license.corpus_license_id.as_str(),
        ),
    ] {
        check_nonempty(value, id, path, errors);
    }
    if record
        .source
        .location
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        errors.push(ValidationError::new(
            id,
            "source.location",
            "required location must be a non-empty string",
        ));
    }
    if record.provenance.annotation_ids.is_empty()
        || record
            .provenance
            .annotation_ids
            .iter()
            .any(|value| value.trim().is_empty())
    {
        errors.push(ValidationError::new(
            id,
            "provenance.annotation_ids",
            "at least one non-empty annotation ID is required",
        ));
    }
    check_nonempty(
        record.license.redistribution.as_deref().unwrap_or(""),
        id,
        "license.redistribution",
        errors,
    );

    if !manifest
        .sources
        .iter()
        .any(|source| source.source_id == record.source.source_id)
    {
        errors.push(ValidationError::new(
            id,
            "source.source_id",
            format!("unknown source reference {:?}", record.source.source_id),
        ));
    }
    let license_id = &record.license.corpus_license_id;
    if !manifest
        .licenses
        .iter()
        .any(|license| license.corpus_license_id == *license_id)
    {
        errors.push(ValidationError::new(
            id,
            "license.corpus_license_id",
            format!("unknown license reference {license_id:?}"),
        ));
    }
    if let Some(source_license) = record.source.license_id.as_deref()
        && source_license != license_id
    {
        errors.push(ValidationError::new(
            id,
            "source.license_id",
            "source license reference does not match record license",
        ));
    }

    let split = &record.provenance.split;
    if !VALID_SPLITS.contains(&split.as_str()) {
        errors.push(ValidationError::new(
            id,
            "provenance.split",
            format!("invalid split name {split:?}"),
        ));
    } else if !manifest.splits.contains(split) {
        errors.push(ValidationError::new(
            id,
            "provenance.split",
            format!("split {split:?} is not declared in the manifest"),
        ));
    }

    let classes = &record.gold.ambiguity_classes;
    for (index, class) in classes.iter().enumerate() {
        if !manifest
            .ambiguity_classes
            .iter()
            .any(|declared| declared == class.as_str())
        {
            errors.push(ValidationError::new(
                id,
                format!("gold.ambiguity_classes[{index}]"),
                format!(
                    "ambiguity class {:?} is not declared in the manifest",
                    class.as_str()
                ),
            ));
        }
    }
    if record.gold.status == GoldStatus::MultipleAnalyses && classes.is_empty() {
        errors.push(ValidationError::new(
            id,
            "gold.ambiguity_classes",
            "multiple_analyses requires at least one ambiguity class",
        ));
    }

    match record.gold.status {
        GoldStatus::SingleAnalysis if record.gold.analyses.len() != 1 => {
            errors.push(ValidationError::new(
                id,
                "gold.analyses",
                "single_analysis requires exactly one accepted analysis",
            ));
        }
        GoldStatus::MultipleAnalyses if record.gold.analyses.len() < 2 => {
            errors.push(ValidationError::new(
                id,
                "gold.analyses",
                "multiple_analyses requires at least two accepted analyses",
            ));
        }
        GoldStatus::Underspecified if record.gold.analyses.is_empty() => {
            errors.push(ValidationError::new(
                id,
                "gold.analyses",
                "underspecified requires at least one accepted analysis",
            ));
        }
        GoldStatus::InsufficientEvidence if !record.gold.analyses.is_empty() => {
            errors.push(ValidationError::new(
                id,
                "gold.analyses",
                "insufficient_evidence requires an empty accepted-analysis list",
            ));
        }
        _ => {}
    }

    let mut analysis_ids = HashSet::new();
    for (index, analysis) in record.gold.analyses.iter().enumerate() {
        let path = format!("gold.analyses[{index}]");
        for (field, value) in [
            ("analysis_id", analysis.analysis_id.as_str()),
            ("ack", analysis.ack.as_str()),
            ("atomic", analysis.atomic.as_str()),
            ("evidence", analysis.evidence.as_str()),
        ] {
            check_nonempty(value, id, &format!("{path}.{field}"), errors);
        }
        if !analysis_ids.insert(&analysis.analysis_id) {
            errors.push(ValidationError::new(
                id,
                format!("{path}.analysis_id"),
                "duplicate analysis ID",
            ));
        }
    }
}

fn check_nonempty(
    value: &str,
    record_id: &str,
    field_path: &str,
    errors: &mut Vec<ValidationError>,
) {
    if value.trim().is_empty() {
        errors.push(ValidationError::new(
            record_id,
            field_path,
            "required value must be a non-empty string",
        ));
    }
}
