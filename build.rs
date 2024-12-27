fn main() {
    std::fs::create_dir_all("src/pb").unwrap();

    let config_builder = tonic_build::configure();

    config_builder
        .out_dir("src/pb")
        .compile_protos(&["./protos/demo.proto"], &["./protos"])
        .unwrap();
}
