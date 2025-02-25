use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let can2040_dir = "../can2040"; // New variable for can2040 directory

    // Run CMake
    Command::new("cmake")
        .args([
            "-B",
            out_dir.to_str().unwrap(),
            "-S",
            can2040_dir, // Using the variable
            "-DCMAKE_BUILD_TYPE=Release",
        ])
        .status()
        .expect("Failed to run cmake");

    // Build the project
    Command::new("cmake")
        .args(["--build", out_dir.to_str().unwrap()])
        .status()
        .expect("Failed to build");

    // Tell cargo to link against the built library
    println!(
        "cargo:rustc-link-search=native={}",
        out_dir.join("").display()
    );
    println!("cargo:rustc-link-lib=static=can2040");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header(format!("{}/src/can2040.h", can2040_dir))
        .derive_default(true)
        .generate_comments(true)
        .use_core()
        .ctypes_prefix("core::ffi")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Tell cargo to rebuild if the C sources change
    println!("cargo:rerun-if-changed={}/src/can2040.c", can2040_dir);
    println!("cargo:rerun-if-changed={}/src/can2040.h", can2040_dir);
    println!("cargo:rerun-if-changed={}/CMakeLists.txt", can2040_dir);
}
