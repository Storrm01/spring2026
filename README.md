# Final_Project

## Project Overview

Final_Project is a concurrent task dispatcher simulation written in Rust. The program simulates how an operating system or service scheduler handles incoming CPU-bound and IO-bound tasks using a bounded worker pool, queue-based scheduling, and concurrent execution.

The system generates tasks over time, places them into queues, dispatches them using scheduling policies, processes them concurrently using worker threads, records performance statistics, and shuts down cleanly after all work is completed.

---

# Features

- Concurrent multi-threaded design
- CPU and IO task simulation
- Queue-based scheduling system
- Bounded worker pool
- FIFO and weighted round-robin dispatch policies
- Automatic workload generation
- Performance metrics collection
- Clean thread shutdown
- Experiment result logging to text files

---

# Technologies Used

- Rust
- Cargo
- Standard library concurrency primitives:
  - `thread`
    - Used for task generator, dispatcher, and worker threads to allow concurrent execution.
  - `mpsc`
    - Used for communication between threads. One side sends tasks while the other receives them. Channels are used between:
      - generator → dispatcher
      - dispatcher → workers
      - workers → metrics collector
  - `Arc`
    - Allows multiple worker threads to safely share ownership of the same receiver.
  - `Mutex`
    - Ensures that only one worker thread accesses shared data at a time, preventing race conditions and unsafe concurrent access.
- `rand` crate
  - Used to generate random task types, durations, and arrival timing for workload simulation.

---

# Project Structure

```text
Final_Project/
├── Cargo.toml
├── src/
│   └── main.rs
├── experiment_results.txt
├── fifo_output_example.txt
└── optimized_output_example.txt
```

---

# How to Build and Run

## Build the Project

```bash
cargo build
```

## Run the Project

```bash
cargo run
```

---

# Command Examples

## Compile only

```bash
cargo build
```

## Run the simulation

```bash
cargo run
```

## Run with release optimizations

```bash
cargo run --release
```

---

# Output

The program generates three output files:

```text
experiment_results.txt
fifo_output_example.txt
optimized_output_example.txt
```

## experiment_results.txt

Contains the primary experiment results for the project workloads, including:
- makespan
- average wait time
- average turnaround time
- worker utilization
- queue statistics
- CPU vs IO completion counts
- worker task summaries

## fifo_output_example.txt

Contains formatted example output for the FIFO scheduling policy.

## optimized_output_example.txt

Contains formatted example output for the optimized weighted round-robin scheduling policy.

---

# Summary of Design

The project uses a concurrent producer-dispatcher-worker architecture.

## Components

### Task Generator Thread

The generator thread automatically creates tasks using a fixed random seed for reproducibility. Tasks are generated over time to simulate realistic workload arrivals.

### Dispatcher Thread

The dispatcher receives tasks and places them into queues using `VecDeque`.

Two scheduling policies were implemented:
- FIFO scheduling
- Weighted round-robin scheduling

The weighted round-robin scheduler gives preference to CPU tasks by dispatching two CPU tasks before one IO task.

This policy allows CPU-heavy workloads to receive more processing time while still ensuring IO tasks continue to make progress.

### Worker Pool

The system uses a bounded worker pool with a fixed number of worker threads.

Workers:
- repeatedly receive tasks
- simulate execution using `thread::sleep`
- record timing metrics
- report completion statistics

### Shared State and Synchronization

The project uses:
- `mpsc` channels for communication between threads
- `Arc<Mutex<_>>` to safely share the worker receiver between threads

This prevents data races while allowing concurrent execution.

### Clean Shutdown

After all tasks are dispatched:
- the dispatcher sends `None` messages
- workers detect shutdown signals and terminate
- all threads are joined cleanly

---

# Scheduling Policies

## FIFO Scheduling

FIFO (First In First Out) scheduling dispatches tasks in the order they arrive. Tasks are processed sequentially from the front of the queue without prioritizing task type.

## Weighted Round-Robin Scheduling

The weighted round-robin scheduler prioritizes CPU tasks by dispatching two CPU tasks before one IO task.

Behavior:
- CPU queue receives priority
- two CPU tasks are dispatched before one IO task
- if one queue becomes empty, the dispatcher uses the remaining queue

The primary optimization goal is reducing total runtime (makespan).

---

# Summary of Experiments

Additional formatted FIFO and optimized scheduler example outputs were also generated for comparison purposes.

## Experiment A — Balanced Workload

Configuration:
- 500 total tasks
- 50% CPU tasks
- 50% IO tasks
- 6 workers
- normal arrival timing

Results:
- balanced task distribution
- stable worker utilization
- predictable queue behavior

---

## Experiment B — Stressed CPU-Heavy Workload

Configuration:
- 500 total tasks
- 85% CPU tasks
- burst-style arrivals
- 6 workers

Results:
- increased queue pressure
- higher wait times
- longer overall runtime

---

# Runtime Priority

The primary performance metric for this project is total runtime (makespan), as specified in the project amendments. Other collected metrics such as wait time, queue statistics, worker utilization, and task distribution are supplementary and are used only to help explain runtime behavior. If a scheduling policy improves secondary metrics at the cost of increasing total runtime, it is not considered a successful optimization for this project.

---

# Metrics Collected

The project records:
- total tasks completed
- makespan
- average wait time
- average turnaround time
- CPU tasks completed
- IO tasks completed
- max wait time
- queue length statistics
- worker utilization
- worker task summaries

---

# Tool Use Disclosure

## Tools Used

- ChatGPT
- Rust documentation

## Help Provided

- debugging concurrency logic
- improving scheduling structure
- refining metrics collection
- improving shutdown handling
- resolving compiler warnings
- improving experiment output formatting

## Example of Accepted Advice

- implementing weighted round-robin scheduling

## Example of Modified/Fixed Advice

- replacing large per-task logging with summary-based reporting for cleaner experiment output