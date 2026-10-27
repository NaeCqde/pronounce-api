//! The HTTP side: `GET /pronounce` and `GET /health`.
//!
//! `/pronounce` is what `okazuri_bot` calls (`src/pronounce.ts`): the query is the text, the answer is
//! the romaji as `text/plain`. A bad request is a 422 with `{"detail": "..."}`, as the FastAPI
//! server this replaces answered.

use crate::romanizer::{Loanwords, Romanizer};
use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use serde::Deserialize;
use std::sync::Arc;

pub struct AppState {
    pub romanizer: Arc<Romanizer>,
    pub max_query_chars: usize,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/pronounce", get(pronounce))
        .route("/health", get(health))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct PronounceQuery {
    q: Option<String>,
    loanwords: Option<String>,
}

async fn pronounce(State(state): State<Arc<AppState>>, Query(query): Query<PronounceQuery>) -> Response {
    let text = query.q.unwrap_or_default();
    let length = text.chars().count();

    if length == 0 {
        return invalid("q is required (at least 1 character)");
    }
    if length > state.max_query_chars {
        return invalid(&format!("q is too long (at most {} characters)", state.max_query_chars));
    }

    let loanwords = match query.loanwords.as_deref() {
        None | Some("romaji") => Loanwords::Romaji,
        Some("english") => Loanwords::English,
        Some(_) => return invalid("loanwords must be romaji or english"),
    };

    match state.romanizer.romanize(&text, loanwords) {
        Ok(romaji) => plain_text(StatusCode::OK, romaji),
        Err(e) => {
            log::error!("cannot convert {:?}: {}", text, e);
            plain_text(StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string())
        }
    }
}

async fn health() -> Response {
    plain_text(StatusCode::OK, "ok".to_string())
}

fn plain_text(status: StatusCode, body: String) -> Response {
    (status, [(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body).into_response()
}

fn invalid(detail: &str) -> Response {
    let body = format!("{{\"detail\":{}}}", json_string(detail));
    (StatusCode::UNPROCESSABLE_ENTITY, [(header::CONTENT_TYPE, "application/json")], body).into_response()
}

/// A JSON string literal (the messages here are plain ASCII, but a quote must not break the body).
fn json_string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::romanizer_state;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn get_path(path: &str) -> (StatusCode, String, String) {
        let response = router(romanizer_state())
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap().to_string())
            .unwrap_or_default();
        let body = response.into_body().collect().await.unwrap().to_bytes();

        (status, content_type, String::from_utf8(body.to_vec()).unwrap())
    }

    fn encode(text: &str) -> String {
        text.bytes().map(|b| format!("%{b:02X}")).collect()
    }

    #[tokio::test]
    async fn answers_the_romaji_as_plain_text() {
        let (status, content_type, body) =
            get_path(&format!("/pronounce?q={}", encode("チートアイテム管理局のお仕事"))).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(content_type, "text/plain; charset=utf-8");
        assert_eq!(body, "cheat item kanri kyoku no oshigoto");
    }

    #[tokio::test]
    async fn loanwords_english_looks_the_hiragana_words_up() {
        let q = encode("ふらすとれーしょんの話");
        assert_eq!(get_path(&format!("/pronounce?q={q}")).await.2, "furasutore-shon no hanashi");
        assert_eq!(
            get_path(&format!("/pronounce?q={q}&loanwords=english")).await.2,
            "frustration no hanashi"
        );
    }

    #[tokio::test]
    async fn a_bad_request_is_a_422() {
        let (status, content_type, body) = get_path("/pronounce").await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(content_type, "application/json");
        assert!(body.contains("\"detail\""), "{body}");

        assert_eq!(get_path("/pronounce?q=").await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(get_path("/pronounce?q=a&loanwords=xx").await.0, StatusCode::UNPROCESSABLE_ENTITY);

        let long = "あ".repeat(301);
        assert_eq!(
            get_path(&format!("/pronounce?q={}", encode(&long))).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
        // the limit counts characters, not bytes
        let ok = "あ".repeat(300);
        assert_eq!(get_path(&format!("/pronounce?q={}", encode(&ok))).await.0, StatusCode::OK);
    }

    #[tokio::test]
    async fn health_and_unknown_paths() {
        assert_eq!(get_path("/health").await, (StatusCode::OK, "text/plain; charset=utf-8".into(), "ok".into()));
        assert_eq!(get_path("/nothing").await.0, StatusCode::NOT_FOUND);
    }

    #[test]
    fn json_strings_are_escaped() {
        assert_eq!(json_string("a\"b\\c\n"), "\"a\\\"b\\\\c\\u000a\"");
    }
}
