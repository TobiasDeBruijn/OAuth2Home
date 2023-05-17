use actix_route_config::Routable;
use actix_web::web;
use actix_web::web::ServiceConfig;
use rand::Rng;

mod authorization;
mod token;

fn random_string(len: usize) -> String {
    rand::thread_rng().sample_iter(rand::distributions::Alphanumeric).take(len).map(char::from).collect()
}

pub struct Router;

impl Routable for Router {
    fn configure(config: &mut ServiceConfig) {
        config
            .route("/authorization", web::get().to(authorization::authorization))
            .route("/token", web::post().to(token::token));
    }
}