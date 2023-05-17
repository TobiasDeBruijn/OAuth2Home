use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use actix_route_config::Routable;
use actix_web::web;
use actix_web::web::ServiceConfig;
use sqlx::{ConnectOptions, FromRow, migrate, SqliteConnection};
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;
use tokio::sync::Mutex;

mod routes;
mod error;
mod redirect;

const MIGRATOR: Migrator = migrate!();

#[derive(Debug, Clone)]
pub struct PermittedClient {
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Path to the file where the SQLite database should be stored
    pub database_path: PathBuf,
    pub permitted_clients: HashMap<String, PermittedClient>,
}

#[derive(Debug, Clone)]
pub(crate) struct AppData {
    /// Sqlite database connection
    sqlite: Arc<Mutex<SqliteConnection>>,
    /// Allowed OAuth2 clients
    permitted_clients: HashMap<String, PermittedClient>,
}

/// Responsible for handling
/// OAuth2 routes.
#[derive(Debug, Clone)]
pub struct OAuth2Server {
    appdata: AppData,
}

impl Routable for OAuth2Server {
    fn configure_non_static(&self, config: &mut ServiceConfig) {
        config.service(web::scope("/oauth")
            .app_data(self.appdata.clone())

        );
    }
}

pub async fn configure_server(server: ServerConfig) -> sqlx::Result<OAuth2Server> {
    let sqlite = open_database(&server).await?;
    Ok(OAuth2Server {
        appdata: AppData {
            sqlite: Arc::new(Mutex::new(sqlite)),
            permitted_clients: server.permitted_clients
        },
    })
}

#[derive(FromRow)]
struct AccessToken {
    expires_at: i64,
}

/// Check if an access  token is valid.
/// Returns `Ok(true)` if the token is valid.
///
/// # Errors
///
/// If database communication failed.
pub async fn check_access<S: AsRef<str>>(oauth2_server: &OAuth2Server, access_token: S) -> sqlx::Result<bool> {
    let mut sqlite = oauth2_server.appdata.sqlite.lock().await;

    let token: Option<AccessToken> = sqlx::query_as("SELECT * FROM access_tokens WHERE access_token = ?")
        .bind(access_token.as_ref())
        .fetch_optional(&mut *sqlite)
        .await?;

    match token {
        Some(token) => Ok(token.expires_at > time::OffsetDateTime::now_utc().unix_timestamp()),
        None => Ok(false)
    }
}

/// Open a database connection to the database.
/// Creates a database if it doesn't yet exist.
/// Applies migrations if needed.
///
/// # Errors
///
/// - If opening the database fails.
/// - If applying migrations fails.
async fn open_database(config: &ServerConfig) -> sqlx::Result<SqliteConnection> {
    let mut sqlite = SqliteConnectOptions::new()
        .create_if_missing(true)
        .filename(&config.database_path)
        .connect()
        .await?;

    MIGRATOR.run(&mut sqlite).await?;

    Ok(sqlite)
}