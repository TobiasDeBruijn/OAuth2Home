use actix_web::web;
use serde::{Deserialize, Serialize};
use crate::AppData;
use crate::error::{WebError, WebResult};
use crate::redirect::Redirect;
use crate::routes::random_string;

#[derive(Debug, Deserialize)]
pub struct RequestQuery {
    client_id: String,
    redirect_uri: String,
    state: String,
    response_type: String,
}

#[derive(Debug, Serialize)]
struct ResponseQuery<'a> {
    code: &'a str,
    state: &'a str,
}

pub(crate) async fn authorization(data: web::Data<AppData>, query: web::Query<RequestQuery>) -> WebResult<Redirect> {
    if let Some(permitted_client) = data.permitted_clients.get(&query.client_id) {
        // Check the redirect URI
        if permitted_client.redirect_uri.ne(&query.redirect_uri) {
            return Err(WebError::RedirectUriMismatch);
        }

        // Check the response type
        if query.response_type.ne("code") {
            return Err(WebError::ResponseTypeMismatch);
        }

        // Create an authorization code and insert it into the database
        let authorization_code = random_string(32);
        let expires_at = (time::OffsetDateTime::now_utc() + time::Duration::minutes(10)).unix_timestamp();
        let mut sqlite = data.sqlite.lock().await;
        sqlx::query("INSERT INTO pending_authorizations (client_id, authorization_code, redirect_uri, expires_at) VALUES (?, ?, ?, ?)")
            .bind(&query.client_id)
            .bind(&authorization_code)
            .bind(&query.redirect_uri)
            .bind(expires_at)
            .execute(&mut *sqlite)
            .await?;

        // Prepare the redirect
        let response_query = serde_qs::to_string(&ResponseQuery {
            code: &authorization_code,
            state: &query.state,
        })?;
        let redirect_to = format!("{}?{}", query.redirect_uri, response_query);
        Ok(Redirect::new(redirect_to))

    } else {
        Err(WebError::UnknownClient)
    }
}