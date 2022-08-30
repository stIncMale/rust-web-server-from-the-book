#![deny(
    warnings,
    nonstandard_style,
    future_incompatible,
    unused_qualifications,
    clippy::all,
    clippy::perf,
    clippy::pedantic,
    clippy::cargo,
    // TODO uncomment in Clippy 1.64
    // clippy::std_instead_of_core,
    // clippy::std_instead_of_alloc,
    // clippy::alloc_instead_of_core,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_html_tags,
)]
#![allow(
    clippy::similar_names,
    clippy::cast_possible_truncation,
    // uncomment below to simplify editing, comment out again before committing
    clippy::pedantic,
    unused_imports,
    unused_variables,
    unused_mut,
    unreachable_code,
    dead_code,
)]

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{sync::mpsc, thread};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Job>,
}

impl ThreadPool {
    /// Creates a new [`ThreadPool`].
    ///
    /// # Parameters
    ///
    /// * `size` - The positive number of threads in the pool.
    ///
    /// # Panics
    ///
    /// Panics if the `size` is 0.
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        Self { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(f)).unwrap();
    }
}

type Job = Box<dyn FnOnce() + Send>;

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            println!("Worker {id} got the job; executing.");
            job();
        });
        Self { id, thread }
    }
}
