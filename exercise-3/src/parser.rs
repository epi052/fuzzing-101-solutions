use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct FuzzerOptions {
    /// output solutions directory
    #[clap(short, long, default_value = "solutions")]
    pub output: PathBuf,

    /// input corpus directory
    #[clap(short, long, default_value = "corpus", multiple_values = true)]
    pub input: PathBuf,

    /// which cores to bind, i.e. --cores 0 1 2
    #[clap(short, long)]
    pub cores: Vec<usize>,

    /// target binary to execute
    #[clap(short, long, required = true, takes_value = true)]
    pub target: String,

    /// arguments to pass to the target binary
    #[clap(
        short,
        long,
        allow_hyphen_values = true,
        multiple_values = true,
        takes_value = true
    )]
    pub args: Vec<String>,
}

pub fn parse_args() -> FuzzerOptions {
    FuzzerOptions::parse()
}
