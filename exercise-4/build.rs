fn main() {
    // code the fuzzer's rpath to include the libqemu-aarch64 shared object instead of
    // dinking around with environment variables et al
    println!("cargo:rustc-link-lib=dylib=qemu-aarch64");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN")
}
