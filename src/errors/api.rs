use serde::Serialize;
use ts_rs::TS;

use super::enums::ErrorKind;

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "errors.ts")]
pub struct ValidationFieldError {
    pub field: String,
    pub code: String,
    pub message: String,
    pub params: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "errors.ts")]
pub struct SerializableValidationErrors {
    pub errors: Vec<ValidationFieldError>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "errors.ts")]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
    pub kind: ErrorKind,
    pub validation_errors: Option<SerializableValidationErrors>,
}
