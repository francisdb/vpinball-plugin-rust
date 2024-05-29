use std::env;
use std::fs::{metadata, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

fn main() {
    let header_file_name = "VPXPlugin.h";

    // download the header file if it does not exist or is older than 24 hours
    download_header_file(header_file_name);

    let bindings = bindgen::Builder::default()
        .header(header_file_name)
        //.clang_arg("-std=c99")
        .clang_arg("-x")
        .clang_arg("c++")
        // .clang_arg("-std=c++14")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn download_header_file(header_file_name: &str) {
    let path = PathBuf::from(header_file_name);

    let should_download = if let Ok(metadata) = metadata(&path) {
        let modified = metadata
            .modified()
            .expect("Failed to get modification time");
        let elapsed = modified
            .elapsed()
            .expect("Failed to calculate elapsed time");
        elapsed > Duration::from_secs(24 * 60 * 60) // more than 24 hours
    } else {
        true // file does not exist
    };

    if should_download {
        let url =
            "https://raw.githubusercontent.com/vpinball/vpinball/10.8.1/src/plugins/VPXPlugin.h";
        eprintln!("Downloading {header_file_name} from {url}");
        let response = reqwest::blocking::get(url).expect("Failed to download file");
        assert!(
            response.status().is_success(),
            "Failed to download file, status: {}",
            response.status()
        );
        let content = response.bytes().expect("Failed to read response bytes");
        let mut file = File::create(header_file_name).expect("Failed to create header file");
        file.write_all(&content).expect("Failed to write to file");
    }
}
