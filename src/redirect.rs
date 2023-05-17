use actix_web::body::BoxBody;
use actix_web::{HttpRequest, HttpResponse, Responder};
use actix_web::http::StatusCode;

pub struct Redirect {
    to: String,
}

impl Redirect {
    pub fn new(to: String) -> Self {
        Redirect {
            to
        }
    }
}

impl Responder for Redirect {
    type Body = BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
            .insert_header(("Location", self.to))
            .finish()
    }
}