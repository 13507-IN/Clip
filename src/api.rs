use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response, Json},
};

use crate::cache::Cache;
use crate::model::*;
use crate::shortener;
use crate::store::Store;

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<Store>,
    pub cache: Arc<Mutex<Cache>>,
    pub base_url: String,
    pub counter: Arc<AtomicU64>,
}

async fn json_response<T: serde::Serialize>(status: StatusCode, body: T) -> Response {
    (status, Json(body)).into_response()
}

pub async fn shorten(
    State(state): State<AppState>,
    Json(req): Json<ShortenRequest>,
) -> Response {
    let original = req.url.trim().to_string();
    if original.is_empty() {
        return json_response(StatusCode::BAD_REQUEST, ErrorResponse {
            error: "url is required".into(),
        });
    }

    if !original.starts_with("http://") && !original.starts_with("https://") {
        return json_response(StatusCode::BAD_REQUEST, ErrorResponse {
            error: "invalid url".into(),
        });
    }

    // Check for existing URL
    match state.store.get_by_original(&original).await {
        Ok(Some(entry)) => {
            let mut cache = state.cache.lock().unwrap();
            cache.set(entry.short_code.clone(), entry.original.clone());
            let short_url = format!("{}/{}", state.base_url, entry.short_code);
            return json_response(StatusCode::OK, ShortenResponse {
                short_code: entry.short_code,
                short_url,
                original: entry.original,
            });
        }
        Ok(None) => {}
        Err(e) => {
            tracing::error!("store lookup error: {e}");
            return json_response(StatusCode::INTERNAL_SERVER_ERROR, ErrorResponse {
                error: "internal error".into(),
            });
        }
    }

    let mut retries = 0;
    loop {
        let id = state.counter.fetch_add(1, Ordering::Relaxed);
        let short_code = shortener::generate_short_code(id);

        match state.store.insert(&short_code, &original).await {
            Ok(entry) => {
                let mut cache = state.cache.lock().unwrap();
                cache.set(entry.short_code.clone(), entry.original.clone());
                let short_url = format!("{}/{}", state.base_url, entry.short_code);
                return json_response(StatusCode::CREATED, ShortenResponse {
                    short_code: entry.short_code,
                    short_url,
                    original: entry.original,
                });
            }
            Err(e) => {
                if e.to_string().contains("UNIQUE") {
                    retries += 1;
                    if retries > 10 {
                        return json_response(StatusCode::INTERNAL_SERVER_ERROR, ErrorResponse {
                            error: "collision limit exceeded".into(),
                        });
                    }
                    continue;
                }
                tracing::error!("insert error: {e}");
                return json_response(StatusCode::INTERNAL_SERVER_ERROR, ErrorResponse {
                    error: "internal error".into(),
                });
            }
        }
    }
}

pub async fn redirect(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Response {
    if !shortener::is_valid_short_code(&code) {
        return json_response(StatusCode::BAD_REQUEST, ErrorResponse {
            error: "invalid short code".into(),
        });
    }

    // Check cache first
    {
        let mut cache = state.cache.lock().unwrap();
        if let Some(original) = cache.get(&code).map(|s| s.to_string()) {
            let _ = state.store.increment_clicks(&code).await;
            return Redirect::permanent(&original).into_response();
        }
    }

    match state.store.get_by_short_code(&code).await {
        Ok(Some(entry)) => {
            // Populate cache
            {
                let mut cache = state.cache.lock().unwrap();
                cache.set(entry.short_code.clone(), entry.original.clone());
            }
            let _ = state.store.increment_clicks(&code).await;
            Redirect::permanent(&entry.original).into_response()
        }
        Ok(None) => json_response(StatusCode::NOT_FOUND, ErrorResponse {
            error: "not found".into(),
        }),
        Err(e) => {
            tracing::error!("store lookup error: {e}");
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ErrorResponse {
                error: "internal error".into(),
            })
        }
    }
}

pub async fn stats(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Response {
    if !shortener::is_valid_short_code(&code) {
        return json_response(StatusCode::BAD_REQUEST, ErrorResponse {
            error: "invalid short code".into(),
        });
    }

    match state.store.get_by_short_code(&code).await {
        Ok(Some(entry)) => json_response(StatusCode::OK, StatsResponse {
            short_code: entry.short_code.clone(),
            original: entry.original,
            clicks: entry.clicks,
            created_at: entry.created_at,
            redirect_uri: format!("{}/{}", state.base_url, entry.short_code),
        }),
        Ok(None) => json_response(StatusCode::NOT_FOUND, ErrorResponse {
            error: "not found".into(),
        }),
        Err(e) => {
            tracing::error!("store lookup error: {e}");
            json_response(StatusCode::INTERNAL_SERVER_ERROR, ErrorResponse {
                error: "internal error".into(),
            })
        }
    }
}
