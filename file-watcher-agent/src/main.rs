use dotenv::dotenv;
use notify::{event::ModifyKind, Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{env, fs, io, path::Path};
use notify::event::CreateKind;

fn main() {
    let env_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    if dotenv::from_path(env_path.as_path()).is_err() {
        dotenv().ok();
    }

    let path = env::var("FILE_PATH").unwrap().to_string();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Watching {path}");

    if let Err(error) = watch(path) {
        log::error!("Error: {error:?}");
    }
}

fn read_and_store_file<P: AsRef<Path>>(src: P) -> io::Result<()> {
    let dest_dir = Path::new("./stored_files");
    fs::create_dir_all(dest_dir)?;
    let filename = src.as_ref()
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "missing filename"))?;
    let dest_path = dest_dir.join(filename);
    fs::copy(&src, &dest_path)?;
    log::info!("Stored new file at {:?}", dest_path);
    Ok(())
}


fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                if matches!(
                    event.kind,
                    EventKind::Create(CreateKind::File) | EventKind::Create(CreateKind::Any) | EventKind::Modify(ModifyKind::Any)
                ) {
                    // log::info!("Change: {event:?}");
                    for path in event.paths {
                        if let Err(e) = read_and_store_file(&path) {
                            log::error!("Failed to store {path:?}: {e}");
                        }
                    }
                }
            },
            Err(error) => log::error!("Error: {error:?}\n\n"),
        }
    }

    Ok(())
}