use std::path::PathBuf;
use std::env;
use tonic_build;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let proto_dir = PathBuf::from(manifest_dir).join("src/proto");
    let proto_file = PathBuf::from("user_service_grpc.proto"); 

    if !proto_dir.join(&proto_file).exists() {
        eprintln!("Proto file not found: {:?}", proto_dir.join(&proto_file));
        std::process::exit(1);
    }

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&[&proto_file], &[proto_dir])?;

    println!("Successfully compiled protobuf: {:?}", proto_file);
    Ok(())
}