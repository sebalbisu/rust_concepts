/*
# Future vs Task:
=============================================================
    Future: a value that will become ready at some point
    in the future.
    Task: a future that is currently executing in a thread.
*/

/*
# Data races across threads:
=============================================================

One thread: (no data races)
    Two tasks on the same thread cannot run at the same time,
    because Tokio’s scheduler runs tasks one at a time and only
    switches tasks when it hits an .await, so there are no data
    races.
    Send/Sync is not required.

Two threads, same core: (data races are possible)
    Data races can occur because the CPU may switch threads and
    advance another task at any time, so access to data can be
    concurrent. It interleaves task progress over time to
    simulate real parallelism, but it is still concurrency.
    Send/Sync is required.

Two threads, different cores: (data races are possible)
    Distinct cores run fully independently and can execute work
    in parallel, so data races can occur.
    Send/Sync is required.
*/

/*
# Cores and threads
=============================================================
    * The OS scheduler decides which CPU core a thread runs on;
    it may migrate the thread later—execution order is not
    guaranteed.
    * If two threads are scheduled on the same core they do not
    run in true parallel, only concurrently.
*/

/*
JoinHandle vs Future:
=============================================================

JoinHandle
For both threads and tasks, runs the work as soon as it is
    created; you can wait for the result later with .join() or
    .await.

A Future
does not run on its own; it only makes progress when you poll
    it (usually via .await), so it only runs when execution
    reaches that point.
*/
#[cfg(test)]
pub mod join_handle_vs_future {
    use std::thread;

    use std::thread::JoinHandle;

    #[test]
    pub fn join_handle() {
        let handle: JoinHandle<&str> = thread::spawn(|| "hello"); // dispatched
        let result = handle.join().unwrap(); // waiting response
        assert_eq!(result, "hello");
    }

    #[tokio::test]
    pub async fn future() {
        let future = async { "hello" }; // created, not dispatched yet
        let result = future.await; // running and waiting response
        assert_eq!(result, "hello");
    }
}

/*
# Spawn
=============================================================
Spawn returns a JoinHandle.

For both threads and tasks, it creates and runs a new
    thread/task immediately; you wait for the result elsewhere
    with .join() or .await.
*/
#[cfg(test)]
pub mod spawn {
    use std::thread;

    #[test]
    pub fn thread() {
        use std::thread::JoinHandle;
        let handle: JoinHandle<&str> = thread::spawn(|| "hello"); // dispatched
        let result = handle.join().unwrap(); // waiting response
        assert_eq!(result, "hello");
    }

    #[tokio::test]
    pub async fn task() {
        use tokio::task::JoinHandle;
        let handle: JoinHandle<&str> = tokio::spawn(async { "hello" }); // dispatched
        let result = handle.await.unwrap(); // waiting response
        assert_eq!(result, "hello");
    }
}

/*
# Waiting for tasks:
=============================================================
- await:
    Sequential execution order:
    One task finishes, then the next starts.
- join!:
    Tasks run in parallel and you wait until all complete.
- select!:
    Waits until the first task completes, then drops the others.
*/
#[cfg(test)]
pub mod waiting_for_tasks {

    use tokio::join;
    use tokio::select;

    /*
    #await:
    =============================================================
    Sequential execution order:
    One task finishes, then the next starts.
     */
    #[tokio::test]
    pub async fn _await() {
        let task1 = tokio::spawn(async { 1 });
        let task2 = tokio::spawn(async { 2 });

        // sequential execution: task1 completes first, then task2 starts
        let _r1 = task1.await.unwrap();
        let _r2 = task2.await.unwrap();

        let task3 = tokio::spawn(async { 3 });
        let task4 = tokio::spawn(async { 4 });

        // sequential execution: task3 completes first, then task4 starts
        let (_r3, _r4) = join!(task3, task4);
    }

    /*
    #join!:
    =============================================================
    Tasks run in parallel; you wait until all finish.
     */
    #[tokio::test]
    pub async fn _join() {
        let task1 = tokio::spawn(async { 1 });
        let task2 = tokio::spawn(async { 2 });

        let (_r1, _r2) = join!(task1, task2);
    }

    /*
    #select!:
    =============================================================
    Waits until the first task completes, then drops the other tasks.
    */
    #[tokio::test]
    pub async fn _select() {
        use tokio::time::{Duration, sleep};

        let task1 = tokio::spawn(async {
            sleep(Duration::from_millis(10)).await;
            1
        });
        let task2 = tokio::spawn(async { 2 });

        let _result = select! {
            _r1 = task1 => _r1,
            _r2 = task2 => _r2
        };
        assert!(_result.is_ok());
        assert!(_result.unwrap() == 2);
    }

    /*
    #select_loop pattern:
    =============================================================
    Processes messages as they arrive until the channel closes.
    */
    use tokio::sync::mpsc;
    #[tokio::test]
    pub async fn _select_loop_pattern() {
        let cap = 2;
        let (tx1, mut rx1) = mpsc::channel::<i32>(cap);
        let (tx2, mut rx2) = mpsc::channel::<i32>(cap);

        tokio::spawn(async move {
            for i in (1..5).step_by(2) {
                tx1.send(i).await.unwrap(); // odd
            }
        });
        tokio::spawn(async move {
            for i in (0..5).step_by(2) {
                tx2.send(i).await.unwrap(); // even
            }
        });

        let mut _closed = false;
        loop {
            select! {
                Some(v) = rx1.recv() => {
                    assert_eq!(v % 2, 1); // odd
                }
                Some(v) = rx2.recv() => {
                    assert_eq!(v % 2, 0); // even
                }
                // Pattern: exit condition (both channels closed)
                else => {
                    _closed = true;
                    break;
                }
            }
        }
        assert!(_closed == true);
    }

    /*

    #select loop priority handlers: (biased)
    =============================================================
    biased;
    By default, tokio::select! picks a branch at random when
    several are ready at the same time to avoid starvation.
    Sometimes you want to prioritize a branch (for example, a
    "stop" or high-priority channel).
    */
    #[tokio::test]
    pub async fn select_loop_biased_pattern() {
        let (_normal_tx, mut normal_rx) = tokio::sync::mpsc::channel::<String>(10);
        let (_priority_tx, mut priority_rx) = tokio::sync::mpsc::channel::<String>(10);
        let (_shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(10);
        loop {
            tokio::select! {
                biased; // <--- Enforce top-to-bottom branch order
                _ = shutdown_rx.recv() => {
                    // If there is a shutdown signal, it is ALWAYS handled first
                    break;
                }
                Some(_msg) = priority_rx.recv() => {
                    //
                }
                Some(_msg) = normal_rx.recv() => {
                    //
                }
                else => {
                    // All channels are closed
                    break;
                }
            }
        }
    }
}

/*
# thread_local!:
=============================================================

Usually `static` variables are shared across all threads. With
    `thread_local!`, you can define variables that are `static` in name
    but per-thread: each thread has its own independent copy.

* Each new thread can use the variable defined with `thread_local!`
* That variable is initialized per new thread and is in scope
    for the whole thread (functions, etc.) and is not shared
    across threads.
* Changing the variable only affects that thread, not others.
* Unlike `task_local!`, which is initialized once when the
    scope is created and persists across task calls inside that
    scope.
* For mutability use `Cell`, `RefCell`.
 */
#[cfg(test)]
pub mod thread_local {

