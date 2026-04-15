use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use rand::RngExt;

// Define a special value that will signal termination
const TERMINATION_SIGNAL: i32 = -1;

fn main() {
    // Number of items to produce
    const ITEM_COUNT: usize = 20;
    const NUM_PRODUCERS: usize = 2;
    const NUM_CONSUMERS: usize = 3;

    
    // TODO: Create a channel for sending numbers
    let (tx, rx) = mpsc::channel();

    let shared_rx = Arc::new(Mutex::new(rx));

    let mut producer_handles = Vec::new();
    let mut consumer_handles = Vec::new();
    
    // TODO: Create 2 producer threads
    for id in 0..NUM_PRODUCERS {
        let tx_clone = tx.clone();
        let items_per_producer = ITEM_COUNT / NUM_PRODUCERS;

        let handle = thread::spawn(move || {
            producer(id, tx_clone, items_per_producer);
        });

        producer_handles.push(handle);
    }

    
    // TODO: Create 3 consumer threads
    for id in 0..NUM_CONSUMERS {
        let rx_clone = Arc::clone(&shared_rx);

        let handle = thread::spawn(move || {
            consumer(id, rx_clone);
        });

        consumer_handles.push(handle);
    }

    
    // TODO: Wait for all threads to finish
    for handle in producer_handles {
        handle.join().unwrap();
    }

    for _ in 0..NUM_CONSUMERS {
        tx.send(TERMINATION_SIGNAL).unwrap();
    }

    for handle in consumer_handles {
        handle.join().unwrap();
    }
    
    println!("All items have been produced and consumed!");
}

// TODO: Implement producer function
fn producer(id: usize, tx: mpsc::Sender<i32>, item_count: usize) {
    // TODO: Generate random numbers and send them to the channel
    let mut rng = rand::rng();

    for _ in 0..item_count {
        let num = rng.random_range(1..=100);
        println!("Producer {} produced {}", id, num);
        tx.send(num).unwrap();
        thread::sleep(Duration::from_millis(200));
    }

    println!("Producer {} finished producing.", id);
    // When finished, producer should NOT send termination signal
}

// TODO: Implement consumer function
fn consumer(id: usize, rx: Arc<Mutex<mpsc::Receiver<i32>>>) {
    // TODO: Receive numbers from the channel and process them
    loop {
        let value = {
            let receiver = rx.lock().unwrap();
            receiver.recv().unwrap()
        };

        if value == TERMINATION_SIGNAL {
            println!("Consumer {} received termination signal and is exiting.", id);
            break;
        } else {
            println!("Consumer {} consumed {}", id, value);
            thread::sleep(Duration::from_millis(300));
        }
    }
    // Break the loop when receiving the termination signal
}