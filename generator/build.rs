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

    
    // this hack is necessary because bevy_spacetimedb expects the primary key to be
    // an `Eq` and `Hash` type, but the generated `ChunkCoords` type is not, even though
    // those traits are set in the server code.
    let file = format!("{}/chunk_coords_type.rs", out_dir);
    let src  = std::fs::read_to_string(&file);
    let patched = src.expect("failed to read chunk_coords_type.rs").replace(
        "#[derive(__lib::ser::Serialize, __lib::de::Deserialize, Clone, PartialEq, Debug)]",
        "#[derive(__lib::ser::Serialize, __lib::de::Deserialize, Clone, PartialEq, Debug, Eq, Hash)]",
    );
    std::fs::write(&file, patched).expect("failed to write chunk_coords_type.rs");

    if !status.success() {
        panic!("spacetime codegen failed");
    }

    // 4) Re-run codegen when any server-side schema changes
    println!("cargo:rerun-if-changed={}/src", server_path);
}
