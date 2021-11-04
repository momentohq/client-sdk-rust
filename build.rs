use std::io::Result;

fn main() -> Result<()> {
    tonic_build::configure()
        .build_server(false)
        // .out_dir("src/generated")
        .compile(
            &[
                "src/protos/controlclient.proto",
                "src/protos/cacheclient.proto",
            ],
            &["src/protos"],
        )
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
    Ok(())
}
