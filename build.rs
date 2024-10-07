extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;
use std::mem;
use libc;

fn add_def(v: &mut Vec<(String, String)>, key: &str, val: &str) {
    v.push((key.to_owned(), val.to_owned()));
}

fn main() {
    let mut defines = Vec::new();
    add_def(&mut defines, "SIZEOF_SIZE_T", mem::size_of::<libc::size_t>().to_string().as_str());
    add_def(&mut defines, "SIZEOF_SIZE_UNSIGNED_INT", mem::size_of::<libc::c_uint>().to_string().as_str());
    add_def(&mut defines, "SIZEOF_SIZE_UNSIGNED_LONG", mem::size_of::<libc::c_ulong>().to_string().as_str());
    add_def(&mut defines, "SIZEOF_SIZE_UNSIGNED_LONG_LONG", mem::size_of::<libc::c_ulonglong>().to_string().as_str());
    add_def(&mut defines, "SECONDARY_DJW", "1");
    add_def(&mut defines, "SECONDARY_FGK", "1");
    add_def(&mut defines, "EXTERNAL_COMPRESSION", "0");
    add_def(&mut defines, "XD3_USE_LARGEFILE64", "1");

    #[cfg(windows)]
    add_def(&mut defines, "XD3_WIN32", "1");
    add_def(&mut defines, "SHELL_TESTS", "0");

    #[cfg(feature = "lzma")]
    {
        add_def(&mut defines, "SECONDARY_LZMA", "1");
        pkg_config::Config::new().probe("liblzma").unwrap();
    }

    {
        let mut builder = cc::Build::new();
        builder.include("xdelta3/xdelta3");
        for (key, val) in &defines {
            builder.define(&key, Some(val.as_str()));
        }

        builder
            .file("xdelta3/xdelta3/xdelta3.c")
            .warnings(false)
            .compile("xdelta3");
    }

    {
        let mut builder = bindgen::Builder::default();

        for (key, val) in &defines {
            builder = builder.clang_arg(format!("-D{}={}", key, val));
        }
        let bindings = builder
            .header("xdelta3/xdelta3/xdelta3.h")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .allowlist_function("xd3_.*")
            .allowlist_type("xd3_.*")
            .rustified_enum("xd3_.*")
            .generate()
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}