    #[test]
    pub fn test() {
        use std::cell::Cell;
        use std::thread;

        thread_local! {
            static COUNTER: Cell<u32> = Cell::new(0);
        }

        fn increment_counter() {
            COUNTER.with(|c| {
                c.set(c.get() + 1); // only affects this thread, not others.
            });
        }

        let handle1 = thread::spawn(|| {
            increment_counter();
            increment_counter();

            COUNTER.with(|c| {
                println!("thread 1 counter = {}", c.get());
                assert_eq!(c.get(), 2);
            });

            let handle3 = thread::spawn(|| {
                COUNTER.with(|c| {
                    println!("thread 3 counter = {}", c.get());
                });
            });
            handle3.join().unwrap();
        });

        let handle2 = thread::spawn(|| {
            increment_counter();

            COUNTER.with(|c| {
                println!("thread 2 counter = {}", c.get());
                assert_eq!(c.get(), 1);
            });
        });

        handle1.join().unwrap();
        handle2.join().unwrap();
    }
}

/*
task_local!:
    Shares local static-like variables across tasks on the same thread.
    No `Sync`/`Send` required; a scheduler must be running.

    Use it to store values reachable from all tasks inside that scope
    without passing the value into every task.
    Further task calls from within a scope run on the same thread, so
    there are no cross-thread races; `Cell` and `RefCell` suffice.
    The static value is initialized once when the scope is created
    and persists across task calls inside that scope.
    Unlike a plain `static`, the value is not shared across all
    threads/tasks—only tasks using the scope created with `.scope()`.
    For mutability use `Cell`, `RefCell`.
 */
#[tokio::test]
pub async fn task_local() {
    use std::cell::Cell;

    tokio::task_local! {
        static COUNTER: Cell<i32>;
    }
    async fn increment_counter() {
        COUNTER.with(|c| {
            c.set(c.get() + 1); // mutate value
        });
    }

    let a = 123;
    let b = &a;
    println!("a: {}, b: {}", a, b);

    COUNTER
        .scope(Cell::new(0), async {
            increment_counter().await;
            increment_counter().await;
            COUNTER.with(|c| {
                println!("counter = {}", c.get()); // read value
            });
        })
        .await;
}

/*
LocalSet
=============================================================

Runs tasks (futures) on the same thread; they need not implement
    `Send` because they stay on one thread.

- Use `run_until(...)` or `block_on(...)` for the main task.
- Use `spawn_local` to spawn subtasks inside that main work.

* run_until:
    1. Creates a future.
    2. Runs it and obtains the result in a later step with `.await`.
* block_on: Blocks the current thread until the provided future
    completes; useful in synchronous contexts.
* spawn_local: used inside `block_on` / `run_until`
    1. Runs the future immediately.
    2. Await the result with `.await`, like `tokio::spawn`.

Characteristics:
- All tasks in a `LocalSet` always run on the same thread, avoiding
    cross-thread concurrency issues.
- Allows `spawn_local` for local tasks.
- Good for async code that uses non-`Send` structures.
- Used with `run_until` (async) or `block_on` (sync), which drive
    local tasks and return when done.
- Especially relevant when `Send`/`Sync` limits other concurrency
    patterns.
*/
#[cfg(test)]
pub mod local_set {
    use tokio::task::LocalSet;

    /*
    # run_until:
    =============================================================
    1. Create a future.
    2. Run it and get the result in a later step with `.await`.

    spawn_local: subtasks
     */
    #[tokio::test]
    pub async fn run_until() {
        let local_set = LocalSet::new();

        let future_local = local_set.run_until(async {
            let mut handles = vec![];
            // sub tasks
            for i in 0..3 {
                handles.push(local_set.spawn_local(async move { i }));
            }
            let mut results_sub_tasks = vec![];
            for handle in handles {
                results_sub_tasks.push(handle.await.unwrap());
            }
            results_sub_tasks
        });
        // created future but not executed yet

        //...

        let response = future_local.await; // executed, and fetch response
        assert!(response.len() == 3);
    }

    /*
    # block_on:
    =============================================================
    1. Runs and blocks until the future completes and returns its
    value.
    2. On the same thread that called `block_on`.
     */
    #[test]
    pub fn block_on() {
        let local_set = LocalSet::new();
        let rt = tokio::runtime::Runtime::new().unwrap();

        // Runs and blocks until the future completes and returns its value
        // on the same thread that called block_on
        let result = local_set.block_on(&rt, async { 123 });

        // block_on blocks until the future completes and returns its value
        assert_eq!(result, 123); // current_thread: everything runs on the
        // same thread
    }
}

/*
# spawn_blocking:
=============================================================
    Runs blocking/expensive code on a thread separate from the
    runtime and worker threads.
    The `spawn_blocking` callback cannot be `async` directly—
    only synchronous code. The idea is to move heavy synchronous
    work off the async runtime. If you need `async`, use
    `block_on` inside.
*/
#[cfg(test)]
mod spawn_blocking {
    use crate::concepts::concurrent_patterns::utils::get_thread_id_number;
    use std::thread;
    use std::thread::ThreadId;
    use tokio::task::spawn_blocking;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    pub async fn test() {
        // main is in one thread, 4 workers are in other threads, blocking
        // operation runs in a different thread
        let thread_id: ThreadId = thread::current().id();
        // starts running here
        let handle = spawn_blocking(|| thread::current().id());

        // wait for the blocking operation to finish
        let value = handle.await.unwrap();

        assert!(value != thread_id);
        assert!(get_thread_id_number(value) >= get_thread_id_number(thread_id) + 4);
    }
}

/*
# JoinSet:
=============================================================
`JoinSet` is a Tokio type for running and managing many async
    tasks (futures) at once, efficiently and safely.

- You can keep spawning tasks and then collect their results
    (join) in order or as they finish.
- Unlike `join_all`, which waits for every task, `join_next`
    waits for the next task to finish and returns it.

For threads, `mpsc` channels are used to collect results in
    completion order.
*/

#[cfg(test)]
pub mod join_set {
    use std::time::Duration;
    use tokio::task::JoinSet;
    use tokio::time::sleep;

