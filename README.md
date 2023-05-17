# OAuth2Home
OAuth2 server implementation for use with Google Home for [Actix Web](https://crates.io/crates/actix-web).  
Does not require logging in by the user, all access granted!

## Example
```rust
use actix_web::{App, HttpServer, web};
use actix_route_config::Routable;

async fn main() -> color_eyre::Result<()> {
    let oauth2 = oauth2home::configure_server(oauth2home::ServerConfig {
        database_path: PathBuf::from_str("./db.sqlite")?,
        permitted_clients: vec![
            ("my_client_id".to_string(), oauth2home::PermittedClient {
                client_secret: "super_secret_token".to_string(),
                redirect_uri: "https://foo.example.com".to_string()
            })
        ].into_iter().collect()
    }).await?;

    HttpServer::new(move || App::new()
        .service(web::scope("/oauth")
            .configure(|cfg | oauth2.configure_non_static(cfg))
        )
    )
    .bind("0.0.0.0:8000")?
    .run()
    .await?;
}
```

This will serve the authorization endpoints at:
- `/oauth/authorization` The authorization endpoint
- `/oauth/token` The token exchange endpoint

## License
This project is licensed under:
- [MIT](LICENSE-MIT)
- [Apache 2.0](LICENSE-APACHE)

At your option.