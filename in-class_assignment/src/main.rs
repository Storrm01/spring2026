use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let total = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..5 {
        let cnt = total.clone();

        let handle = thread::spawn(move || {
            for _ in 0..10 {
                *cnt.lock().unwrap() += 1;
            }
        });

        handles.push(handle);
    }


    for handle in handles {
        handle.join().unwrap();
    }


    println!("Final counter value: {}", *total.lock().unwrap());
}