    #[tokio::test]
    async fn joinset_example() {
        let mut set = JoinSet::new();

        let order = vec![3, 1, 2];
        let keys_order = vec![1, 2, 0];

        // Spawn several async tasks
        for i in 0..3 {
            let order = order.clone();
            set.spawn(async move {
                // Simulate async work
                sleep(Duration::from_millis(order[i] * 10)).await;
                i
            });
        }

        // Collect results in completion order
        let mut keys_order_iter = keys_order.iter();
        while let Some(res) = set.join_next().await {
            assert_eq!(res.unwrap(), *keys_order_iter.next().unwrap());
        }
    }
}

/*
# multi_thread:
=============================================================
    The scheduler decides which thread runs each task.
    Running multithread in a set
 */
#[cfg(test)]
pub mod multi_thread {
    use std::collections::HashSet;
    use std::thread::ThreadId;
    use tokio::task::{JoinSet, yield_now};
    /**
    The scheduler decides which thread to run the tasks on.
    Running multithread in a set
     */
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    pub async fn scheduler_decides() {
        let mut set = JoinSet::new();
        for _ in 0..10 {
            set.spawn(async {
                yield_now().await;
                std::thread::current().id()
            });
        }
        let results = set.join_all().await;
        let threads_id_set: HashSet<ThreadId> = results.into_iter().collect();
        println!("threads_id_set: {:?}", threads_id_set);
        assert!(1 < threads_id_set.len() && threads_id_set.len() <= 4);
    }
}

/*
Ownership, inmutable ref, exclusive mut ref
=============================================================
Model:
The short answer is: there is no garbage collector and safety is
    required, so the compiler must control when a variable or an
    associated reference can be dropped. It also helps the
    optimizer apply aggressive optimizations.

Owner:
The compiler applies aggressive optimizations; the generated code
    is not the “original” source—it takes shortcuts and
    optimization tricks.
For optimization to be both effective and safe, simple strict
    rules are needed.
The owner controls when the variable is dropped.
References are only accesses; they do not drop the variable.

A single exclusive `mut` ref:
If several `mut` refs were allowed, the optimizer might use
    cached values instead of the real memory,
which could disagree with the original memory and cause data
    races across different variables.
With a single `mut` ref there is exclusive access to memory,
    avoiding data races.
Every read happens before the write, so stale values are not used.

Lifetimes:
A lifetime is how long a reference is valid. Without a GC, this
    is how the compiler knows a reference never outlives the data
    it points to. That prevents use-after-free deterministically
    at compile time.

Move:
When a value is moved, the original binding is no longer valid.
    That prevents duplicated ownership, double frees, and
    invalidates associated references for that lifetime.

Send and Sync for concurrent safety:
`Send` and `Sync` help guarantee safe concurrency without a garbage
    collector.

Interior mutability (`Cell`, `RefCell`):
Lets you mutate internal state without making the whole struct
    `&mut`.

unsafe:
Allows code the compiler cannot verify—whether or not it is
    actually safe.
*/

/*
Raw Pointers:
=============================================================
A raw pointer (`*mut T`)
    * Skips borrow-checker rules.
    * You can have multiple reads/writes to the same data without
    exclusivity and manipulate memory directly.

Read-only raw pointer (`*const T`):
    * Accesses data outside the borrow checker (no lifetime or
    exclusivity rules).
    * May be null.
    * May point to invalid memory, unlike `&T`, which is
    compile-time checked.

null:
Setting a pointer to null points it at address 0x0; it does not
    clear the old memory.
If several pointers alias the same memory, nulling one does not
    affect the others.
*/

/*
Interior Mutability:
=============================================================
Used to change internal state without requiring the whole struct
    to be mutable.
Classic example: caches, internal counters, or lazy init in
    methods that only take `&self`.
*/
#[cfg(test)]
pub mod interior_mutability {
    use std::cell::Cell;

    #[test]
    pub fn example() {
        struct AppConfig {
            pub debug_mode_enabled: Cell<bool>, // mutate internal state through
            // non-mutable binding
            pub _max_connections: u32,
            // ... other settings
        }
        impl AppConfig {
            fn new(debug: bool, max_conn: u32) -> Self {
                AppConfig {
                    debug_mode_enabled: Cell::new(debug),
                    _max_connections: max_conn,
                }
            }
            // Method only needs `&self` but can still flip debug_mode_enabled.
            fn toggle_debug_mode(&self) {
                let current_state = self.debug_mode_enabled.get(); // copy of current
                // value
                self.debug_mode_enabled.set(!current_state); // replace with new value
                println!("Debug mode now: {}", self.debug_mode_enabled.get());
            }
            fn check_mode(&self) {
                println!("Current debug mode: {}", self.debug_mode_enabled.get());
            }
        }

        let config = AppConfig::new(true, 100); // immutable config, mutable
        // debug_mode_enabled inside
        config.toggle_debug_mode();
        config.check_mode();
    }
}

/*
UnsafeCell:
=============================================================
UnsafeCell<T> : interior mutability by raw pointer (unsafe).

`T` can wrap any type; `T` need not be `Copy`.
It is the most primitive interior-mutability wrapper.

`.get()` -> raw pointer
    * Read/write from multiple accesses.
    * Escapes lifetime rules.
    * Does not verify memory validity.
    * May be null.
    * FFI interoperability.
`.get_mut()` -> exclusive `&mut` (safe API if `.get()` was not
    used before)

*/

#[cfg(test)]
pub mod unsafe_cell {
    use std::cell::UnsafeCell;
    #[test]
    pub fn simple_unsafe_cell_example() {
        let my_value = UnsafeCell::new(0);

        // Raw mutable pointer from an immutable reference to the cell.
        // Several pointers to the same cell are allowed because they are raw.
        let ptr = my_value.get();
        let ptr2 = my_value.get();

        // Requires `unsafe` because the compiler cannot prove this mutation
        // is safe:
        // dangerous code; the optimizer cannot rule out data races on a
        // single thread
        // if each variable’s value were cached separately, for example.
        // Also allows one pointer to be null and another not.
        unsafe {
            *ptr += 1;
            *ptr2 += 1;
        }

        // To read the value we also dereference the raw pointer
        let final_value = unsafe { *ptr };
        println!("Value after mutation: {}", final_value);
        assert_eq!(final_value, 2);
    }

    #[test]
    pub fn unsafe_cell_aliasing_mutation() {
        // Mutable reference with ownership or `&mut` to the cell
        let mut another_value = UnsafeCell::new(20);
        let x = another_value.get(); // can get pointers with .get() first
        let mut_ref = another_value.get_mut(); // has to be declared as mutable
        // let mut_ref2 = another_value.get_mut(); // error, exclusividad
        // let ref3 = another_value.get(); // error, exclusividad
        *mut_ref += 5;
        println!("x: {}", unsafe { *x });
        println!("Other value after mutation: {}", *mut_ref);
        assert_eq!(*mut_ref, 25);
    }
}

/*
Cell:
=============================================================
Cell<T> : interior mutability by copy.

T must be Copy.
    * One exclusive `mut` ref at a time.
    * Each `get()` returns an independent copy of the cell’s value.
    * `set(value)` sets the value without needing the current
    value first.

`as_ptr()`: `Cell` also supports unsafe raw-pointer access.
*/
#[cfg(test)]
pub mod refcell {
    use std::cell::Cell;

    #[test]
    pub fn cell_mut_and_copy() {
        // let _cell_string = Cell::new(String::from("hello"));
        // let bad = _cell_string.get(); // error, not trait Copy on String

        let mut cell = Cell::new(0);
        let mut a = cell.get(); // copy
        let b = cell.get(); // copy
        let x = cell.get_mut(); // mut ref
        // let y = cell.get(); // error, exclusividad
        *x += 2;
        a += 1;
        assert_eq!(a, 1); // because is an independent copy
        assert_eq!(b, 0); // because is an independent copy
        assert_eq!(cell.get(), 2);

        // set the value, by copy
        cell.set(3);
        assert_eq!(cell.get(), 3);
    }

