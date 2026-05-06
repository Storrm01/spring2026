# Final_Project

## Project Overview

Final_Project is a concurrent task dispatcher simulation written in Rust.
The program simulates how an operating system or service scheduler handles incoming CPU-bound and IO-bound tasks using a bounded worker pool, queue-based scheduling, and concurrent execution.

The system generates tasks over time, places them into queues, dispatches them using a weighted round-robin scheduling policy, processes them concurrently using worker threads, records performance statistics, and shuts down cleanly after all work is completed.

---

# Features

- Concurrent multi-threaded design
- CPU and IO task simulation
- Queue-based scheduling system
- Bounded worker pool
- Weighted round-robin dispatch policy
- Automatic workload generation
- Performance metrics collection
- Clean thread shutdown
- Experiment result logging to a text file

---

# Technologies Used

- Rust
- Cargo
- Standard library concurrency primitives:
  - `thread` 
    - for task generator, dispatcher and workers
  - `mpsc`
    - for one side to send a task, while the other receives it (used for generator -> dispatcher, dispatcher -> workers, workers -> metrics collector) to safely send a task to another thread
  - `Arc`
    - for when multiple worker threads need access to the same receiver, if 'Arc' isnt here, only one thread could own the receiver
  - `Mutex`
    - Only one worker thread can access the receiver at once, preventing race conditions
- `rand` crate
    - to create random task durations, using 

---

# Project Structure

```text
Final_Project/
├── Cargo.toml
├── src/
│   └── main.rs
└── experiment_results.txt
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

The program writes experiment results to:

```text
experiment_results.txt
```

The file includes:
- makespan
- average wait time
- average turnaround time
- worker utilization
- queue statistics
- CPU vs IO completion counts
- worker task summaries

---

# Summary of Design

The project uses a concurrent producer-dispatcher-worker architecture.

## Components

### Task Generator Thread

The generator thread automatically creates tasks using a fixed random seed for reproducibility. Tasks are generated over time to simulate realistic workload arrivals.

### Dispatcher Thread

The dispatcher receives tasks and places them into separate CPU and IO queues using `VecDeque`.

The dispatcher uses a weighted round-robin scheduling policy:
- two CPU tasks are preferred
- then one IO task is selected

This policy allows CPU-heavy workloads to receive more processing time while still preventing IO starvation.

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

# Scheduling Policy

The scheduling policy used is a weighted round-robin dispatcher.

Behavior:
- CPU queue receives priority
- two CPU tasks are dispatched before one IO task
- if one queue becomes empty, the dispatcher uses the remaining queue

This improves throughput for CPU-heavy workloads while still allowing IO tasks to progress.

---

# Summary of Experiments

## Experiment A — Balanced Workload

Configuration:
- 500 total tasks
- 50% CPU tasks
- 50% IO tasks
- 6 workers
- normal arrival timing

Results:
- lower average wait times
- balanced worker utilization
- fair distribution between CPU and IO tasks

This workload demonstrated stable scheduling performance and efficient concurrency.

---

## Experiment B — Stressed CPU-Heavy Workload

Configuration:
- 500 total tasks
- 85% CPU tasks
- burst-style arrivals
- 6 workers

Results:
- higher average wait times
- increased turnaround times
- heavier CPU queue pressure

This workload stressed the dispatcher and demonstrated how CPU-heavy workloads increase contention and waiting time.

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

Tools used:
- ChatGPT
- Rust documentation

Help provided:
- debugging concurrency logic
- improving scheduling structure
- refining metrics collection
- improving shutdown handling
- issues with warnings and what to do to fix them

Example of accepted advice:
- implementing weighted round-robin scheduling

Example of modified/fixed advice:
- replacing large per-task logging with summary-based reporting for cleaner experiment output