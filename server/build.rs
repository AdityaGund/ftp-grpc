use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let proto_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("proto");

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("ftp_descriptor.bin"))
        .compile_protos(&["ftp.proto"], &[proto_dir])
        .unwrap();

    Ok(())
}