    // Read/write via raw pointers (unsafe)
    #[test]
    pub fn cell_unsafe_example() {
        let cell = Cell::new(0);
        let ptr = cell.as_ptr();
        unsafe {
            *ptr += 1;
        }
        assert_eq!(cell.get(), 1);
    }
}

/*
RefCell:
=============================================================
RefCell<T> : interior mutability by reference

`T` can be any type; `T` need not be `Copy`.

`RefCell` enforces borrowing rules at runtime (tracks borrows), not
    only at compile time.
Rust may compile overlapping borrows through `RefCell`, but it checks
    at runtime and panics on violation—so it compiles but can panic
    at runtime.

`RefCell` helps when static rules are too strict (e.g. graphs with
    cycles, shared APIs with interior mutability). Without `RefCell`,
    some designs need `unsafe`. Escaping static rules is fine when
    the pattern is well understood.

`RefCell` uses a runtime borrow counter because the compiler cannot
    track these borrows statically here.

borrow(): returns an immutable reference to the inner value, through
    an smart pointer Ref<T>.
borrow_mut(): returns a mutable reference to the inner value.

as_ptr(): returns a raw pointer to the inner value. (unsafe)
*/
#[cfg(test)]
pub mod refcell_test {
    use std::cell::RefCell;
    #[test]
    pub fn refcell_borrow() {
        let refcell1 = RefCell::new(String::from("hello"));
        let a = refcell1.borrow();
        let b = refcell1.borrow();
        assert_eq!(*a, *b);
    }

    #[test]
    pub fn refcell_borrow_mut() {
        let refcell1 = RefCell::new(String::from("hello"));
        // let a = refcell1.borrow(); // error runtime, but compiles
        let mut x = refcell1.borrow_mut();
        // let b = refcell1.borrow(); // error runtime, but compiles
        *x += " world";
        // assert_eq!(*a, *b);
        assert_eq!(*x, "hello world");
    }
}

/*
Atomic Values
=============================================================
Besides `AtomicUsize` and `AtomicIsize`, there are:

Types:
AtomicBool
AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicU8, AtomicU16,
    AtomicU32, AtomicU64

For all atomic methods, if another thread is using the variable,
    the current thread waits until the other finishes.
E.g. `.store()` waits if another thread is reading.

Speed: atomic vs non-atomic:
Atomic ops need CPU-level coordination for cross-thread exclusivity,
    including memory barriers and cross-core sync, so each
    increment/decrement costs more. In simple benchmarks, incrementing
    an `AtomicUsize` is often 2–10× slower than a plain `usize`.
    Atomic methods are slower than non-atomic ones.
That is why `Rc` is faster than `Arc`: no atomics.

Speed: atomic vs `Mutex`:
Atomics (`Atomic*`) are usually faster than a `Mutex` because they
    use CPU instructions only (no kernel or heavy locks). A `Mutex`
    may block the thread waiting for the lock, causing context
    switches and often much worse behavior under high contention.

Native atomic operations:

load:
    returns the current value of the atomic variable.
store:
    stores a new value in the atomic variable.
swap:
    stores a new value, and returns the old value.
compare_and_swap:
    compares the value of the atomic variable with a new value and
    swaps it if they are equal.
fetch_add:
    adds a value to the atomic variable.
fetch_sub, fetch_and, fetch_or, fetch_xor,
fetch_max:
    sets the value to the maximum of the current value and the new
    value.
fetch_min:
    sets the value to the minimum of the current value and the new
    value.
fetch_update:
    updates the value of the atomic variable with a function.
        Runs in an internal loop: the value is only updated if
        nobody else changed it between load and store. On a
        concurrent change it retries until it succeeds in one
        indivisible step, so updates are not lost and races are
        avoided.

Atomic ops rely on hardware support for fixed-size types
    (integers, booleans). `String` is complex and dynamic and
    cannot be updated with a single CPU atomic instruction.

The operation happens in one CPU instruction, indivisible, to
    avoid races.

Non-atomic updates (e.g. `x += 1`, plain loads/stores, or
    mutating `String`) are not single indivisible CPU
    instructions. Multiple threads can interleave reads/writes,
    causing races, corruption, or undefined behavior. Atomics
    provide safe, consistent concurrent updates.
*/
#[cfg(test)]
mod atomics {
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::thread;

