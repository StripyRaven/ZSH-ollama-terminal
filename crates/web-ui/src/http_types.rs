//! crates/web-ui/src/http_types.rs
//! # Strict HTTP Response Typing
//! Гарантированная типизация HTTP ответов с security headers.

use axum::{
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// Типизированные HTTP ответы с compile-time гарантиями
pub struct TypedResponse<T = ()> {
    status: StatusCode,
    headers: HeaderMap,
    body: ResponseBody<T>,
    security: SecurityHeaders,
}

// #[derive(Default)]
pub struct SecurityHeaders {
    pub content_security_policy: Option<String>,
    pub x_frame_options: Option<String>,
    pub x_content_type_options: Option<String>,
    pub strict_transport_security: Option<String>,
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self {
            content_security_policy: Some("default-src 'self'; script-src 'self' 'unsafe-inline' https://unpkg.com/htmx.org@1.9.6".to_string()),
            x_frame_options: Some("DENY".to_string()),
            x_content_type_options: Some("nosniff".to_string()),
            strict_transport_security: None, // Только для HTTPS
        }
    }
}

pub enum ResponseBody<T> {
    Html(String),
    Json(T),
    Plain(String),
}

impl<T> TypedResponse<T> {
    pub fn html_secure(content: String) -> Self {
        Self {
            status: StatusCode::OK,
            headers: Self::html_headers(),
            body: ResponseBody::Html(content),
            security: SecurityHeaders::default(),
        }
    }

    pub fn htmx_fragment(content: String) -> Self {
        let mut headers = Self::html_headers();
        headers.insert("HX-Reswap", "innerHTML".parse().unwrap());

        Self {
            status: StatusCode::OK,
            headers,
            body: ResponseBody::Html(content),
            security: SecurityHeaders::default(),
        }
    }

    pub fn json_secure(data: T) -> Self
    where
        T: Serialize,
    {
        Self {
            status: StatusCode::OK,
            headers: Self::json_headers(),
            body: ResponseBody::Json(data),
            security: SecurityHeaders::default(),
        }
    }

    fn html_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "text/html; charset=utf-8".parse().unwrap());
        headers
    }

    fn json_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "content-type",
            "application/json; charset=utf-8".parse().unwrap(),
        );
        headers
    }
}

impl<T> IntoResponse for TypedResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let mut response = match self.body {
            ResponseBody::Html(html) => html.into_response(),
            ResponseBody::Json(json) => axum::Json(json).into_response(),
            ResponseBody::Plain(text) => text.into_response(),
        };

        *response.status_mut() = self.status;

        let headers = response.headers_mut();
        headers.extend(self.headers);

        // Добавляем security headers
        if let Some(csp) = self.security.content_security_policy {
            headers.insert("content-security-policy", csp.parse().unwrap());
        }
        if let Some(xfo) = self.security.x_frame_options {
            headers.insert("x-frame-options", xfo.parse().unwrap());
        }
        if let Some(xcto) = self.security.x_content_type_options {
            headers.insert("x-content-type-options", xcto.parse().unwrap());
        }
        if let Some(hsts) = self.security.strict_transport_security {
            headers.insert("strict-transport-security", hsts.parse().unwrap());
        }

        response
    }
}

/// Типизированный результат для Web обработчиков
pub type WebResult<T> = Result<TypedResponse<T>, TypedErrorResponse>;

/// Типизированная ошибка с гарантированным HTTP представлением
pub struct TypedErrorResponse {
    status: StatusCode,
    message: String,
    error_type: ErrorType,
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    Validation,
    Security,
    Analysis,
    Template,
    Internal,
}

impl TypedErrorResponse {
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
            error_type: ErrorType::Validation,
        }
    }

    pub fn security_error(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.into(),
            error_type: ErrorType::Security,
        }
    }
}

impl IntoResponse for TypedErrorResponse {
    fn into_response(self) -> Response {
        let body = format!(
            "{}: {}",
            match self.error_type {
                ErrorType::Validation => "Validation Error",
                ErrorType::Security => "Security Error",
                ErrorType::Analysis => "Analysis Error",
                ErrorType::Template => "Template Error",
                ErrorType::Internal => "Internal Error",
            },
            self.message
        );

        TypedResponse {
            status: self.status,
            headers: HeaderMap::new(),
            body: ResponseBody::Plain(body),
            security: SecurityHeaders::default(),
        }
        .into_response()
    }
}
