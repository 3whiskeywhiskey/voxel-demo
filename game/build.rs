// generator/build.rs

use std::env;
use std::process::Command;

fn main() {
    // 1) Where is our server module on disk?
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");
    let server_path = format!("{}/../server", manifest_dir);

    // 2) Where do we want the Rust bindings to land?
    let out_dir = format!("{}/src/stdb", manifest_dir);

    // 3) Invoke the Spacetime CLI
    let status = Command::new("spacetime")
        .arg("generate")
        .arg("--lang").arg("rust")
        .arg("--out-dir").arg(&out_dir)
        .arg("--project-path").arg(&server_path)
        .status()
        .expect("failed to execute `spacetime generate`");

    if !status.success() {
        panic!("spacetime codegen failed");
    }

    // 4) Re-run codegen when any server-side schema changes
    println!("cargo:rerun-if-changed={}/src", server_path);
}
