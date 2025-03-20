use tonic_build;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = "./src/proto/user_service.proto"; 

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile(&[proto_file], &["proto"])?;

    Ok(())
}