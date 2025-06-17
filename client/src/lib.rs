use actix_web::{App, HttpServer};
use dotenv::dotenv;
use std::env;
use std::io::Result;
use actix_cors::Cors;
use tokio::sync::broadcast;
use actix_web::web;

pub mod error;
pub mod grpc_client;
pub mod handlers;
pub mod routes;

#[derive(Clone)]
pub struct AppState {
    pub notifier: broadcast::Sender<grpc_client::TransferResponse>,
}

pub async fn run_client() -> Result<()> {
    dotenv().ok();

    let host = env::var("CLIENT_HOST").unwrap().to_string();
    let port = env::var("CLIENT_PORT").unwrap().to_string();
    let addr = format!("{}:{}", host, port);

    let (tx, _rx) = broadcast::channel::<grpc_client::TransferResponse>(100);
    let app_state = AppState { notifier: tx };

    println!("[CLIENT GRPC] Starting Actix-web server at http://{}", addr);

    HttpServer::new(move || {
        let cors = Cors::default()
        .allowed_origin("http://localhost:5173") // Replace with your frontend's origin
        .allowed_methods(vec!["GET", "POST"]) // Allow your desired methods
        .allowed_headers(vec![
            actix_web::http::header::CONTENT_TYPE,
            actix_web::http::header::ACCEPT,
        ]) // Allow your desired headers
        .expose_headers(vec![actix_web::http::header::CONTENT_LENGTH]) // Expose headers to the client
        .max_age(3600); // Set the preflight request max age in seconds

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(cors)
            // .app_data(grpc_client.clone())
            .configure(routes::configure_routes)
    })
    .bind(&addr)?
    .run()
    .await
}
