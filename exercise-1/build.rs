use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let cwd = env::current_dir().unwrap().to_string_lossy().to_string();
    let xpdf_dir = format!("{}/xpdf", cwd);

    // make clean; remove any leftover gunk from prior builds
    Command::new("make")
        .arg("clean")
        .current_dir(&xpdf_dir)
        .status()
        .expect("Couldn't clean xpdf directory");

    // clean doesn't know about the built-with-* directories we use to build, remove them as well
    Command::new("rm")
        .arg("-r")
        .arg("-f")
        .arg(&format!("{}/built-with-lto", xpdf_dir))
        .arg(&format!("{}/built-with-fast", xpdf_dir))
        .current_dir(&xpdf_dir)
        .status()
        .expect("Couldn't clean xpdf's built-with-* directories");

    // export LLVM_CONFIG=llvm-config-11
    env::set_var("LLVM_CONFIG", "llvm-config-11");

    for (build_dir, compiler) in [("fast", "afl-clang-fast"), ("lto", "afl-clang-lto")] {
        // configure with `compiler` and set install directory to ./xpdf/built-with-`build_dir`
        Command::new("./configure")
            .arg(&format!("--prefix={}/built-with-{}", xpdf_dir, build_dir))
            .env("CC", format!("/usr/local/bin/{}", compiler))
            .env("CXX", format!("/usr/local/bin/{}++", compiler))
            .current_dir(&xpdf_dir)
            .status()
            .expect(&format!(
                "Couldn't configure xpdf to build using afl-clang-{}",
                compiler
            ));

        // make && make install
        Command::new("make")
            .current_dir(&xpdf_dir)
            .status()
            .expect("Couldn't make xpdf");

        Command::new("make")
            .arg("install")
            .current_dir(&xpdf_dir)
            .status()
            .expect("Couldn't install xpdf");
    }
}
