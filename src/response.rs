use std::collections::HashMap;

use axum::{
    Json,
    response::{IntoResponse, Response},
};
use inflector::string::pluralize::to_plural;
use serde::Serialize;

// Trait for getting the response key
pub trait ResponseKey {
    fn response_key() -> &'static str;
}

// Generic API response wrapper
#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    #[serde(flatten)]
    data: HashMap<String, T>,
}

// Wrapper for list responses
#[derive(Serialize)]
struct ApiListResponse<T: Serialize> {
    #[serde(flatten)]
    data: HashMap<String, Vec<T>>,
}

// Custom response type that will handle the wrapping
pub struct WrappedJson<T>(pub T);

// Implementation to convert our types into responses
impl<T> IntoResponse for WrappedJson<T>
where
    T: Serialize + ResponseKey,
{
    fn into_response(self) -> Response {
        let mut map = HashMap::new();
        map.insert(T::response_key().to_string(), self.0);

        let json = Json(ApiResponse { data: map });
        json.into_response()
    }
}

// Implementation for Vec responses
impl<T> IntoResponse for WrappedJson<Vec<T>>
where
    T: Serialize + ResponseKey,
{
    fn into_response(self) -> Response {
        let mut map = HashMap::new();
        map.insert(to_plural(T::response_key()), self.0);

        let json = Json(ApiListResponse { data: map });
        json.into_response()
    }
}