    #[test]
    fn atomic_vs_non_atomic_operations() {
        // Shared atomic counter across threads via `Arc`
        // `Arc` enables shared ownership and drops when the last clone goes
        // away.
        let atomic_count = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();
        static mut STATIC_COUNT: usize = 0;

        for _ in 0..10 {
            let atomic_count = Arc::clone(&atomic_count);
            handles.push(thread::spawn(move || {
                for _ in 0..1_000_000 {
                    atomic_count.fetch_add(1, Ordering::SeqCst); // safe
                    unsafe {
                        STATIC_COUNT += 1; // data races here
                    }
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let total = 10 * 1_000_000;
        let value = atomic_count.load(Ordering::SeqCst);
        assert_eq!(value, total); // atomic operations are thread-safe
        assert!(unsafe { STATIC_COUNT } < total); // data races, not thread-safe
    }
}

/*
RC: Reference Counted
=============================================================
Rc<T> is for single-threaded ownership, with a non-atomic
    reference counter.

Multiple owners, shared immutable access only.
For shared mutability use `Rc<RefCell<T>>` or other interior
    mutability.
Dropped when the strong count reaches zero.
*/
#[cfg(test)]
pub mod rc {
    use std::rc::Rc;
    #[test]
    pub fn immutable_owners() {
        let rc1 = Rc::new(String::from("hello"));
        // Via deref both see the same value; they do not bump the Rc count.
        let _real_value = &*rc1;
        let _rc_value = &rc1; // also carries refcount metadata
        let rc2 = Rc::clone(&rc1);
        assert_eq!(Rc::strong_count(&rc1), 2);
        assert_eq!(rc1.as_ptr(), rc2.as_ptr());
    }

    /*
    Chained nodes with `Rc`
    =============================================================
    If one owning node is dropped while others remain, memory is
    not freed.
    Simplifies lifetime/ownership code.
    Only adds refcount overhead.
    */
    #[test]
    pub fn chained_nodes() {
        use std::rc::Rc;

        // Example: chained nodes with Rc
        #[derive(Debug)]
        struct Node {
            value: i32,
            next: Option<Rc<Node>>,
        }

        // Two chained nodes
        let node2 = Rc::new(Node {
            value: 2,
            next: None,
        });
        let node1 = Rc::new(Node {
            value: 1,
            next: Some(Rc::clone(&node2)),
        });

        // `node1` and `other` both own `node2`.
        let other = Rc::clone(&node2);

        // Strong count for node2
        assert_eq!(Rc::strong_count(&node2), 3);
        assert_eq!(Rc::strong_count(&node1), 1);

        // Read values through references
        assert_eq!(node1.value, 1);
        assert_eq!(node1.next.as_ref().unwrap().value, 2);
        assert_eq!(other.value, 2);

        // On scope exit counts drop and memory is freed when last `Rc` goes
        // away
    }
}

/*
Arc: Atomic Reference Counted
=============================================================
Arc<T> is for shared ownership across threads (or async tasks),
    with atomic reference counting to safely drop the heap value
    only when the last reference is gone.
Unlike `Rc` (single-threaded), `Arc` uses atomic CPU instructions
    to update the refcount so concurrent clone/drop cannot corrupt
    the count (avoiding races). Note: `Rc` is faster than `Arc`
    because it avoids atomics.

`Arc<T>` only allows immutable access to its data and does not
    implement `DerefMut`. To mutate, use `Arc<Mutex<T>>` (or
    similar).
*/
#[cfg(test)]
pub mod arc {
    use std::sync::Arc;
    use std::thread;

    #[tokio::test]
    pub async fn similar_rc() {
        let arc1 = Arc::new(String::from("hello"));
        // arc1.push_str(" world"); // error, not implemented DerefMut, only
        // Deref so only access to &T
        let arc2 = Arc::clone(&arc1);
        assert_eq!(Arc::strong_count(&arc1), 2);
        assert_eq!(arc1.as_ptr(), arc2.as_ptr());
        drop(arc1);
        // continues existing until the last reference is dropped
        assert_eq!(Arc::strong_count(&arc2), 1);
        assert_eq!(arc2.as_str(), "hello");
        // Cannot move out unless `Copy`—would leave invalid refs; compiler
        // error.
        // let z = *arc2; // error to move the content.
        assert_eq!(arc2.to_uppercase(), "HELLO"); // `String` methods via
        // `Deref` to `&T`
    }

    #[test]
    pub fn use_in_multi_thread() {
        let arc = Arc::new(String::from("hello"));
        let mut handles = vec![];
        for _ in 0..10 {
            let arc = Arc::clone(&arc);
            handles.push(thread::spawn(move || {
                assert_eq!(*arc, "hello"); // shared immutable access
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }
    }
}

/*
Mutex: Mutual Exclusion
=============================================================
`Mutex` protects any type (including `String`) from concurrent
    access by serializing access and avoiding races even when
    there is no atomic instruction for that type.

There are lower-level mutexes:
often faster, integer-only, less safe; Rust picks per platform.
    `std::sync::Mutex` is safer, portable, and works for all data
    types.
Rust’s mutex is portable, works with third-party code, and handles
    panics better.
* `pthread_mutex_t`: common on Unix-like systems (Linux, macOS, BSD),
* `futex`: POSIX systems,
* `SRWLock`: Windows,
* `parking_lot`-style mutex: cross-platform.

Methods:

`lock()`:
    Blocks the current thread until the mutex is available.
`try_lock()`:
    Tries to acquire immediately; returns `Err` if unavailable.
`clear_poisoned()`:
    Clears poison state (see below).

Poisoned:
If a thread panics while holding a `Mutex`, the mutex is
    “poisoned” and later `.lock()` calls return `Err` for poison.
    That signals the protected data may be inconsistent. Other
    threads can detect this and recover.
You can also use `.clear_poison()` / `.is_poisoned()` to clear
    poison and reuse the mutex.

*/
#[cfg(test)]
pub mod mutex {
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::thread;

    #[test]
    pub fn test() {
        let mutex = Arc::new(Mutex::new(0));
        let mut handles = vec![];
        for _ in 0..10 {
            let mutex = Arc::clone(&mutex);
            let mut num = mutex.lock().unwrap();
            *num += 1;
            handles.push(thread::spawn(move || {}));
        }
        for handle in handles {
            handle.join().unwrap();
        }
        assert_eq!(*mutex.lock().unwrap(), 10);
    }

    #[test]
    fn poisoned_mutex_basic() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let m = Arc::new(Mutex::new(String::from("hello")));
        let m2 = Arc::clone(&m);

        // Thread panics while holding the lock
        let t = thread::spawn(move || {
            let _lock = m2.lock().unwrap();
            panic!("poisoned");
        });

        let _ = t.join(); // join result carries the panic

        // # mutex error handling:
        // Mutex is poisoned; lock returns Err
        assert!(m.lock().is_err());

        // Only use unwrap/expect if poison is unexpected.
        // let x = m.lock().unwrap(); // error, mutex is poisoned
        // let x = m.lock().expect("mutex is poisoned");

        {
            let _ = match m.lock() {
                Ok(_x) => true,
                Err(_e) => false,
            };
        }

        {
            if let Ok(_x) = m.lock() {
                // handle ok
            } else {
                // handle error
            }
        }

        // Recover inner value after panic via poison error
        let guard = m.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(guard.as_str(), "hello");

        m.clear_poison(); // clear poison so the mutex can be used again

        assert!(m.lock().is_ok());
    }
}

/*
Mutex with parking_lot:
=============================================================
`parking_lot` is an efficient sync primitives implementation
    (`Mutex`, `RwLock`, etc.); it may still use atomics internally
    to coordinate threads. It minimizes blocked time and only
    parks when needed, but does not remove atomic ops entirely.
Available on Windows, Linux, and macOS. `futex` and `parking_lot`
    are not the same as “POSIX-only” in the same way—see crate
    docs.

`parking_lot` is similar to `std` sync but adds timeouts, is
    often faster, and offers extras like a better `Condvar`.

`std::sync::Mutex` favors safety and portability via OS
    primitives (more syscalls/overhead) and lacks some
    optimizations (spinning, advanced wait) that `parking_lot`
    may use.

`try_lock_for` / timeout APIs:
    Try to acquire with a timeout; returns `None` if not acquired
    in time.

Similar types in `parking_lot`:
    `Mutex`, `RwLock`, `Condvar`, `Once`, `Semaphore`, `Barrier`
*/
#[cfg(test)]
pub mod mutex_with_parking_lot {
    use parking_lot::Mutex;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    pub fn lock_with_timeout() {
        // Simple parking_lot Mutex with timeout
        let lock = Arc::new(Mutex::new(0));
        let lock2 = Arc::clone(&lock);

        // One thread holds the lock for a while
        let handle = thread::spawn(move || {
            let _guard = lock2.lock();
            thread::sleep(Duration::from_millis(50));
        });

        // Give the spawned thread time to acquire the lock
        thread::sleep(Duration::from_millis(10));

        // try_lock_for: succeeds once the other thread releases (or timeout)
        assert!(lock.try_lock_for(Duration::from_millis(50)).is_some());

        handle.join().unwrap();
    }
}

/*
Deadlock:
=============================================================
A deadlock happens when a thread/task tries to acquire a lock
    already held (directly or indirectly).
Rust can deadlock when using `Mutex` or other sync primitives
    (e.g. `RwLock`).

Cases:
Double lock:
    Locking the same mutex again on the same thread without
    releasing (non-reentrant).
Circular wait:
    Two threads each hold one lock and wait for the other’s lock.

Mitigations:
- Use `{ }` to limit lock scope (double lock).
- Always acquire multiple locks in the same order (`lock1` →
    `lock2` → …) to avoid circular wait.
- Hold locks as briefly as possible; release before blocking I/O,
    etc.
- Prefer channels, actor model, or atomics instead of mutexes
    where appropriate.
*/

#[cfg(test)]
pub mod death_lock {
    use parking_lot::Mutex;
    use std::thread;
    use std::time::Duration;
    #[test]
    pub fn double_lock() {
        let m = Mutex::new(5);
        // First lock acquisition
        let mut num1 = m.lock();
        *num1 += 1;
        // DEADLOCK: second lock on same thread (non-reentrant),
        // let mut num2 = m.lock();
    }

    use std::sync::Arc;
    #[test]
    pub fn circular_wait() {
        let mut locks = vec![Arc::new(Mutex::new(0)), Arc::new(Mutex::new(0))];
        let mut handles = vec![];
        for _i in 0..2 {
            locks.swap(0, 1); // swap positions of arrays locks, so that circular
            // wait can be produced.
            let lock1 = Arc::clone(&locks[0]);
            let lock2 = Arc::clone(&locks[1]);
            handles.push(thread::spawn(move || {
                let _ = lock1.lock();
                thread::sleep(Duration::from_millis(10));
                match lock2.try_lock_for(Duration::from_millis(20)) {
                    None => {
                        assert!(true);
                    }
                    _ => {}
                }
            }));
        }
        handles
            .into_iter()
            .for_each(|handle| handle.join().unwrap());
    }
}

/*
RwLock (Read-Write Lock)
=============================================================
`RwLock` is a readers–writers lock: many readers or one writer
    at a time.

- While no writer is active, any number of readers may proceed.
- A writer waits for all readers to finish and for no other
    writer to be active, then gets exclusive access.
- While a writer holds the lock, no readers or other writers run.

Useful when reads dominate writes, improving concurrency vs a
    plain `Mutex`.

Methods:
    `read()`:
        Shared read lock; many concurrent readers.
    `write()`:
        Exclusive write lock; one writer.
    `try_read()`, `try_write()`:
        Non-blocking attempts; return `Err` if unavailable.
*/
#[cfg(test)]
pub mod rw_lock {
    use std::sync::Arc;
    use std::sync::RwLock;
    use std::thread;
    #[test]
    pub fn rw_lock_example() {
        let lock = Arc::new(RwLock::new(0));
        let lock2 = Arc::clone(&lock);
        let handle = thread::spawn(move || {
            // read
            {
                let r = lock2.read().unwrap();
                assert_eq!(*r, 0);
            }
            // write
            {
                let mut w = lock2.write().unwrap();
                *w += 1;
            }
        });

        handle.join().unwrap();
        assert_eq!(*lock.read().unwrap(), 1);
    }
}

/*
Condvar: Condition Variable
=============================================================
Parks the current thread until a condition holds.

A thread waits on a `Condvar` with `.wait()`, which releases the
    paired mutex and sleeps until another thread wakes it.
Another thread calls `.notify_one()` or `.notify_all()` after
    updating shared state that defines progress.

Benefits:
    - Efficient waiting without busy spinning.
    - Clean coordination when threads must wait for state.

Caveats:
    - After waking, re-acquire the mutex and re-check the
    condition before proceeding.
    - Use a loop: spurious wakeups can occur without `.notify_*()` .

Key methods:
    - `.wait(guard)`: Release lock and sleep until notified.
    - `.notify_one()`: Wake one waiter.
    - `.notify_all()`: Wake all waiters.
*/
#[cfg(test)]
pub mod condvar {
    #[test]
    pub fn test() {
        use std::sync::{Arc, Condvar, Mutex};
        use std::thread;

        // Shared data and condition
        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        let pair2 = pair.clone();

        // Thread waiting on the condition
        let waiter = thread::spawn(move || {
            let (lock, cvar) = &*pair2;
            let mut ready = lock.lock().unwrap();
            // Wait until `ready` is true
            while !*ready {
                ready = cvar.wait(ready).unwrap(); // park; releases lock
                // On notify, lock is re-acquired and we loop to re-check.
            }
            return "unlocked";
        });

        // Let the waiter block first
        thread::sleep(std::time::Duration::from_millis(10));

        // Thread that sets the flag and notifies
        {
            let (lock, cvar) = &*pair;
            let mut ready = lock.lock().unwrap();
            *ready = true;
            // Wake the waiter
            cvar.notify_one();
        }

        let result = waiter.join().unwrap();
        assert_eq!(result, "unlocked");
    }
}

/*
Channels (mpsc)
=============================================================
N producers, 1 consumer.

`mpsc::channel` properties:
- Multiple `Sender`s (clone `tx`).
- One `Receiver`.
- When all `tx` are dropped, `rx.recv()` returns `Err` (channel
    closed).
- Ownership moves on each send.
- FIFO internal queue.
- Also `try_recv`, `recv_timeout` for non-blocking or timed
    receives.

*/
#[cfg(test)]
pub mod channels {
    use std::time::Duration;

    #[test]
    pub fn test() {
        use std::sync::mpsc;
        use std::thread;

        let (tx, rx) = mpsc::channel();

        // Producer threads
        for (index, message) in vec!["A", "B", "C", "D", "E"].into_iter().enumerate() {
            let tx = tx.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(index as u64 * 10));
                tx.send(message).unwrap();
            });
        }
        drop(tx); // only lives in the threads

        // Consumer (may run on main)
        // when last `tx` is dropped, `rx.recv()` returns Err and the loop
        // ends
        let mut received = vec![];
        while let Ok(msg) = rx.recv() {
            received.push(msg);
        }
        assert_eq!(received, vec!["A", "B", "C", "D", "E"]);
    }
}

/*
Broadcast (multi-consumer)
=============================================
1 producer, N consumers.

Tasks only (Tokio), not `std::thread`.

Behavior:
- When a message is sent, all active subscribers can receive it.
- Subscribers can join dynamically and only see messages from
    then on.
- Each subscriber has its own cursor; a slow consumer may lag
    and miss recent messages if the buffer is exceeded.
Alternatives:
- If only one consumer should get each message, use `mpsc`.
- To share state with everyone, consider `Arc<Mutex<T>>`.

*/
#[cfg(test)]
pub mod broadcast {
    use std::thread;
    use std::time::Duration;
    use tokio::sync::broadcast;
    #[tokio::test]
    pub async fn test() {
        let (tx, mut rx1) = broadcast::channel(16);
        let mut rx2 = tx.subscribe();

        // Wait for the broadcast message
        let handle1 = tokio::spawn(async move { rx1.recv().await.unwrap() });
        let handle2 = tokio::spawn(async move { rx2.recv().await.unwrap() });

        thread::sleep(Duration::from_millis(10));
        tx.send("message").unwrap();

        assert_eq!(handle1.await.unwrap(), "message");
        assert_eq!(handle2.await.unwrap(), "message");
    }
}

/*
Once / OnceCell
=============================================================
Runs initialization exactly once, even when invoked from many
    concurrent threads or tasks.

If another caller arrives while init is in progress, it waits
    until it finishes.
After success, later calls do not re-run the closure.

*/
#[cfg(test)]
pub mod once_or_once_cell {
    use std::sync::Arc;
    use std::sync::Mutex;
    use tokio::sync::OnceCell;
    /*
    With tasks:
    Others wait for initialization; the init closure runs only once.
    */
    #[tokio::test]
    async fn call_once_tasks() {
        let once = Arc::new(OnceCell::new());
        let value = Arc::new(Mutex::new(0));

        once.get_or_init(|| async {
            *value.lock().unwrap() = 123;
        })
        .await;

        let futs = (0..10).map(|i| {
            let once = Arc::clone(&once);
            let value = Arc::clone(&value);
            tokio::spawn(async move {
                once.get_or_init(|| async {
                    *value.lock().unwrap() = i;
                })
                .await;
            })
        });

        for fut in futs {
            fut.await.unwrap();
        }

        assert_eq!(*value.lock().unwrap(), 123);
    }
}

/*
Barrier
=============================================================
Lets multiple threads/tasks meet at a point and wait until all
    arrive before continuing.
*/
#[cfg(test)]
pub mod barrier {
    use std::sync::Mutex;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    pub fn test() {
        let n = 5_000;
        let barrier = Arc::new(Barrier::new(n));
        let value = Arc::new(Mutex::new(0));
        let mut handles = vec![];

        for _i in 0..n {
            let barrier = Arc::clone(&barrier);
            let value = Arc::clone(&value);
            handles.push(thread::spawn(move || {
                assert_eq!(*value.lock().unwrap(), 0);
                barrier.wait(); // all wait
                *value.lock().unwrap() = 1;
                barrier.wait(); // all wait
                assert_eq!(*value.lock().unwrap(), 1);
                barrier.wait(); // all wait
                *value.lock().unwrap() = 0;
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
    }
}

/*
Semaphore
=============================================================
Limits how many tasks may use a shared resource at once.
    If the limit is reached, tasks wait until a permit is released.

For OS threads, combine `Condvar` + `Mutex` or a crate semaphore.

*/
#[cfg(test)]
pub mod semaphore {

    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Semaphore;

    #[tokio::test]
    pub async fn tasks() {
        let limit = 3;
        let semaphore = Arc::new(Semaphore::new(limit));
        let counter = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];
        for _i in 0..(limit * 3) {
            let sem = semaphore.clone();
            let counter = counter.clone();
            handles.push(tokio::spawn(async move {
                assert!(counter.load(Ordering::SeqCst) <= limit);
                let permit = sem.acquire().await.unwrap(); // acquire permit or wait if at capacity
                counter.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                counter.fetch_sub(1, Ordering::SeqCst);
                drop(permit); // release permit.
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }
}

/*
Lock-free patterns
=============================================================
Lock-free patterns let many threads or tasks access and update
    data concurrently without mutex-style locks. That can cut
    latency and raise throughput, but you rely on atomics for
    correctness.

They usually build on `compare_exchange` / CAS on `Atomic*`
    types (`AtomicUsize`, `AtomicBool`, …). Examples: atomic
    counters, lock-free stacks, lock-free queues.

They suit high-concurrency designs with low contention points
    but are harder to implement and reason about than
    lock-based code.

Under heavy contention, lock-free code may spin on failed CAS,
    burning CPU.
The scheduler can also favor some threads, causing imbalance.

Lock-free trade-offs:

1. Starvation:
    On CAS failure another thread may win; the loser retries.
    Persistent loss is rare but theoretically possible.

    The race window is tiny.
    Retries are cheap and stay on-core.

2. High contention (where it hurts)
    Lock-free: many threads compete; one wins, others spin
    (~50–100 ns per failed CAS).

    Mutex: one holder; others block in the OS (~1–10 µs context
    switches).

    Summary: lock-free spins in user space; mutexes sleep
    threads—often worse under heavy contention.
 */
#[cfg(test)]
pub mod lock_free_patterns {
    #[test]
    fn test() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::thread;

        let counter = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];

        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.load(Ordering::SeqCst), 10 * 1000);
    }

    /*
    Example:
    Nodes linked list lock-free stack.
     */
    #[test]
    fn lock_free_stack_example() {
        use std::ptr;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicPtr, Ordering};
        use std::thread;
        struct Node {
            _value: usize,
            next: *mut Node,
        }
        let head = Arc::new(AtomicPtr::new(ptr::null_mut()));
        let mut handles = vec![];
        for i in 0..10 {
            let head = Arc::clone(&head);
            handles.push(thread::spawn(move || {
                let node = Box::new(Node {
                    _value: i,
                    next: ptr::null_mut(),
                });
                let node_ptr = Box::into_raw(node);
                // atomic operation: update node_ptr and head to the new node ptr
                loop {
                    let old_head = head.load(Ordering::SeqCst);
                    unsafe {
                        (*node_ptr).next = old_head;
                    }
                    // compete for the lock, try to update the head with the new node.
                    if head
                        .compare_exchange(old_head, node_ptr, Ordering::SeqCst, Ordering::SeqCst)
                        .is_ok()
                    {
                        break;
                    }
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        // Walk the stack to verify all nodes were pushed
        let mut current = head.load(Ordering::SeqCst);
        let mut count = 0;
        while !current.is_null() {
            unsafe {
                current = (*current).next;
            }
            count += 1;
        }
        assert_eq!(count, 10);
    }
}

/*
Actor Model
=============================================================
Actors do work and communicate via messages; the actor thread
    owns and updates state, avoiding shared mutexes.

Safety comes from a single thread (or single actor task)
    mutating that state; messages serialize changes so races are
    ruled out by design.

Private state:
The actor alone owns its state. Other threads/actors do not read
    or write it directly.
Mailbox: To change state or query it, others send messages
    instead of touching memory.
Sequential handling: The actor processes messages one at a time
    in order, so locks are unnecessary for that state.
*/
#[cfg(test)]
pub mod actor_model {
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[test]
    pub fn test() {
        // Actor: two channels per independent actor instance. Here one actor
        // handles sequential work on one inbound channel.
        let (tx, rx) = mpsc::channel();
        let (tx_actor, rx_actor) = mpsc::channel();

        // Actor runs on a dedicated thread
        let handle = thread::spawn(move || {
            for msg in rx {
                // Reply with double the input
                tx_actor.send(msg * 2).unwrap();
            }
        });

        // Send several messages
        vec![1, 2, 3, 4, 5]
            .into_iter()
            .for_each(|i| tx.send(i).unwrap());

        // Close inbound channel so the actor loop can end
        drop(tx);

        let mut responses = vec![];
        // Collect replies from the actor
        while let Ok(val) = rx_actor.recv_timeout(Duration::from_millis(100)) {
            responses.push(val);
        }

        // Each reply should be double the corresponding input
        assert_eq!(responses, vec![2, 4, 6, 8, 10]);
        handle.join().unwrap();
    }
}

/*
Worker Pool
=============================================================
A worker pool uses a fixed set of threads pulling jobs from a
    shared queue to cap parallelism and reuse threads.

Basics:
- Job queue (often a channel).
- N long-lived worker threads blocking on the queue.
- Producers enqueue work; an idle worker picks it up.
- Typical for parallel file/CPU/network work units.

Benefits:
- Bounded concurrency (e.g. near core count).
- Amortizes thread creation cost.
- Natural backpressure: excess jobs wait in the queue.
*/

/*
Example:
Worker pool with mpsc channel.
 */
#[cfg(test)]
pub mod worker_pool {
    use std::collections::HashSet;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;
    #[test]
    pub fn test() {
        let num_workers = 4;
        let num_jobs = 28;

        // Job channel (here just integers)
        let (tx, rx) = mpsc::channel();

        // Share one `Receiver` across workers via `Arc<Mutex<_>>`.
        // (`Receiver` is not `Clone`; alternative is one channel per worker.)
        use std::sync::{Arc, Mutex};
        let rx = Arc::new(Mutex::new(rx));
        let jobs_processed = Arc::new(Mutex::new(HashSet::<usize>::new()));

        let mut handles = vec![];

        // Spawn worker threads
        for _id in 0..num_workers {
            let rx = Arc::clone(&rx);
            let jobs_processed = Arc::clone(&jobs_processed);
            let handle = thread::spawn(move || {
                loop {
                    // Wait for a job; exit when the channel closes
                    let msg = rx.lock().unwrap().recv();
                    match msg {
                        Ok(job) => {
                            jobs_processed.lock().unwrap().insert(job);
                            thread::sleep(Duration::from_millis(10));
                        }
                        Err(_) => {
                            // Channel closed: stop
                            break;
                        }
                    }
                }
            });
            handles.push(handle);
        }

        // Submit jobs
        for job in 1..=num_jobs {
            tx.send(job).unwrap();
        }
        // Close sender so workers can exit
        drop(tx);

        // Wait for workers
        for handle in handles {
            handle.join().unwrap();
        }
        assert_eq!(jobs_processed.lock().unwrap().len(), num_jobs);
    }
}

/*
Fork-Join
=============================================================
Fork–join splits a large task into parallel subtasks (“fork”),
    runs them concurrently, then waits for all to finish
    (“join”) before continuing.

Steps:
1. Fork: parent splits work and spawns threads/tasks.
2. Parallelism: workers run their slice at the same time.
3. Join: parent waits and combines results as needed.
*/

/*
DashMap (concurrent hash map)
=============================================================
`DashMap` is a concurrent hash map: many threads can read/write at
    once without one global lock—only per-shard synchronization. It
    shards the table so contention drops vs a single big mutex.

Highlights:
- Concurrent reads/writes with less contention than one lock for the
    whole map.
- API similar to `HashMap`, but entries are accessed through guards
    that lock only the relevant shard.
- Fits workloads where keys spread across shards.

Implementation notes:
Per-shard `RwLock`-style locking; hasher runs on the key.
Shard index often uses `hash & (shard_count - 1)` instead of `%`
    when `shard_count` is a power of two—cheap masking vs division.
Hashing (e.g. `RandomState` / SipHash) spreads keys pseudo-randomly
    across shards.

Tunables:
    Internal lock type (rarely changed).
    Custom `BuildHasher`.
    Key/value types.
    Capacity via `with_capacity`, etc.

Threads working on different shards rarely block each other; good
    when you want concurrent `HashMap`-like access without locking
    the entire table.
*/
#[cfg(test)]
pub mod dashmap {
    #[test]
    pub fn test() {
        use dashmap::DashMap;
        use std::sync::Arc;
        use std::thread;

        let shards = 4;
        let map = Arc::new(DashMap::new());
        let mut handles = vec![];
        for i in 0..shards {
            let map = Arc::clone(&map);
            handles.push(thread::spawn(move || {
                map.insert(i, i * 10);
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }
        // Concurrent reads
        assert_eq!(map.len(), shards);
        // Iteration may block briefly if another thread mutates the same shard
        for entry in map.iter() {
            println!("key={}, value={}", entry.key(), entry.value());
        }
    }
}

/*
Crossbeam
=============================================================
N producer, N consumers

Crossbeam channels:
    - MPMC: multiple senders and receivers.
- Often lock-free where possible, reducing contention.
    - `select!` across multiple channels (Go-like).
- Bounded and unbounded channels.
- Closing senders ends the stream cleanly.

Compared to `std::sync::mpsc`: more flexible, often faster,
    supports `select!` on several channels.
*/
#[cfg(test)]
pub mod crossbeam {
    use crossbeam_channel::{bounded, select};
    use std::thread;

    #[test]
    pub fn test_bounded_and_select() {
        let capacity = 2; // max 2 messages in the channel, if more, the
        // sender will block until a message is received.
        let (s1, r1) = bounded(capacity);
        let (s2, r2) = (s1.clone(), r1.clone()); // N producers, N consumers
        let (_s1_keep, _r1_keep) = (s1.clone(), r1.clone()); // keep living,
        // because if dropped return err instead of messages.
        // (listening on a closed channel)

        let handle1 = thread::spawn(move || {
            s1.send("msg from s1").unwrap();
        });

        let handle2 = thread::spawn(move || {
            s2.send("msg from s2").unwrap();
        });

        // Use select! to wait on both receivers
        let mut messages = vec![];
        for _ in 0..2 {
            select! {
                recv(r1) -> msg => messages.push(msg.unwrap()),
                recv(r2) -> msg => messages.push(msg.unwrap()),
            }
        }
        drop((r1, r2));
        messages.sort();
        assert_eq!(messages, vec!["msg from s1", "msg from s2"]);

        handle1.join().unwrap();
        handle2.join().unwrap();
    }
}

/*
Rayon (data parallelism)
=============================================================
Rayon is a data-parallelism crate: split collections and process
    pieces in parallel (`par_iter`, `map`, `filter`, `collect`, …)
    using a thread pool so you do not spawn unbounded threads.

Good for embarrassingly parallel work where per-element operations
    are independent.

Other APIs:
    `par_iter_mut()` — parallel in-place updates.
    `par_chunks(n)` — parallel per chunk.
    `par_sort()` — parallel sort.
    `par_filter()` — via `.par_iter().filter()`.
    `reduce`, `for_each`, `find_any`, `any`, `all` — parallel
    reductions and searches.

Uses a worker pool; good for parallel search or large splittable
    workloads.
    `par_chunks` runs each chunk in parallel.

You can also hand-roll parallelism with `thread::spawn` /
    `spawn_blocking`, or use `tokio-rayon` for async integration.
*/
#[cfg(test)]
pub mod rayon {
    use rayon::prelude::*;

    #[test]
    pub fn test() {
        // Example: square each element in parallel
        let a = vec![1, 2, 3, 4, 5];
        let mut squares: Vec<_> = a.par_iter().map(|x| x * x).collect();
        squares.sort(); // parallel iter order is not defined
        assert_eq!(squares, vec![1, 4, 9, 16, 25]);
    }

    #[test]
    pub fn test_config_worker_pool() {
        let num_threads = 4;
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .unwrap();
        pool.install(|| {
            let a = vec![1, 2, 3, 4, 5];
            let mut squares: Vec<_> = a.par_iter().map(|x| x * x).collect();
            squares.sort();
            assert_eq!(squares, vec![1, 4, 9, 16, 25]);
        });
    }
}

/*
Work Stealing
=============================================================
Work stealing balances load: idle worker threads can take tasks
    from busier workers’ queues.

Each worker keeps a local deque; when empty, it steals from
    another worker’s queue instead of using only one global queue.

This improves utilization and total runtime when work is uneven
    or hard to partition evenly up front.

Avoiding one global task queue (lock contention):
    With a single global queue, every idle core might contend on
    one mutex to grab work.

Cache locality:
    RAM is slow vs L1/L2/L3. Pushing work to a thread’s local
    queue keeps related data hot in that core’s cache. Stealing
    from another queue can move cache lines and cost more.

Stealing is a fallback:
    Local work is preferred; stealing is for imbalance.

Libraries: Rayon (and many runtimes) use work-stealing schedulers.
*/

// Utils
#[cfg(test)]
mod utils {

    use std::char;
    use std::thread::ThreadId;
    pub fn get_thread_id_number(thread_id: ThreadId) -> u64 {
        let s = format!("{:?}", thread_id);
        s.matches(char::is_numeric)
            .collect::<String>()
            .to_string()
            .parse::<u64>()
            .unwrap()
    }
}
