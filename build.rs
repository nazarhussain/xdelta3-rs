extern crate bindgen;
extern crate cc;

use rand::Rng;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn add_def(v: &mut Vec<(String, String)>, key: &str, val: &str) {
    v.push((key.to_owned(), val.to_owned()));
}

fn main() {
    let mut defines = Vec::new();
    for i in &[
        "size_t",
        "unsigned int",
        "unsigned long",
        "unsigned long long",
    ] {
        let def_name = format!("SIZEOF_{}", i.to_uppercase().replace(" ", "_"));
        defines.push((def_name, check_native_size(i)));
    }
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
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .whitelist_function("xd3_.*")
            .whitelist_type("xd3_.*")
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

fn check_native_size(name: &str) -> String {
    let code = format!("#include <stdint.h>\n#include <stdio.h>\nint main() {{printf(\"%lu\", sizeof({})); return 0;}}\n", name);

    return execute_c_code(code);
}

fn execute_c_code(code: String) -> String {
    let compiler = cc::Build::new().get_compiler();
    let output_dir = &env::var("OUT_DIR").unwrap();
    let key = rand::thread_rng().gen::<i32>();

    let src_path = String::from(
        Path::new(output_dir)
            .join(format!("src-{}.c", key))
            .to_str()
            .expect(&format!("Can not compute the src path for src-{}.c", key)),
    );

    let output_path = String::from(
        Path::new(output_dir)
            .join(format!("out-{}", key))
            .to_str()
            .expect(&format!("Can not compute the out path for out-{}", key)),
    );

    File::create(&src_path)
        .expect(&format!("Can not create src file {}", src_path))
        .write_all(code.as_bytes())
        .expect(&format!("Can not write src file {}", src_path));

    #[cfg(windows)]
    let output_path = format!("{}.exe", output_path);

    let mut compile_cmd = Command::new(compiler.path().as_os_str());

    if compiler.is_like_msvc() {
        compile_cmd.args(&[&src_path, &format!("/Fe{}", output_path)]);
    } else {
        compile_cmd.args(&[&src_path, "-o", &output_path]);
    }

    compile_cmd
        .output()
        .expect(&format!("Can not compile {}", &src_path));

    let mut command = Command::new(&output_path);
    let output = command
        .output()
        .expect("Error executing get-native-sizes-binary")
        .stdout;
    let output = String::from_utf8(output).expect("Error converting Unicode sequence");

    return output;
}
