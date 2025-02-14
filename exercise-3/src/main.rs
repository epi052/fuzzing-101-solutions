mod parser;

use libafl_bolts::core_affinity::Cores;
use libafl_sugar::ForkserverBytesCoverageSugar;

fn main() {
    let parsed_opts = parser::parse_args();
    let cores = Cores::from_cmdline(&parsed_opts.cores).expect("Failed to parse cores");

    // map size of 98_816 required on my machine
    ForkserverBytesCoverageSugar::builder()
        .input_dirs(&[parsed_opts.input])
        .output_dir(parsed_opts.output)
        .cores(&cores)
        .program(parsed_opts.target)
        .debug_output(parsed_opts.debug)
        .arguments(&parsed_opts.args)
        .build()
        .run()
}
