use client::run_client;
use destination::run_destination;
use client::grpc_client::{transfer_data, ftp::transfer_service_client::TransferServiceClient};
use std::collections::HashMap;
use dotenv::dotenv;
use notify::{event::ModifyKind, Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{env, fs, io, path::Path};
use notify::event::CreateKind;
use std::path::PathBuf;
use tokio::task;
use tokio::sync::mpsc;
use serde_json;

const FILE_SIZES_STORE: &str = "file_sizes.json";

#[actix_web::main]
async fn main() {
    let env_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    if dotenv::from_path(env_path.as_path()).is_err() {
        dotenv().ok();
    }

    let agentic_mode = env::var("AGENTIC_MODE").unwrap();

    // agentic mode + server
    if agentic_mode == "TRUE".to_string() {

        let destination_handle = actix_web::rt::spawn(async move {
            if let Err(e) = run_destination().await {
                eprintln!("Destination failed: {}", e);
            }
        });
    
        let client_handle = actix_web::rt::spawn(async move {
            if let Err(e) = run_client().await {
                eprintln!("Client failed: {}", e);
            }
        });
    
        
        let path = env::var("FILE_PATH").unwrap().to_string();
        
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
        
        
        let watch_handle = actix_web::rt::spawn(async move {
            // log::info!("Watching {path}");
            if let Err(e) = watch_async(path).await {
                log::error!("Watcher error: {e:?}");
            }
        });
        
        let _ = tokio::join!(client_handle, destination_handle, watch_handle);
    }
    // only server
    else {

        println!("\tStarting services...");
    
        let destination_handle = actix_web::rt::spawn(async move {
            if let Err(e) = run_destination().await {
                eprintln!("Destination failed: {}", e);
            }
        });
    
        let client_handle = actix_web::rt::spawn(async move {
            if let Err(e) = run_client().await {
                eprintln!("Client failed: {}", e);
            }
        });
    
        let _ = tokio::join!(client_handle, destination_handle);
    }
} 

async fn send_file_via_grpc(path: PathBuf) {
    let dest_host = env::var("ADMIN_SERVER_HOST").unwrap().to_string();
    let dest_port = env::var("ADMIN_SERVER_PORT").unwrap().to_string();
    let dest_bank_id = env::var("ADMIN_ID").unwrap().to_string();
    let dest_bank_ip = dest_host.clone();
    let sender_bank_id = env::var("SENDER_BANK_ID").unwrap().to_string();

    let endpoint = format!("http://{}:{}", dest_host, dest_port);

    // println!("connecting");
    if let Ok(contents) = fs::read_to_string(&path) {
        println!("[DEBUG] Contents being sent:\n{}", contents);
    } else if let Ok(bytes) = fs::read(&path) {
        println!("[DEBUG] Sending {} bytes (binary)", bytes.len());
    }

    match TransferServiceClient::connect(endpoint.clone()).await {
        Ok(mut client) => {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let file_path_str = path.to_string_lossy().to_string();

    // println!("transfering");

            if let Err(e) = transfer_data(
                &mut client,
                Some((&file_path_str, &file_name)),
                None,
                Some(&dest_bank_id),
                Some(&dest_bank_ip),
                Some(&sender_bank_id),
                true
            )
            .await
            {
                log::error!("gRPC transfer failed: {:?}", e);
            }
        }
        Err(e) => {
            log::error!("Could not connect to destination gRPC server ({}): {:?}", endpoint, e);
        }
    }
}

fn save_file_sizes(map: &HashMap<PathBuf, u64>) {
    let string_map: HashMap<String, u64> = map
        .iter()
        .map(|(k, v)| (k.to_string_lossy().to_string(), *v))
        .collect();

    match serde_json::to_string(&string_map) {
        Ok(json) => {
            if let Err(e) = fs::write(FILE_SIZES_STORE, json) {
                log::error!("Failed to write {FILE_SIZES_STORE}: {e}");
            }
        }
        Err(e) => log::error!("Failed to serialise file sizes: {e}"),
    }
}

fn process_file_change<P: AsRef<Path>>(src: P, file_sizes: &mut HashMap<PathBuf, u64>) -> io::Result<()> {
    // println!("started");

    let src_path = src.as_ref();

    if !src_path.is_file() {
        return Ok(());
    }

    // println!("on it");
    let src_size = fs::metadata(src_path)?.len();
    let last_size = file_sizes.get(src_path).copied().unwrap_or(0);
    
    // println!("update map and that");
    if src_size > last_size {
        // Update map first so repeated events don't resend the same data.
        file_sizes.insert(src_path.to_path_buf(), src_size);
        save_file_sizes(file_sizes);

        // Read the delta bytes ------------------------------------------------
        use std::io::{Read, Seek, SeekFrom};
        let mut src_f = fs::File::open(src_path)?;
        src_f.seek(SeekFrom::Start(last_size))?;
        let mut buffer = Vec::with_capacity((src_size - last_size) as usize);
        src_f.read_to_end(&mut buffer)?;

        // Write them to a temporary file -------------------------------------
        let tmp_file_path = std::env::temp_dir().join(
            src_path.file_name().unwrap_or_default(),
        );
        fs::write(&tmp_file_path, &buffer)?;

        // Spawn async transfer ------------------------------------------------
        // println!("pre spawn");
        let tmp_clone = tmp_file_path.clone();
        task::spawn(async move {
            // println!("spawn");
            send_file_via_grpc(tmp_clone.clone()).await;

            if let Err(e) = tokio::fs::remove_file(tmp_clone).await {
                log::error!("Failed to remove temp file: {e}");
            }
        });
    }

    // println!("ended");
    Ok(())
}

async fn watch_async<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<notify::Result<notify::Event>>();

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        Config::default(),
    )?;

    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    let mut file_sizes: HashMap<PathBuf, u64> =
        if let Ok(data) = fs::read_to_string(FILE_SIZES_STORE) {
            match serde_json::from_str::<HashMap<String, u64>>(&data) {
                Ok(map) => map
                    .into_iter()
                    .map(|(k, v)| (PathBuf::from(k), v))
                    .collect(),
                Err(_) => HashMap::new(),
            }
        } else {
            HashMap::new()
        };

    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                if matches!(
                    event.kind,
                    EventKind::Create(CreateKind::File) | EventKind::Create(CreateKind::Any) | EventKind::Modify(ModifyKind::Any)
                ) {
                    log::info!("Change: {event:?}");
                    for path in event.paths {
                        if let Err(e) = process_file_change(&path, &mut file_sizes) {
                            log::error!("Failed to process {path:?}: {e}");
                        }
                    }
                }
            }
            Err(error) => log::error!("Watcher error: {error:?}"),
        }
    }

    Ok(())
}


// fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
//     let (tx, rx) = std::sync::mpsc::channel();

//     // Automatically select the best implementation for your platform.
//     // You can also access each implementation directly e.g. INotifyWatcher.
//     let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

//     // Add a path to be watched. All files and directories at that path and
//     // below will be monitored for changes.
//     watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

//     // Track previously observed file sizes to detect growth/truncation
//     let mut file_sizes: HashMap<PathBuf, u64> = HashMap::new();

//     for res in rx {
//         match res {
//             Ok(event) => {
//                 if matches!(
//                     event.kind,
//                     EventKind::Create(CreateKind::Any) | EventKind::Modify(ModifyKind::Any)
//                 ) {
//                     log::info!("Change: {event:?}");
//                     for path in event.paths {
//                         if let Err(e) = process_file_change(&path, &mut file_sizes) {
//                             log::error!("Failed to process {path:?}: {e}");
//                         }
//                     }
//                 }
//             },
//             Err(error) => log::error!("Error: {error:?}\n\n"),
//         }
//     }

//     Ok(())
// }

