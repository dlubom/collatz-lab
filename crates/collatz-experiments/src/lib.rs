#![forbid(unsafe_code)]

//! Reproducible Collatz input, control, configuration, and result contracts.

pub mod catalog;
pub mod number;

pub use catalog::{Catalog, CatalogError};
pub use number::{
    NUMBER_DEFINITION_SCHEMA_VERSION, NumberConstruction, NumberDefinition, NumberValidationError,
    Provenance, ValidatedNumber, ValueOrigin,
};
