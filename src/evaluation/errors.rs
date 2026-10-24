use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub record_id: String,
    pub field_path: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(
        record_id: impl Into<String>,
        field_path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            record_id: record_id.into(),
            field_path: field_path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}: {}",
            self.record_id, self.field_path, self.message
        )
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::ValidationError;

    #[test]
    fn display_includes_record_path_and_message() {
        let error = ValidationError::new("record-1", "gold.status", "unknown status");
        assert_eq!(error.to_string(), "record-1: gold.status: unknown status");
    }
}
