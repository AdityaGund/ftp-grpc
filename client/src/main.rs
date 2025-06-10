use actix_web::{App, HttpServer};
use dotenv::dotenv;
use std::env;
use std::io::Result;

mod error;
mod grpc_client;
mod handlers;
mod routes;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let host = env::var("CLIENT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("CLIENT_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{}:{}", host, port);

    println!("Starting Actix-web server at http://{}", addr);

    HttpServer::new(move || {
        App::new().configure(routes::configure_routes)
    })
    .bind(&addr)?
    .run()
    .await
}