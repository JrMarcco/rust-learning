use std::process::Command;

fn main() {
    let build_enable = option_env!("BUILD_PROTO")
        .map(|v| v == "1")
        .unwrap_or(false);

    if !build_enable {
        println!("### Skipped compiling proto ###");
    }

    prost_build::Config::new()
        .out_dir("src/pb")
        .compile_protos(&["abi.proto"], &["."])
        .unwrap();

    Command::new("cargo")
        .args(&["fmt", "--", "src/*.rs"])
        .status()
        .unwrap();
}
