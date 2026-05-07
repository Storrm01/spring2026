use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq)]
enum TaskKind {
    Cpu,
    Io,
}

#[derive(Clone, Copy)]
enum SchedulingPolicy {
    Fifo,
    WeightedRoundRobin,
}

#[derive(Clone, Debug)]
struct Task {
    id: usize,
    arrival_time: u128,
    kind: TaskKind,
    duration: u64,
    arrived_at: Instant,
}

#[derive(Debug)]
struct CompletedTask {
    task_id: usize,
    kind: TaskKind,
    wait_ms: u128,
    turnaround_ms: u128,
    duration_ms: u64,
    worker_id: usize,
}

#[derive(Clone, Copy)]
struct ExperimentConfig {
    name: &'static str,
    total_tasks: usize,
    workers: usize,
    cpu_percent: u32,
    bursty: bool,
    policy: SchedulingPolicy,
}

struct ExperimentStats {
    total_runtime: u128,
    makespan: u128,
    total_completed: usize,
    avg_wait: f64,
    avg_turnaround: f64,
    avg_io_wait: f64,
    avg_cpu_wait: f64,
    cpu_completed: usize,
    io_completed: usize,
    max_wait: u128,
    max_wait_task_id: usize,
    max_queue: usize,
    avg_queue: f64,
    utilization: f64,
    avg_workers_active: f64,
    monitor_samples: usize,
    worker_task_counts: Vec<usize>,
}

fn generate_task(
    id: usize,
    config: ExperimentConfig,
    rng: &mut StdRng,
    experiment_start: Instant,
) -> Task {
    let kind = if rng.gen_range(1..=100) <= config.cpu_percent {
        TaskKind::Cpu
    } else {
        TaskKind::Io
    };

    let duration = match kind {
        TaskKind::Cpu => rng.gen_range(25..=80),
        TaskKind::Io => rng.gen_range(10..=45),
    };

    Task {
        id,
        arrival_time: experiment_start.elapsed().as_millis(),
        kind,
        duration,
        arrived_at: Instant::now(),
    }
}

