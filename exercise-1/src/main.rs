use libafl::bolts::current_nanos;
use libafl::bolts::rands::StdRand;
use libafl::bolts::shmem::{ShMem, ShMemProvider, StdShMemProvider};
use libafl::bolts::tuples::tuple_list;
use libafl::corpus::{
    Corpus, InMemoryCorpus, IndexesLenTimeMinimizerCorpusScheduler, OnDiskCorpus,
    QueueCorpusScheduler,
};
use libafl::events::SimpleEventManager;
use libafl::executors::{ForkserverExecutor, TimeoutForkserverExecutor};
use libafl::feedbacks::{MapFeedbackState, MaxMapFeedback, TimeFeedback, TimeoutFeedback};
use libafl::inputs::BytesInput;
use libafl::mutators::{havoc_mutations, StdScheduledMutator};
use libafl::observers::{ConstMapObserver, HitcountsMapObserver, TimeObserver};
use libafl::stages::StdMutationalStage;
use libafl::state::{HasCorpus, StdState};
use libafl::stats::SimpleStats;
use libafl::{feedback_and_fast, feedback_or, Fuzzer, StdFuzzer};
use std::path::PathBuf;
use std::time::Duration;

/// Size of coverage map shared between observer and executor
const MAP_SIZE: usize = 65536;

