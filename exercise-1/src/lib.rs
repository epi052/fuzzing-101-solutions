use libafl::bolts::current_nanos;
use libafl::bolts::rands::StdRand;
use libafl::bolts::tuples::tuple_list;
use libafl::corpus::{
    Corpus, InMemoryCorpus, IndexesLenTimeMinimizerCorpusScheduler, OnDiskCorpus,
    QueueCorpusScheduler,
};
use libafl::events::{setup_restarting_mgr_std, EventConfig, EventRestarter};
use libafl::executors::{ExitKind, InProcessExecutor, TimeoutExecutor};
use libafl::feedbacks::{MapFeedbackState, MaxMapFeedback, TimeFeedback, TimeoutFeedback};
use libafl::inputs::{BytesInput, HasTargetBytes};
use libafl::monitors::MultiMonitor;
use libafl::mutators::{havoc_mutations, StdScheduledMutator};
use libafl::observers::{HitcountsMapObserver, StdMapObserver, TimeObserver};
use libafl::stages::StdMutationalStage;
use libafl::state::{HasCorpus, StdState};
use libafl::{feedback_and_fast, feedback_or, Error, Fuzzer, StdFuzzer};
use libafl_targets::{libfuzzer_test_one_input, EDGES_MAP, MAX_EDGES_NUM};
use std::path::PathBuf;
use std::time::Duration;

#[no_mangle]
fn libafl_main() -> Result<(), Error> {
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

    // Create an observation channel using the coverage map.
    //
    // The ForkserverExecutor gets a pointer to shared memory from the __AFL_SHM_ID environment
    // variable, but since this fuzzer now uses an InProcessExecutor, we need to use EDGES_MAP
    // from the coverage module.
    //
    // further explanation from toka: the edges map pointed by __AFL_SHM_ID is inserted by
    // afl-clang-fast, if you use afl-clang-fast, you can use __AFL_SHM_ID to get the ptr to the
    // map, but if you use libafl-cc which uses a sancov backend, you can use EDGES_MAP.
    let edges = unsafe { &mut EDGES_MAP[0..MAX_EDGES_NUM] };
    let edges_observer = HitcountsMapObserver::new(StdMapObserver::new("edges", edges));

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

    // create a new map feedback state with a history map of size EDGES_MAP_SIZE which provides
    // state about the edges feedback for timeouts
    let objective_state = MapFeedbackState::new("timeout_edges", unsafe { EDGES_MAP.len() });

    // A feedback is used to choose if an input should be added to the corpus or not. In the case
    // below, we're saying that in order for a testcase's input to be added to the corpus, it must:
    //   1: be a timeout
    //        AND
    //   2: have created new coverage of the binary under test
    //
    // The goal is to do similar deduplication to what AFL does
    //
    // The feedback_and_fast macro combines the two feedbacks with a fast AND operation, which
    // means only enough feedback functions will be called to know whether or not the objective
    // has been met, i.e. short-circuiting logic.
    let objective = feedback_and_fast!(
        TimeoutFeedback::new(),
        MaxMapFeedback::new(&objective_state, &edges_observer)
    );

    //
    // Component: Monitor
    //

    // MultiMonitor displays cumulative and per-client statistics (used to be named
    // SimpleStats/MultiStats). It uses LLMP for communication between broker / client(s). It
    // displays 2 clients are connected, even when only a single client is active.
    //
    // further explanation from domenukk: The 0th client is the client that opens a network socket
    // and listens for other clients and potentially brokers. It's still a client from llmp's
    // perspective, so it's more or less an implementation detail.
    let monitor = MultiMonitor::new(|s| {
        println!("{}", s);
    });

    //
    // Component: EventManager
    //

    // The event manager handles the various events generated during the fuzzing loop
    // such as the notification of the addition of a new testcase to the corpus
    //
    // The restarting state will spawn the same process again as child, then restart it each
    // time it crashes. One of the reasons we'll want restarting behavior is to essentially 'clean
    // out the bits' from 1000's of old executions of the harness, so we can start with a clean
    // slate.
    let (state, mut mgr) = match setup_restarting_mgr_std(monitor, 1337, EventConfig::AlwaysUnique)
    {
        Ok(res) => res,
        Err(err) => match err {
            Error::ShuttingDown => {
                return Ok(());
            }
            _ => {
                panic!("Failed to setup the restarting manager: {}", err);
            }
        },
    };

    //
    // Component: State
    //

    // Creates a new State, taking ownership of all of the individual components during fuzzing.
    //
    // On the initial pass, setup_restarting_mgr_std returns (None, LlmpRestartingEventManager).
    // On each successive execution (i.e. on a fuzzer restart), it returns the state from the prior
    // run that was saved off in shared memory. The code below handles the initial None value
    // by providing a default StdState. After the first restart, we'll simply unwrap the
    // Some(StdState) returned from the call to setup_restarting_mgr_std
    let mut state = state.unwrap_or_else(|| {
        StdState::new(
            // random number generator with a time-based seed
            StdRand::with_seed(current_nanos()),
            input_corpus,
            timeouts_corpus,
            // States of the feedbacks that store the data related to the feedbacks that should be
            // persisted in the State.
            tuple_list!(feedback_state, objective_state),
        )
    });

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
    // Component: harness
    //

    //
    let mut harness = |input: &BytesInput| {
        let target = input.target_bytes();
        let buffer = target.as_slice();
        libfuzzer_test_one_input(buffer);
        ExitKind::Ok
    };

    //
    // Component: Executor
    //

    // Create an in-process executor. The TimeoutExecutor wraps the InProcessExecutor and sets a
    // timeout before each run. This gives us an executor that will execute a bunch of testcases
    // within the same process, eliminating a lot of the overhead associated with a fork/exec or
    // forkserver execution model.
    let in_proc_executor = InProcessExecutor::new(
        &mut harness,
        tuple_list!(edges_observer, time_observer),
        &mut fuzzer,
        &mut state,
        &mut mgr,
    )
    .unwrap();

    let timeout = Duration::from_millis(5000);

    // wrap in process executor with a timeout
    let mut executor = TimeoutExecutor::new(in_proc_executor, timeout);

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

    fuzzer
        .fuzz_loop_for(&mut stages, &mut executor, &mut state, &mut mgr, 10000)
        .unwrap();

    // Since were using this fuzz_loop_for in a restarting scenario to only run for n iterations
    // before exiting, we need to ensure we call on_restart() and pass it the state. This way, the
    // state will be available in the next, respawned, iteration.
    mgr.on_restart(&mut state).unwrap();

    Ok(())
}
