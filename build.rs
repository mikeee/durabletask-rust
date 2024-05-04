#[cfg(feature = "genproto")]
fn genproto() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = prost_build::Config::new();
    config
        .default_package_filename("microsoft.durabletask.implementation.protobuf") // TODO: remove override for non-existent package name
        .enable_type_names();

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .build_transport(true)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .extern_path(".google.protobuf.Duration", "::prost_wkt_types::Duration")
        .extern_path(".google.protobuf.Timestamp", "::prost_wkt_types::Timestamp")
        .out_dir("src/genproto") // you can change the generated code's location
        .compile_with_config(
            config,
            &["submodules/durabletask-protobuf/protos/orchestrator_service.proto"],
            &["submodules/durabletask-protobuf/"], // specify the root location to search proto dependencies
        )?;
    Ok(())
}

fn main() {
    #[cfg(feature = "genproto")]
    {
        println!("compiling protos");
        match genproto() {
            Ok(_) => {
                println!("compiled protos")
            }
            Err(e) => {
                panic!("{:?}", e)
            }
        };
    }
}