fn main() {
    //
    // Component: Corpus
    //

    // path to input corpus
    let corpus_dirs = vec![PathBuf::from("./corpus")];

    // Corpus that will be evolved, we keep it in memory for performance
    let input_corpus = InMemoryCorpus::<BytesInput>::new();

    // Corpus in which we store solutions (timeouts/hangs in this example),
    // on disk so the user can get them after stopping the fuzzer
    let timeouts_corpus =
        OnDiskCorpus::new(PathBuf::from("./timeouts")).expect("Could not create timeouts corpus");

    //
    // Component: Observer
    //

    // A Shared Memory Provider which uses `shmget`/`shmat`/`shmctl` to provide shared
    // memory mappings. The provider is used to ... provide ... a coverage map that is then
    // shared between the Observer and the Executor
    let mut shmem = StdShMemProvider::new().unwrap().new_map(MAP_SIZE).unwrap();

    // save the shared memory id to the environment, so that the forkserver knows about it; the
    // ShMemId is populated as part of the implementor of the ShMem trait
    shmem
        .write_to_env("__AFL_SHM_ID")
        .expect("couldn't write shared memory ID");

    // this is the actual shared map, as a &mut [u8]
    let mut shmem_map = shmem.map_mut();

    // Create an observation channel using the coverage map; since MAP_SIZE is known at compile
    // time, we can use ConstMapObserver to speed up Feedback::is_interesting
    let edges_observer = HitcountsMapObserver::new(ConstMapObserver::<_, MAP_SIZE>::new(
        "shared_mem",
        &mut shmem_map,
    ));

    // Create an observation channel to keep track of the execution time and previous runtime
    let time_observer = TimeObserver::new("time");

    //
    // Component: Feedback
    //

    // This is the state of the data that the feedback wants to persist in the fuzzers's state. In
    // particular, it is the cumulative map holding all the edges seen so far that is used to track
    // edge coverage.
    let feedback_state = MapFeedbackState::with_observer(&edges_observer);

    // A Feedback, in most cases, processes the information reported by one or more observers to
    // decide if the execution is interesting. This one is composed of two Feedbacks using a logical
    // OR.
    //
    // Due to the fact that TimeFeedback can never classify a testcase as interesting on its own,
    // we need to use it alongside some other Feedback that has the ability to perform said
    // classification. These two feedbacks are combined to create a boolean formula, i.e. if the
    // input triggered a new code path, OR, false.
    let feedback = feedback_or!(
        // New maximization map feedback (attempts to maximize the map contents) linked to the
        // edges observer and the feedback state. This one will track indexes, but will not track
        // novelties, i.e. new_tracking(... true, false).
        MaxMapFeedback::new_tracking(&feedback_state, &edges_observer, true, false),
        // Time feedback, this one does not need a feedback state, nor does it ever return true for
        // is_interesting, However, it does keep track of testcase execution time by way of its
        // TimeObserver
        TimeFeedback::new_with_observer(&time_observer)
    );

    // create a new map feedback state with a history map of size MAP_SIZE which provides state
    // about the edges feedback for timeouts
    let objective_state = MapFeedbackState::new("timeout_edges", MAP_SIZE);

    // A feedback is used to choose if an input should be added to the corpus or not. In the case
    // below, we're saying that in order for a testcase's input to be added to the corpus, it must:
    //   1: be a timeout
    //        AND
    //   2: have created new coverage of the binary under test
    //
    // The goal is to do similar deduplication to what AFL does
    let objective = feedback_and_fast!(
        // A TimeoutFeedback reports as "interesting" if the exits via a Timeout
        TimeoutFeedback::new(),
        // Combined with the requirement for new coverage over timeouts
        MaxMapFeedback::new(&objective_state, &edges_observer)
    );

    //
    // Component: State
    //

    // Creates a new State, taking ownership of all of the individual components during fuzzing
    let mut state = StdState::new(
        // random number generator with a time-based seed
        StdRand::with_seed(current_nanos()),
        input_corpus,
        timeouts_corpus,
        // States of the feedbacks that store the data related to the feedbacks that should be
        // persisted in the State.
        tuple_list!(feedback_state, objective_state),
    );

    //
    // Component: Stats
    //

    // call println with SimpleStats::display as input to report to the terminal. introspection
    // feature flag can be added for additional stats
    let stats = SimpleStats::new(|s| println!("{}", s));

    //
    // Component: EventManager
    //

    // The event manager handles the various events generated during the fuzzing loop
    // such as the notification of the addition of a new testcase to the corpus
    let mut mgr = SimpleEventManager::new(stats);

    //
    // Component: Scheduler
    //

    // A minimization + queue policy to get test cases from the corpus
    //
    // IndexesLenTimeMinimizerCorpusScheduler is a MinimizerCorpusScheduler with a
    // LenTimeMulFavFactor that prioritizes quick and small Testcases that exercise all the
    // entries registered in the MapIndexesMetadata
    //
    // a QueueCorpusScheduler walks the corpus in a queue-like fashion
    let scheduler = IndexesLenTimeMinimizerCorpusScheduler::new(QueueCorpusScheduler::new());

    //
    // Component: Fuzzer
    //

    // A fuzzer with feedback, objectives, and a corpus scheduler
    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    //
    // Component: Executor
    //

    // Create the executor for the forkserver. The TimeoutForkserverExecutor wraps the standard
    // ForkserverExecutor and sets a timeout before each run. This gives us an executor
    // that implements an AFL-like mechanism that will spawn child processes to fuzz
    let fork_server = ForkserverExecutor::new(
        "./xpdf/install/bin/pdftotext".to_string(),
        &[String::from("@@")],
        // we're passing testcases via on-disk file; set to use_shmem_testcase to false
        false,
        tuple_list!(edges_observer, time_observer),
    ).unwrap();

    let timeout = Duration::from_millis(5000);

    // ./pdftotext @@
    let mut executor = TimeoutForkserverExecutor::new(fork_server, timeout)
        .unwrap();

    // In case the corpus is empty (i.e. on first run), load existing test cases from on-disk
    // corpus
    if state.corpus().count() < 1 {
        state
            .load_initial_inputs(&mut fuzzer, &mut executor, &mut mgr, &corpus_dirs)
            .unwrap_or_else(|err| {
                panic!(
                    "Failed to load initial corpus at {:?}: {:?}",
                    &corpus_dirs, err
                )
            });
        println!("We imported {} inputs from disk.", state.corpus().count());
    }

    //
    // Component: Mutator
    //

    // Setup a mutational stage with a basic bytes mutator
    let mutator = StdScheduledMutator::new(havoc_mutations());

    //
    // Component: Stage
    //

    let mut stages = tuple_list!(StdMutationalStage::new(mutator));

    // start the fuzzing
    fuzzer
        .fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)
        .expect("Error in the fuzzing loop");
}
