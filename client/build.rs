use std::{env, error::Error, path::PathBuf};

fn main()-> Result<(),Box<dyn Error>>{

    // reflection
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    let _ = tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("ftp_descriptor.bin"))
        .compile_protos(&["proto/ftp.proto"], &["proto"]);

    let _ = tonic_build::compile_protos("../proto/ftp.proto");
    Ok(())
}
