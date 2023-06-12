use libafl::bolts::rands::StdRand;
use libafl::bolts::shmem::{ShMem, ShMemProvider, StdShMemProvider};
use libafl::bolts::tuples::tuple_list;
use libafl::bolts::{current_nanos, AsMutSlice};
use libafl::corpus::{Corpus, InMemoryCorpus, OnDiskCorpus};
use libafl::events::{EventRestarter, SimpleEventManager};
use libafl::executors::{ForkserverExecutor, TimeoutForkserverExecutor};
use libafl::feedbacks::{MaxMapFeedback, TimeFeedback, TimeoutFeedback};
use libafl::inputs::BytesInput;
use libafl::monitors::SimpleMonitor;
use libafl::mutators::{havoc_mutations, StdScheduledMutator};
use libafl::observers::{HitcountsMapObserver, StdMapObserver, TimeObserver};
use libafl::schedulers::{IndexesLenTimeMinimizerScheduler, QueueScheduler};
use libafl::stages::StdMutationalStage;
use libafl::state::{HasCorpus, StdState};
use libafl::{feedback_and_fast, feedback_or, Error, Fuzzer, StdFuzzer};
use std::path::PathBuf;
use std::time::Duration;

/// size of the shared memory mapping used as the coverage map
const MAP_SIZE: usize = 65536;

fn main() -> Result<(), Error> {
    //
    // Component: Corpus
    //

    // path to input corpus
    let corpus_dirs = vec![PathBuf::from("./corpus")];

    // Corpus that will be evolved, we keep it in memory for performance
    let input_corpus = InMemoryCorpus::<BytesInput>::new();

    // Corpus in which we store solutions (timeouts/hangs in this example),
    // on disk so the user can get them after stopping the fuzzer
    let timeouts_corpus = OnDiskCorpus::new(PathBuf::from("./timeouts"))?;

    //
    // Component: Observer
    //

    // Create an observation channel to keep track of the current testcase's execution time
    let time_observer = TimeObserver::new("time");

    // Create an observation channel using the coverage map.
    //
    // The ForkserverExecutor gets a pointer to shared memory from the __AFL_SHM_ID environment
    // variable.
    //
    // further explanation from toka: the edges map pointed by __AFL_SHM_ID is inserted by
    // afl-clang-fast, if you use afl-clang-fast, you can use __AFL_SHM_ID to get the ptr to the
    // map

    // The shmem provider supported by AFL++ for shared memory
    let mut shmem_provider = StdShMemProvider::new()?;

    // The coverage map shared between observer and executor
    let mut shmem = shmem_provider.new_shmem(MAP_SIZE)?;

    // let the forkserver know the shmid
    shmem.write_to_env("__AFL_SHM_ID")?;
    let shmem_buf = shmem.as_mut_slice();

    // Create an observation channel using the signals map
    let edges_observer =
        unsafe { HitcountsMapObserver::new(StdMapObserver::new("shared_mem", shmem_buf)) };

    //
    // Component: Feedback
    //

    // A Feedback, in most cases, processes the information reported by one or more observers to
    // decide if the execution is interesting. This one is composed of two Feedbacks using a logical
    // OR.
    //
    // Due to the fact that TimeFeedback can never classify a testcase as interesting on its own,
    // we need to use it alongside some other Feedback that has the ability to perform said
    // classification. These two feedbacks are combined to create a boolean formula, i.e. if the
    // input triggered a new code path, OR, false.
    let mut feedback = feedback_or!(
        // New maximization map feedback (attempts to maximize the map contents) linked to the
        // edges observer. This one will track indexes, but will not track novelties,
        // i.e. new_tracking(... true, false).
        MaxMapFeedback::tracking(&edges_observer, true, false),
        // Time feedback, this one never returns true for is_interesting, However, it does keep
        // track of testcase execution time by way of its TimeObserver
        TimeFeedback::with_observer(&time_observer)
    );

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
    let mut objective =
        feedback_and_fast!(TimeoutFeedback::new(), MaxMapFeedback::new(&edges_observer));

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
    let monitor = SimpleMonitor::new(|s| println!("{s}"));

    //
    // Component: EventManager
    //

    // The event manager handles the various events generated during the fuzzing loop
    // such as the notification of the addition of a new testcase to the corpus.
    // The SimpleEventManager is the simplest event manager available to us.
    let mut mgr = SimpleEventManager::new(monitor);

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
    let mut state = StdState::new(
        // random number generator with a time-based seed
        StdRand::with_seed(current_nanos()),
        input_corpus,
        timeouts_corpus,
        // States of the feedbacks that store the data related to the feedbacks that should be
        // persisted in the State.
        &mut feedback,
        &mut objective,
    )?;

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
    let scheduler = IndexesLenTimeMinimizerScheduler::new(QueueScheduler::new());

    //
    // Component: Fuzzer
    //

    // A fuzzer with feedback, objectives, and a corpus scheduler
    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    //
    // Component: Executor
    //

    // Create an in-process executor. The TimeoutExecutor wraps the InProcessExecutor and sets a
    // timeout before each run. This gives us an executor that will execute a bunch of testcases
    // within the same process, eliminating a lot of the overhead associated with a fork/exec or
    // forkserver execution model.
    let fork_server = ForkserverExecutor::builder()
        .program("./xpdf/install/bin/pdftotext")
        .parse_afl_cmdline(["@@"])
        .coverage_map_size(MAP_SIZE)
        .build(tuple_list!(time_observer, edges_observer))?;

    let timeout = Duration::from_secs(5);

    // wrap the fork server executor and its associated timeout limit
    let mut executor = TimeoutForkserverExecutor::new(fork_server, timeout)?;

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
