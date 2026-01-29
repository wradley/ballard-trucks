use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use tracing::Instrument;
use uuid::Uuid;

const X_REQUEST_ID: &str = "x-request-id";

/// Propagates or generates `x-request-id` and attaches it to request span/response headers.
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get(X_REQUEST_ID)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        request
            .headers_mut()
            .insert(HeaderName::from_static(X_REQUEST_ID), header_value.clone());

        let span = tracing::info_span!(
            "req",
            request_id = %request_id,
            method = %request.method(),
            path = %request.uri().path(),
        );

        let mut response = next.run(request).instrument(span).await;
        response
            .headers_mut()
            .insert(HeaderName::from_static(X_REQUEST_ID), header_value);
        return response;
    }

    next.run(request).await
}
