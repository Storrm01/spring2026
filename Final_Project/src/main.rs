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

#[derive(Clone, Debug)]
struct Task {
    id: usize,
    kind: TaskKind,
    duration: u64,
    arrived_at: Instant,
}

#[derive(Debug)]
struct CompletedTask {
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
}

fn generate_task(
    id: usize,
    config: ExperimentConfig,
    rng: &mut StdRng,
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
        kind,
        duration,
        arrived_at: Instant::now(),
    }
}

fn run_experiment(config: ExperimentConfig, file: &mut File) {
    println!("Running experiment: {}", config.name);

    writeln!(file, "\n==============================").unwrap();
    writeln!(file, "Experiment: {}", config.name).unwrap();
    writeln!(file, "==============================").unwrap();

    let start_time = Instant::now();

    let (arrival_sender, arrival_receiver) = mpsc::channel::<Task>();
    let (work_sender, work_receiver) = mpsc::channel::<Option<Task>>();
    let (completed_sender, completed_receiver) = mpsc::channel::<CompletedTask>();

    let shared_work_receiver = Arc::new(Mutex::new(work_receiver));

    let generator_config = config;

    let generator = thread::spawn(move || {
        let mut rng = StdRng::seed_from_u64(42);

        for id in 0..generator_config.total_tasks {
            let task = generate_task(id, generator_config, &mut rng);

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

    let dispatcher = thread::spawn(move || {
        let mut cpu_queue: VecDeque<Task> = VecDeque::new();
        let mut io_queue: VecDeque<Task> = VecDeque::new();

        let mut cpu_turns = 0;
        let mut queue_samples = Vec::new();
        let mut generator_done = false;

        while !generator_done || !cpu_queue.is_empty() || !io_queue.is_empty() {
            match arrival_receiver.recv_timeout(Duration::from_millis(5)) {
                Ok(task) => match task.kind {
                    TaskKind::Cpu => cpu_queue.push_back(task),
                    TaskKind::Io => io_queue.push_back(task),
                },

                Err(mpsc::RecvTimeoutError::Timeout) => {}

                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    generator_done = true;
                }
            }

            queue_samples.push(cpu_queue.len() + io_queue.len());

            let next_task = if cpu_turns < 2 && !cpu_queue.is_empty() {
                cpu_turns += 1;
                cpu_queue.pop_front()
            } else if !io_queue.is_empty() {
                cpu_turns = 0;
                io_queue.pop_front()
            } else if !cpu_queue.is_empty() {
                cpu_queue.pop_front()
            } else {
                None
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

        (max_queue, avg_queue)
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
                    let _task_id = task.id;

                    let start = Instant::now();

                    let wait_ms = start.duration_since(task.arrived_at).as_millis();

                    thread::sleep(Duration::from_millis(task.duration));

                    let turnaround_ms = Instant::now()
                        .duration_since(task.arrived_at)
                        .as_millis();

                    let completed = CompletedTask {
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

    let (max_queue, avg_queue) = dispatcher.join().unwrap();

    let makespan = start_time.elapsed().as_millis();
    let total_completed = results.len();

    let avg_wait =
        results.iter().map(|r| r.wait_ms).sum::<u128>() as f64 / total_completed as f64;

    let avg_turnaround =
        results.iter().map(|r| r.turnaround_ms).sum::<u128>() as f64 / total_completed as f64;

    let cpu_completed = results.iter().filter(|r| r.kind == TaskKind::Cpu).count();

    let io_completed = results.iter().filter(|r| r.kind == TaskKind::Io).count();

    let max_wait = results.iter().map(|r| r.wait_ms).max().unwrap_or(0);

    let total_work_time: u64 = results.iter().map(|r| r.duration_ms).sum();

    let possible_work_time = makespan as f64 * config.workers as f64;

    let utilization = total_work_time as f64 / possible_work_time * 100.0;

    writeln!(file, "\nSummary Metrics").unwrap();
    writeln!(file, "Total tasks completed: {}", total_completed).unwrap();
    writeln!(file, "Makespan: {} ms", makespan).unwrap();
    writeln!(file, "Average wait time: {:.2} ms", avg_wait).unwrap();
    writeln!(
        file,
        "Average turnaround time: {:.2} ms",
        avg_turnaround
    )
    .unwrap();
    writeln!(file, "CPU tasks completed: {}", cpu_completed).unwrap();
    writeln!(file, "IO tasks completed: {}", io_completed).unwrap();
    writeln!(file, "Max wait time: {} ms", max_wait).unwrap();
    writeln!(file, "Max queue length: {}", max_queue).unwrap();
    writeln!(file, "Average queue length: {:.2}", avg_queue).unwrap();
    writeln!(file, "Worker utilization: {:.2}%", utilization).unwrap();

    writeln!(file, "\nWorker Task Summary").unwrap();

    for (id, count) in worker_task_counts.iter().enumerate() {
        writeln!(file, "Worker {} completed {} tasks", id, count).unwrap();
    }

    println!("Finished experiment: {}", config.name);
}

fn main() {
    let mut file = File::create("experiment_results.txt").unwrap();

    let balanced = ExperimentConfig {
        name: "Balanced workload",
        total_tasks: 500,
        workers: 6,
        cpu_percent: 50,
        bursty: false,
    };

    let stressed = ExperimentConfig {
        name: "Stressed CPU-heavy burst workload",
        total_tasks: 500,
        workers: 6,
        cpu_percent: 85,
        bursty: true,
    };

    run_experiment(balanced, &mut file);
    run_experiment(stressed, &mut file);

    println!("\nResults written to experiment_results.txt");
}