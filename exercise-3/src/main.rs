mod parser;

use libafl::bolts::core_affinity::Cores;
use libafl_sugar::ForkserverBytesCoverageSugar;

fn main() {
    let parsed_opts = parser::parse_args();
    let cores = Cores::from_cmdline(&parsed_opts.cores).expect("Failed to parse cores");

    ForkserverBytesCoverageSugar::<80642>::builder()
        .input_dirs(&[parsed_opts.input])
        .output_dir(parsed_opts.output)
        .cores(&cores)
        .program(parsed_opts.target)
        .debug_output(parsed_opts.debug)
        .arguments(&parsed_opts.args)
        .build()
        .run()
}
