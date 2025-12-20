fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .out_dir("src")
        .compile_protos(&["proto/service.proto"], &["proto"])?;
    Ok(())
}
