use crate::config::ServerConfig;
use crate::error::{AppError, Result};
use axum::{
    extract::{Request, State},
    http::{
        HeaderMap, HeaderValue, Method, StatusCode,
        header::{
            ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS,
            ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_MAX_AGE, ACCESS_CONTROL_REQUEST_HEADERS,
            ACCESS_CONTROL_REQUEST_METHOD, ORIGIN, VARY,
        },
    },
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashSet;
use std::sync::Arc;

const CORS_PREFLIGHT_MAX_AGE_SECONDS: &str = "86400";

/// Validated CORS policy derived from the server configuration.
#[derive(Debug, Clone)]
pub struct CorsPolicy {
    same_origin: String,
    allowed_origins: HashSet<String>,
}

impl CorsPolicy {
    /// Creates a validated CORS policy from the server configuration.
    ///
    /// # Arguments
    /// * `server` - Server configuration containing `base_url` and `cors_origins`.
    ///
    /// # Returns
    /// Returns a normalized policy that compares origins as `scheme://host[:port]`.
    ///
    /// # Errors
    /// Returns `AppError::Config` if `base_url` or any configured origin is invalid.
    pub fn from_server_config(server: &ServerConfig) -> Result<Self> {
        let same_origin = normalize_origin(&server.base_url, "server.base_url")?;
        let allowed_origins = server
            .cors_origins
            .iter()
            .map(|origin| normalize_origin(origin, "server.cors_origins"))
            .collect::<Result<HashSet<_>>>()?;

        Ok(Self {
            same_origin,
            allowed_origins,
        })
    }

    fn allows_origin(&self, origin: &str) -> bool {
        origin == self.same_origin || self.allowed_origins.contains(origin)
    }
}

/// Enforces the configured CORS policy for every request.
///
/// Requests without an `Origin` header are treated as non-CORS requests.
/// Requests with an `Origin` header must be same-origin or explicitly whitelisted.
/// Valid preflight requests are handled directly by this middleware.
pub async fn enforce_cors(
    State(policy): State<Arc<CorsPolicy>>,
    req: Request,
    next: Next,
) -> Result<Response> {
    let Some(origin) = req.headers().get(ORIGIN).cloned() else {
        return Ok(next.run(req).await);
    };

    let origin_str = origin
        .to_str()
        .map_err(|_| AppError::Forbidden("Invalid Origin header".to_string()))?;
    if !policy.allows_origin(origin_str) {
        return Err(AppError::Forbidden(
            "CORS origin is not allowed".to_string(),
        ));
    }

    if req.method() == Method::OPTIONS && req.headers().contains_key(ACCESS_CONTROL_REQUEST_METHOD)
    {
        let request_method = req
            .headers()
            .get(ACCESS_CONTROL_REQUEST_METHOD)
            .cloned()
            .ok_or_else(|| {
                AppError::Forbidden("Missing Access-Control-Request-Method header".to_string())
            })?;
        let request_headers = req.headers().get(ACCESS_CONTROL_REQUEST_HEADERS).cloned();
        return Ok(build_preflight_response(
            origin,
            request_method,
            request_headers,
        ));
    }

    let mut response = next.run(req).await;
    apply_actual_cors_headers(response.headers_mut(), &origin);

    Ok(response)
}

fn normalize_origin(value: &str, field_name: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Config(format!(
            "{} must not be empty",
            field_name
        )));
    }

    let uri = trimmed
        .parse::<axum::http::Uri>()
        .map_err(|e| AppError::Config(format!("Invalid {} '{}': {}", field_name, trimmed, e)))?;

    let scheme = uri.scheme_str().ok_or_else(|| {
        AppError::Config(format!(
            "Invalid {} '{}': missing URL scheme",
            field_name, trimmed
        ))
    })?;
    let authority = uri.authority().ok_or_else(|| {
        AppError::Config(format!(
            "Invalid {} '{}': missing host",
            field_name, trimmed
        ))
    })?;

    Ok(format!("{}://{}", scheme, authority))
}

fn build_preflight_response(
    origin: HeaderValue,
    request_method: HeaderValue,
    request_headers: Option<HeaderValue>,
) -> Response {
    let mut response = StatusCode::NO_CONTENT.into_response();
    let headers = response.headers_mut();

    headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, origin);
    headers.insert(ACCESS_CONTROL_ALLOW_METHODS, request_method);

    match request_headers {
        Some(value) => {
            headers.insert(ACCESS_CONTROL_ALLOW_HEADERS, value);
        }
        None => {
            headers.insert(
                ACCESS_CONTROL_ALLOW_HEADERS,
                HeaderValue::from_static("authorization, content-type"),
            );
        }
    }

    headers.insert(
        ACCESS_CONTROL_MAX_AGE,
        HeaderValue::from_static(CORS_PREFLIGHT_MAX_AGE_SECONDS),
    );
    append_vary_headers(headers);

    response
}

fn apply_actual_cors_headers(headers: &mut HeaderMap, origin: &HeaderValue) {
    headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, origin.clone());
    append_vary_headers(headers);
}

fn append_vary_headers(headers: &mut HeaderMap) {
    headers.append(VARY, HeaderValue::from_static("Origin"));
    headers.append(
        VARY,
        HeaderValue::from_static("Access-Control-Request-Method"),
    );
    headers.append(
        VARY,
        HeaderValue::from_static("Access-Control-Request-Headers"),
    );
}
