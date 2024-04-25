fn main() {
    #[cfg(feature = "generate")]
    {
        let config = prost_build::Config::new();
        generate(config, "src/genproto/");
    }
}

#[cfg(feature = "generate")]
fn generate(config: prost_build::Config, out_dir: impl AsRef<std::path::Path>) {
    tonic_build::configure()
        .build_server(true)
        .out_dir(out_dir) // you can change the generated code's location
        .compile_with_config(
            config,
            &["submodules/durabletask-protobuf/protos/orchestrator_service.proto"],
            &["durabletask-protobuf"], // specify the root location to search proto dependencies
        )
        .unwrap();
}
