use client::run_client;
// use destination::run_destination;

#[actix_web::main]
async fn main() {

    println!("\tStarting services...");

    // let destination_handle = actix_web::rt::spawn(async move {
    //     if let Err(e) = run_destination().await {
    //         eprintln!("Destination failed: {}", e);
    //     }
    // });

    let client_handle = actix_web::rt::spawn(async move {
        if let Err(e) = run_client().await {
            eprintln!("Client failed: {}", e);
        }
    });

    let _ = tokio::join!(client_handle);
} 