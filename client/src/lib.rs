use actix_web::{App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use std::env;
use std::io::Result;
use actix_cors::Cors;
use std::path::Path;
use dotenv::dotenv;

pub mod error;
pub mod grpc_client;
pub mod handlers;
pub mod routes;
pub mod models;
pub mod services;
pub mod middleware;

// #[derive(Clone)]
// pub struct AppState {
//     pub notifier: broadcast::Sender<grpc_client::TransferResponse>,
// }

pub async fn run_client() -> Result<()> {
    // Always load the `.env` that sits next to Cargo.toml of the *client* crate,
    // even if the binary is launched from the workspace root. `CARGO_MANIFEST_DIR`
    // is set at compile-time to that directory.
    let client_env_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    // If that specific file exists, load it; otherwise fall back to the default
    // search (current directory and parents).
    if dotenv::from_path(client_env_path.as_path()).is_err() {
        dotenv().ok();
    }

    let host = env::var("CLIENT_HOST").unwrap().to_string();
    let port = env::var("CLIENT_PORT").unwrap().to_string();
    let addr = format!("{}:{}", host, port);

    // let (tx, _rx) = broadcast::channel::<grpc_client::TransferResponse>(100);
    // let app_state = AppState { notifier: tx };

    println!("[CLIENT GRPC] Starting Actix-web server at http://{}", addr);

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            // .app_data(web::Data::new(app_state.clone()))
            .wrap(cors)
            .wrap(HttpAuthentication::bearer(middleware::validator))
            // .app_data(grpc_client.clone())
            .configure(routes::configure_routes)
    })
    .bind(&addr)?
    .run()
    .await
}
