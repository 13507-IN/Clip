use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlEntry {
    pub id: i64,
    pub short_code: String,
    pub original: String,
    pub created_at: String,
    pub clicks: i64,
}

#[derive(Debug, Deserialize)]
pub struct ShortenRequest {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ShortenResponse {
    pub short_code: String,
    pub short_url: String,
    pub original: String,
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub short_code: String,
    pub original: String,
    pub clicks: i64,
    pub created_at: String,
    pub redirect_uri: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
