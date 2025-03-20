use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = "./proto/user_service.proto"; 
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("shared/proto_generated");

    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .build_client(true)
        .build_server(true)
        .file_descriptor_set_path(out_dir.join("user_service.bin"))
        .out_dir(out_dir) 
        .compile(&[proto_file], &["proto"])?;

    Ok(())
}