fn run_experiment(config: ExperimentConfig) -> ExperimentStats {
    println!("Running experiment: {}", config.name);

    let experiment_start = Instant::now();

    let (arrival_sender, arrival_receiver) = mpsc::channel::<Task>();
    let (work_sender, work_receiver) = mpsc::channel::<Option<Task>>();
    let (completed_sender, completed_receiver) = mpsc::channel::<CompletedTask>();

    let shared_work_receiver = Arc::new(Mutex::new(work_receiver));

    let generator_config = config;
    let generator_start = experiment_start;

    let generator = thread::spawn(move || {
        let mut rng = StdRng::seed_from_u64(42);

        for id in 0..generator_config.total_tasks {
            let task = generate_task(id, generator_config, &mut rng, generator_start);

            arrival_sender.send(task).unwrap();

            if generator_config.bursty {
                if id % 50 == 0 {
                    thread::sleep(Duration::from_millis(100));
                }
            } else {
                thread::sleep(Duration::from_millis(rng.gen_range(1..=5)));
            }
        }
    });

    let dispatcher_workers = config.workers;
    let policy = config.policy;

    let dispatcher = thread::spawn(move || {
        let mut fifo_queue: VecDeque<Task> = VecDeque::new();
        let mut cpu_queue: VecDeque<Task> = VecDeque::new();
        let mut io_queue: VecDeque<Task> = VecDeque::new();

        let mut cpu_turns = 0;
        let mut queue_samples = Vec::new();
        let mut generator_done = false;

        while !generator_done
            || !fifo_queue.is_empty()
            || !cpu_queue.is_empty()
            || !io_queue.is_empty()
        {
            match arrival_receiver.recv_timeout(Duration::from_millis(5)) {
                Ok(task) => match policy {
                    SchedulingPolicy::Fifo => {
                        fifo_queue.push_back(task);
                    }
                    SchedulingPolicy::WeightedRoundRobin => match task.kind {
                        TaskKind::Cpu => cpu_queue.push_back(task),
                        TaskKind::Io => io_queue.push_back(task),
                    },
                },

                Err(mpsc::RecvTimeoutError::Timeout) => {}

                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    generator_done = true;
                }
            }

            queue_samples.push(fifo_queue.len() + cpu_queue.len() + io_queue.len());

            let next_task = match policy {
                SchedulingPolicy::Fifo => fifo_queue.pop_front(),

                SchedulingPolicy::WeightedRoundRobin => {
                    if cpu_turns < 2 && !cpu_queue.is_empty() {
                        cpu_turns += 1;
                        cpu_queue.pop_front()
                    } else if !io_queue.is_empty() {
                        cpu_turns = 0;
                        io_queue.pop_front()
                    } else if !cpu_queue.is_empty() {
                        cpu_queue.pop_front()
                    } else {
                        None
                    }
                }
            };

            if let Some(task) = next_task {
                work_sender.send(Some(task)).unwrap();
            }
        }

        for _ in 0..dispatcher_workers {
            work_sender.send(None).unwrap();
        }

        let max_queue = queue_samples.iter().max().copied().unwrap_or(0);

        let avg_queue = if queue_samples.is_empty() {
            0.0
        } else {
            queue_samples.iter().sum::<usize>() as f64 / queue_samples.len() as f64
        };

        (max_queue, avg_queue, queue_samples.len())
    });

    let mut handles = Vec::new();

    for worker_id in 0..config.workers {
        let receiver = Arc::clone(&shared_work_receiver);
        let sender = completed_sender.clone();

        let handle = thread::spawn(move || loop {
            let message = {
                let lock = receiver.lock().unwrap();
                lock.recv()
            };

            match message {
                Ok(Some(task)) => {
                    let _arrival_time = task.arrival_time;

                    let start = Instant::now();
                    let wait_ms = start.duration_since(task.arrived_at).as_millis();

                    thread::sleep(Duration::from_millis(task.duration));

                    let turnaround_ms = Instant::now()
                        .duration_since(task.arrived_at)
                        .as_millis();

                    let completed = CompletedTask {
                        task_id: task.id,
                        kind: task.kind,
                        wait_ms,
                        turnaround_ms,
                        duration_ms: task.duration,
                        worker_id,
                    };

                    sender.send(completed).unwrap();
                }

                Ok(None) => break,

                Err(_) => break,
            }
        });

        handles.push(handle);
    }

    drop(completed_sender);

    let mut results = Vec::new();
    let mut worker_task_counts = vec![0; config.workers];

    for completed in completed_receiver {
        worker_task_counts[completed.worker_id] += 1;
        results.push(completed);
    }

    generator.join().unwrap();

    for handle in handles {
        handle.join().unwrap();
    }

    let (max_queue, avg_queue, monitor_samples) = dispatcher.join().unwrap();

    let total_runtime = experiment_start.elapsed().as_millis();
    let makespan = total_runtime;

    let total_completed = results.len();

    let avg_wait =
        results.iter().map(|r| r.wait_ms).sum::<u128>() as f64 / total_completed as f64;

    let avg_turnaround =
        results.iter().map(|r| r.turnaround_ms).sum::<u128>() as f64 / total_completed as f64;

    let cpu_completed = results.iter().filter(|r| r.kind == TaskKind::Cpu).count();
    let io_completed = results.iter().filter(|r| r.kind == TaskKind::Io).count();

    let cpu_results: Vec<&CompletedTask> =
        results.iter().filter(|r| r.kind == TaskKind::Cpu).collect();

    let io_results: Vec<&CompletedTask> =
        results.iter().filter(|r| r.kind == TaskKind::Io).collect();

    let avg_cpu_wait = if cpu_results.is_empty() {
        0.0
    } else {
        cpu_results.iter().map(|r| r.wait_ms).sum::<u128>() as f64
            / cpu_results.len() as f64
    };

    let avg_io_wait = if io_results.is_empty() {
        0.0
    } else {
        io_results.iter().map(|r| r.wait_ms).sum::<u128>() as f64
            / io_results.len() as f64
    };

    let max_wait_task = results.iter().max_by_key(|r| r.wait_ms).unwrap();

    let total_work_time: u64 = results.iter().map(|r| r.duration_ms).sum();

    let possible_work_time = total_runtime as f64 * config.workers as f64;

    let utilization = total_work_time as f64 / possible_work_time * 100.0;

    let active_workers = worker_task_counts.iter().filter(|&&count| count > 0).count();

    let avg_workers_active = active_workers as f64;

    ExperimentStats {
        total_runtime,
        makespan,
        total_completed,
        avg_wait,
        avg_turnaround,
        avg_io_wait,
        avg_cpu_wait,
        cpu_completed,
        io_completed,
        max_wait: max_wait_task.wait_ms,
        max_wait_task_id: max_wait_task.task_id,
        max_queue,
        avg_queue,
        utilization,
        avg_workers_active,
        monitor_samples,
        worker_task_counts,
    }
}

fn write_experiment_results(
    file: &mut File,
    config: ExperimentConfig,
    stats: &ExperimentStats,
) {
    writeln!(file, "\n==============================").unwrap();
    writeln!(file, "Experiment: {}", config.name).unwrap();
    writeln!(file, "==============================").unwrap();

    writeln!(file, "\nSummary Metrics").unwrap();
    writeln!(file, "Total runtime: {} ms", stats.total_runtime).unwrap();
    writeln!(file, "Makespan: {} ms", stats.makespan).unwrap();
    writeln!(file, "Total tasks completed: {}", stats.total_completed).unwrap();
    writeln!(file, "Average wait time: {:.2} ms", stats.avg_wait).unwrap();
    writeln!(
        file,
        "Average turnaround time: {:.2} ms",
        stats.avg_turnaround
    )
    .unwrap();
    writeln!(file, "CPU tasks completed: {}", stats.cpu_completed).unwrap();
    writeln!(file, "IO tasks completed: {}", stats.io_completed).unwrap();
    writeln!(file, "Max wait time: {} ms", stats.max_wait).unwrap();
    writeln!(file, "Max queue length: {}", stats.max_queue).unwrap();
    writeln!(file, "Average queue length: {:.2}", stats.avg_queue).unwrap();
    writeln!(file, "Worker utilization: {:.2}%", stats.utilization).unwrap();

    writeln!(file, "\nWorker Task Summary").unwrap();

    for (id, count) in stats.worker_task_counts.iter().enumerate() {
        writeln!(file, "Worker {} completed {} tasks", id, count).unwrap();
    }
}

