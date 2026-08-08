use core::fmt;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::{NumberDefinition, NumberValidationError, ValidatedNumber};

/// A validated, ordered catalog with unique stable input identifiers.
#[derive(Clone, Debug)]
pub struct Catalog {
    entries: Vec<ValidatedNumber>,
}

impl Catalog {
    pub fn load_jsonl(path: impl AsRef<Path>) -> Result<Self, CatalogError> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|source| CatalogError::Io {
            operation: "open",
            path: path.to_path_buf(),
            source,
        })?;
        let mut definitions = Vec::new();

        for (index, line) in BufReader::new(file).lines().enumerate() {
            let line_number = index + 1;
            let line = line.map_err(|source| CatalogError::Io {
                operation: "read",
                path: path.to_path_buf(),
                source,
            })?;
            if line.trim().is_empty() {
                return Err(CatalogError::BlankLine { line: line_number });
            }
            let definition = serde_json::from_str(&line).map_err(|source| CatalogError::Json {
                line: line_number,
                message: source.to_string(),
            })?;
            definitions.push(definition);
        }

        Self::from_definitions(definitions)
    }

    pub fn from_definitions(definitions: Vec<NumberDefinition>) -> Result<Self, CatalogError> {
        let mut identifiers = HashSet::new();
        let mut entries = Vec::with_capacity(definitions.len());

        for (index, definition) in definitions.into_iter().enumerate() {
            let line = index + 1;
            let validated = ValidatedNumber::validate(definition)
                .map_err(|source| CatalogError::InvalidDefinition { line, source })?;
            let input_id = &validated.definition().input_id;
            if !identifiers.insert(input_id.clone()) {
                return Err(CatalogError::DuplicateInputId {
                    input_id: input_id.clone(),
                });
            }
            entries.push(validated);
        }

        Ok(Self { entries })
    }

    pub fn entries(&self) -> &[ValidatedNumber] {
        &self.entries
    }

    pub fn get(&self, input_id: &str) -> Option<&ValidatedNumber> {
        self.entries
            .iter()
            .find(|entry| entry.definition().input_id == input_id)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// A catalog parsing or definition-validation failure.
#[derive(Debug)]
pub enum CatalogError {
    Io {
        operation: &'static str,
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    Json {
        line: usize,
        message: String,
    },
    BlankLine {
        line: usize,
    },
    InvalidDefinition {
        line: usize,
        source: NumberValidationError,
    },
    DuplicateInputId {
        input_id: String,
    },
}

impl CatalogError {
    pub const fn status_code(&self) -> &'static str {
        "invalid_input"
    }
}

impl fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                operation,
                path,
                source,
            } => write!(formatter, "cannot {operation} {}: {source}", path.display()),
            Self::Json { line, message } => {
                write!(formatter, "invalid JSON on catalog line {line}: {message}")
            }
            Self::BlankLine { line } => write!(formatter, "blank catalog line {line}"),
            Self::InvalidDefinition { line, source } => {
                write!(formatter, "invalid_input on catalog line {line}: {source}")
            }
            Self::DuplicateInputId { input_id } => {
                write!(formatter, "duplicate catalog input_id {input_id}")
            }
        }
    }
}

impl std::error::Error for CatalogError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidDefinition { source, .. } => Some(source),
            Self::Json { .. } | Self::BlankLine { .. } | Self::DuplicateInputId { .. } => None,
        }
    }
}
