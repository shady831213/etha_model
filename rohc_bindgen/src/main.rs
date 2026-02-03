extern crate bindgen;
use clap::{App, Arg};
use std::path::{Path, PathBuf};
fn main() {
    let matches = App::new("rohc_bindgen")
        .author("shady83123 <shady831213@126.com>")
        .arg(
            Arg::with_name("inc_dir")
                .index(1)
                .required(true)
                .value_name("inc_dir")
                .validator(|path| {
                    if Path::new(path.as_str()).is_dir() {
                        Ok(())
                    } else {
                        Err(format!("{} is not dir!", path))
                    }
                })
                .help("rohc includes dir"),
        )
        .arg(
            Arg::with_name("output")
                .index(2)
                .required(true)
                .value_name("output")
                .validator(|path| {
                    if Path::new(path.as_str()).is_dir() {
                        Ok(())
                    } else {
                        Err(format!("{} is not dir!", path))
                    }
                })
                .help("rohc_bindings output path"),
        )
        .get_matches();
    let path = matches.value_of("inc_dir").unwrap();
    let out_path = PathBuf::from(matches.value_of("output").unwrap()).join("rohc_bindings.rs");

    bindgen::Builder::default()
        .header("src/rohc_wrapper.h")
        .clang_arg(format!("-I{}", path))
        .detect_include_paths(true)
        .size_t_is_usize(true)
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&out_path)
        .expect("Couldn't write bindings!");
    println!("Gen {} successfully!", out_path.display());
}