fn write_output_example(
    file_name: &str,
    title: &str,
    config: ExperimentConfig,
    stats: &ExperimentStats,
) {
    let mut file = File::create(file_name).unwrap();

    let io_percent = 100 - config.cpu_percent;

    writeln!(file, "== {} ==", title).unwrap();
    writeln!(
        file,
        "{} tasks, {}% IO / {}% CPU, {} workers, cap 100%",
        config.total_tasks, io_percent, config.cpu_percent, config.workers
    )
    .unwrap();

    writeln!(file).unwrap();
    writeln!(file, "— results —").unwrap();

    writeln!(file, "{:<24}: {} ms", "total runtime", stats.total_runtime).unwrap();
    writeln!(file, "{:<24}: {} ms", "makespan", stats.makespan).unwrap();

    writeln!(
        file,
        "{:<24}: {} (IO={}, CPU={})",
        "tasks completed",
        stats.total_completed,
        stats.io_completed,
        stats.cpu_completed
    )
    .unwrap();

    writeln!(file, "{:<24}: {:.2} ms", "avg wait time", stats.avg_wait).unwrap();

    if title.contains("Optimized") {
        writeln!(
            file,
            "{:<24}: {:.2} ms",
            "avg wait (IO only)",
            stats.avg_io_wait
        )
        .unwrap();

        writeln!(
            file,
            "{:<24}: {:.2} ms",
            "avg wait (CPU only)",
            stats.avg_cpu_wait
        )
        .unwrap();
    }

    writeln!(
        file,
        "{:<24}: {:.2} ms",
        "avg turnaround time",
        stats.avg_turnaround
    )
    .unwrap();

    if title.contains("Optimized") {
        writeln!(
            file,
            "{:<24}: {} ms (task #{})",
            "max wait time",
            stats.max_wait,
            stats.max_wait_task_id
        )
        .unwrap();
    } else {
        writeln!(file, "{:<24}: {} ms", "max wait time", stats.max_wait).unwrap();
    }

    writeln!(file, "{:<24}: {:.2} %", "avg CPU usage", stats.utilization).unwrap();

    writeln!(
        file,
        "{:<24}: {:.2} / {}",
        "avg workers active",
        stats.avg_workers_active,
        config.workers
    )
    .unwrap();

    writeln!(file, "{:<24}: {}", "monitor samples", stats.monitor_samples).unwrap();
    writeln!(file, "{:<24}: {}", "monitor csv", "monitor_log.csv").unwrap();
}

fn main() {
    let mut experiment_file = File::create("experiment_results.txt").unwrap();

    let balanced = ExperimentConfig {
        name: "Balanced workload",
        total_tasks: 500,
        workers: 6,
        cpu_percent: 50,
        bursty: false,
        policy: SchedulingPolicy::WeightedRoundRobin,
    };

    let stressed = ExperimentConfig {
        name: "Stressed CPU-heavy burst workload",
        total_tasks: 500,
        workers: 6,
        cpu_percent: 85,
        bursty: true,
        policy: SchedulingPolicy::WeightedRoundRobin,
    };

    let fifo_example = ExperimentConfig {
        name: "FIFO output example",
        total_tasks: 1000,
        workers: 8,
        cpu_percent: 30,
        bursty: false,
        policy: SchedulingPolicy::Fifo,
    };

    let optimized_example = ExperimentConfig {
        name: "Optimized output example",
        total_tasks: 1000,
        workers: 8,
        cpu_percent: 30,
        bursty: false,
        policy: SchedulingPolicy::WeightedRoundRobin,
    };

    let balanced_stats = run_experiment(balanced);
    write_experiment_results(&mut experiment_file, balanced, &balanced_stats);

    let stressed_stats = run_experiment(stressed);
    write_experiment_results(&mut experiment_file, stressed, &stressed_stats);

    let fifo_stats = run_experiment(fifo_example);
    write_output_example(
        "fifo_output_example.txt",
        "FIFO simulation",
        fifo_example,
        &fifo_stats,
    );

    let optimized_stats = run_experiment(optimized_example);
    write_output_example(
        "optimized_output_example.txt",
        "Optimized simulation",
        optimized_example,
        &optimized_stats,
    );

    println!("\nResults written to:");
    println!("experiment_results.txt");
    println!("fifo_output_example.txt");
    println!("optimized_output_example.txt");
}