use std::env;
use std::path::PathBuf;
fn main() {
    //only for tests and gen
    if env::var("CARGO_FEATURE_ROHC").is_ok() && std::env::var("PROFILE").unwrap() == "debug" {
        let lib_path = PathBuf::from(format!("examples/rohc/lib/{}", env::var("TARGET").unwrap()));
        println!("cargo:rustc-link-search=native={}", lib_path.display());
        println!("cargo:rustc-link-lib=static=rohc");
        println!(
            "cargo:rerun-if-changed={}",
            lib_path.join("librohc.a").display()
        );
        println!("cargo:rerun-if-changed=src/rohc/trace_callback.c");
        cc::Build::new()
            .file("src/rohc/trace_callback.c")
            .flag("-Wno-unused-parameter")
            .compile("trace_callback");
    }
}
