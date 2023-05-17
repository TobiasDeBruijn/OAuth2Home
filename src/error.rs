use actix_web::{HttpResponse, ResponseError};
use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use thiserror::Error;

pub(crate) type WebResult<T> = Result<T, WebError>;

#[derive(Debug, Error)]
pub(crate) enum WebError {
    #[error("Unknown OAuth2 Client")]
    UnknownClient,
    #[error("Redirect URI provided does not match known redirect URI for this client")]
    RedirectUriMismatch,
    #[error("Response type does not match expectations")]
    ResponseTypeMismatch,
    #[error("Bad request")]
    BadRequest,
    #[error("Invalid grant")]
    InvalidGrant,
    #[error("Internal server error")]
    Sqlx(#[from] sqlx::error::Error),
    #[error("Internal server error")]
    SerializeQuery(#[from] serde_qs::Error),
}

impl ResponseError for WebError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::UnknownClient => StatusCode::FORBIDDEN,
            Self::RedirectUriMismatch => StatusCode::BAD_REQUEST,
            Self::ResponseTypeMismatch => StatusCode::BAD_REQUEST,
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::InvalidGrant => StatusCode::BAD_REQUEST,
            Self::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SerializeQuery(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            Self::InvalidGrant => HttpResponse::build(self.status_code())
                .content_type(ContentType::json())
                .body(r#"{"error": "invalid_grant"}"#),
            _ => HttpResponse::build(self.status_code())
                .content_type(ContentType::plaintext())
                .body(self.to_string())
        }
    }
}