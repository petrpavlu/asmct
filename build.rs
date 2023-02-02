// Copyright (C) 2023 Petr Pavlu <petr.pavlu@dagobah.cz>
// SPDX-License-Identifier: GPL-3.0-or-later

extern crate bindgen;

const LIBOPCODES_SO_DIR: &'static str = "/usr/lib64";
const LIBLLVM15_SO_DIR: &'static str = "/usr/lib64";

fn add_library(dir: &str, prefix: &str, suffix: &str) -> std::io::Result<()> {
    assert!(prefix.starts_with("lib"));

    println!("cargo:rustc-link-search=native={}", dir);
    let dir_iter = match std::fs::read_dir(dir) {
        Ok(dir_iter) => dir_iter,
        Err(e) => {
            eprintln!("Failed to read directory {}: {}", dir, e);
            return Err(e);
        }
    };
    let mut maybe_libname: Option<String> = None;
    for entry_or_err in dir_iter {
        let entry = match entry_or_err {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Failed to read directory {}: {}", dir, e);
                return Err(e);
            }
        };
        let name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => continue,
        };
        if name.starts_with(prefix) && name.ends_with(suffix) {
            if let Some(prev) = maybe_libname.as_ref() {
                if name.len() > prev.len() {
                    maybe_libname = Some(name);
                }
            } else {
                maybe_libname = Some(name);
            }
        }
    }

    let libname = match maybe_libname {
        Some(libname) => libname,
        None => {
            eprintln!("Failed to find {} in {}", prefix, dir);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Missing {}!", prefix),
            ));
        }
    };
    println!("cargo:rustc-link-arg=-l:{}", libname);
    Ok(())
}

fn main() -> std::io::Result<()> {
    // Find needed libraries.
    add_library(&LIBOPCODES_SO_DIR, "libopcodes", ".so")?;
    add_library(&LIBLLVM15_SO_DIR, "libLLVM.so.15", "")?;

    // Generate bindings.
    cc::Build::new()
        .file("wrapper/wrapper.c")
        .compile("wrapper");

    let bindings = match bindgen::Builder::default()
        .header("wrapper/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
    {
        Ok(bindings) => bindings,
        Err(e) => {
            eprintln!("Failed to generate bindings to C libraries: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Bindings-generation failed!",
            ));
        }
    };

    // Write the bindings to $OUT_DIR/bindings.rs.
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    match bindings.write_to_file(out_path.join("bindings.rs")) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to write bindings.rs: {}", e);
            return Err(e);
        }
    };

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper/wrapper.c");
    println!("cargo:rerun-if-changed=wrapper/wrapper.h");
    Ok(())
}
