use crate::server::handler::WebAppHandler;
use axum::body::Body;
use axum::extract;
use axum::http::{header, Response, StatusCode};
use extract::Path;

#[derive(rust_embed::RustEmbed)]
#[folder = "./web/"]
struct WebAppAssets;

impl WebAppHandler {
    pub async fn get(path: Option<Path<String>>) -> impl axum::response::IntoResponse {
        tracing::debug!("Serving {:?}", path);
        let key = match path {
            Some(Path(p)) => {
                if p.is_empty() {
                    "index.html".to_string()
                } else {
                    p
                }
            }
            None => "index.html".to_string(),
        };
        if let Some(content) = WebAppAssets::get(&key) {
            let mime = mime_guess::from_path(key).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data.into_owned()))
                .unwrap()
        } else {
            // SPA fallback
            if let Some(index) = WebAppAssets::get("index.html") {
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(Body::from(index.data.into_owned()))
                    .unwrap()
            } else {
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("Not found"))
                    .unwrap()
            }
        }
    }
}
