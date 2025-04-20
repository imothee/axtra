use serde::{Serialize, Serializer};
use ts_rs::TS;
use validator::ValidationErrors;

#[derive(Clone, Debug)]
pub enum ErrorFormat {
    Html,
    Json,
}

#[derive(Clone, Debug, TS)]
#[ts(export, export_to = "errors.ts", type = "string")]
pub enum ErrorKind {
    Authentication,
    Authorization { resource: String, action: String },
    BadRequest { detail: String },
    NotFound { resource: String },
    Exception { detail: String },
    Validation { errors: ValidationErrors },
}

impl Serialize for ErrorKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let variant_name = match self {
            ErrorKind::Authentication => "Authentication",
            ErrorKind::Authorization { .. } => "Authorization",
            ErrorKind::BadRequest { .. } => "BadRequest",
            ErrorKind::NotFound { .. } => "NotFound",
            ErrorKind::Exception { .. } => "Exception",
            ErrorKind::Validation { .. } => "Validation",
        };
        serializer.serialize_str(variant_name)
    }
}
