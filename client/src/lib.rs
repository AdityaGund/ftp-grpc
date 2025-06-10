use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use std::env;
use std::io::Result;
use std::sync::Arc;
use crate::grpc_client::TransferServiceClient;

pub mod error;
pub mod grpc_client;
pub mod handlers;
pub mod routes;

pub async fn run_client() -> Result<()> {
    dotenv().ok();

    let host = env::var("CLIENT_HOST").unwrap().to_string();
    let port = env::var("CLIENT_PORT").unwrap().to_string();
    let addr = format!("{}:{}", host, port);

    // let destination_port = env::var("DESTINATION_PORT").unwrap().to_string();
    // let destination_addr = format!("http://127.0.0.1:{}", destination_port);
    // let grpc_client = TransferServiceClient::connect(destination_addr).await.expect("Failed to connect to destination service");
    // let grpc_client = web::Data::new(Arc::new(grpc_client));

    println!("[CLIENT GRPC] Starting Actix-web server at http://{}", addr);

    HttpServer::new(move || {
        App::new()
            // .app_data(grpc_client.clone())
            .configure(routes::configure_routes)
    })
    .bind(&addr)?
    .run()
    .await
}
