use libafl_cc::{ClangWrapper, CompilerWrapper};
use std::env;

pub fn main() {
    let cwd = env::current_dir().unwrap();
    let args: Vec<String> = env::args().collect();

    let mut cc = ClangWrapper::new();

    if let Some(code) = cc
        .cpp(false)
        // silence the compiler wrapper output, needed for some configure scripts.
        .silence(true)
        .parse_args(&args)
        .expect("Failed to parse the command line")
        .link_staticlib(&cwd, "exercisetwo")
        .add_arg("-fsanitize-coverage=trace-pc-guard")
        .add_arg("-fsanitize=address")
        .run()
        .expect("Failed to run the wrapped compiler")
    {
        std::process::exit(code);
    }
}
