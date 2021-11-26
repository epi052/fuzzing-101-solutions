mod parser;

use libafl_sugar::ForkserverBytesCoverageSugar;

fn main() {
    let parsed_opts = parser::parse_args();

    ForkserverBytesCoverageSugar::builder()
        .input_dirs(&[parsed_opts.input])
        .output_dir(parsed_opts.output)
        .cores(&parsed_opts.cores)
        .program(parsed_opts.target)
        .arguments(&parsed_opts.args)
        .build()
        .run()
}
