use actix_web::web;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, query_as};
use crate::AppData;
use crate::error::{WebError, WebResult};
use crate::routes::random_string;

#[derive(Debug, Deserialize)]
pub struct RequestQuery {
    /// A string that identifies the request origin as Google.
    /// This string must be registered within your system as Google's unique identifier.
    client_id: String,
    /// A secret string that you registered with Google for your service.
    client_secret: String,
    /// The type of token being exchanged. It's either authorization_code or refresh_token.
    grant_type: GrantType,
    /// When grant_type=authorization_code,
    /// this parameter is the code Google received from either your sign-in or token exchange endpoint.
    code: Option<String>,
    /// When grant_type=authorization_code,
    /// this parameter is the URL used in the initial authorization request.
    redirect_uri: Option<String>,
    /// When grant_type=refresh_token,
    /// this parameter is the refresh token Google received from your token exchange endpoint.
    refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub enum GrantType {
    #[serde(rename = "authorization_code")]
    AuthorizationCode,
    #[serde(rename = "refresh_token")]
    RefreshToken,
}

#[derive(Debug, Serialize)]
pub struct ResponseBody {
    token_type: String,
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
}

pub(crate) async fn token(data: web::Data<AppData>, query: web::Query<RequestQuery>) -> WebResult<web::Json<ResponseBody>> {
    let authorized_client = data.permitted_clients.get(&query.client_id)
        .ok_or(WebError::InvalidGrant)?;

    // Check if the client secret matches what we have on file
    if authorized_client.client_secret.ne(&query.client_secret) {
        return Err(WebError::InvalidGrant);
    }

    match query.grant_type {
        GrantType::AuthorizationCode => {
            let authorization_code = query.code.as_ref().ok_or(WebError::BadRequest)?;
            let redirect_uri = query.redirect_uri.as_ref().ok_or(WebError::BadRequest)?;

            let mut sqlite = data.sqlite.lock().await;

            // Fetch the authorization request associated with the provided code
            let pending_authorization: PendingAuthorization = query_as("SELECT * FROM pending_authorizations WHERE authorization_code = ?")
                .bind(&authorization_code)
                .fetch_optional(&mut *sqlite)
                .await?
                .ok_or(WebError::InvalidGrant)?;

            // Verify the client ID associated with the pending authorization
            // matches what is provided
            if pending_authorization.client_id.ne(&query.client_id) {
                return Err(WebError::InvalidGrant);
            }

            // Check that the code hasn't expired yet
            if time::OffsetDateTime::now_utc().unix_timestamp() >= pending_authorization.expires_at {
                return Err(WebError::InvalidGrant);
            }

            // Check the provided redirect URI matches what we have from the authorization request
            if pending_authorization.redirect_uri.ne(redirect_uri) {
                return Err(WebError::InvalidGrant);
            }

            let authorization_id = random_string(16);
            let access_token = random_string(32);
            let refresh_token = random_string(32);
            // Google recommends 1 hour for the access  token
            let access_token_expires_at = (time::OffsetDateTime::now_utc() + time::Duration::hours(1)).unix_timestamp();

            // Create the authorization row
            sqlx::query("INSERT INTO authorizations (authorization_id, client_id, client_secret, refresh_token) VALUES (?, ?, ?, ?)")
                .bind(&authorization_id)
                .bind(&query.client_id)
                .bind(&query.client_secret)
                .bind(&refresh_token)
                .execute(&mut *sqlite)
                .await?;

            // Create the access token row
            sqlx::query("INSERT INTO access_tokens (access_token, expires_at, authorization_id) VALUES (?, ?, ?)")
                .bind(&authorization_id)
                .bind(access_token_expires_at)
                .bind(&access_token)
                .execute(&mut *sqlite)
                .await?;

            // Done
            Ok(web::Json(ResponseBody {
                token_type: "Bearer".to_string(),
                access_token,
                refresh_token: Some(refresh_token),
                expires_in: time::OffsetDateTime::now_utc().unix_timestamp() - access_token_expires_at,
            }))
        },
        GrantType::RefreshToken => {
            let refresh_token = query.refresh_token.as_ref().ok_or(WebError::BadRequest)?;

            let mut sqlite = data.sqlite.lock().await;

            // Fetch the authorization, if there is any
            let authorization: Authorization = query_as("SELECT * FROM authorizations WHERE refresh_token = ?")
                .bind(&refresh_token)
                .fetch_optional(&mut *sqlite)
                .await?
                .ok_or(WebError::InvalidGrant)?;

            // Check that the client ID associated with the authorization
            // matches the request
            if authorization.client_id.ne(&query.client_id) {
                return Err(WebError::InvalidGrant);
            }

            let access_token = random_string(32);
            // Google recommends 1 hour for the access  token
            let access_token_expires_at = (time::OffsetDateTime::now_utc() + time::Duration::hours(1)).unix_timestamp();

            // Create the access token row
            sqlx::query("INSERT INTO access_tokens (access_token, expires_at, authorization_id) VALUES (?, ?, ?)")
                .bind(&authorization.authorization_id)
                .bind(access_token_expires_at)
                .bind(&access_token)
                .execute(&mut *sqlite)
                .await?;

            // Done
            Ok(web::Json(ResponseBody {
                token_type: "Bearer".to_string(),
                access_token,
                refresh_token: None,
                expires_in: time::OffsetDateTime::now_utc().unix_timestamp() - access_token_expires_at,
            }))
        }
    }
}

#[derive(FromRow)]
struct PendingAuthorization {
    client_id: String,
    redirect_uri: String,
    expires_at: i64,
}

#[derive(FromRow)]
struct Authorization {
    authorization_id: String,
    client_id: String,
}