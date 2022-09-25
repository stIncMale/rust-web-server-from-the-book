#![deny(
    warnings,
    nonstandard_style,
    future_incompatible,
    unused_qualifications,
    clippy::all,
    clippy::perf,
    clippy::pedantic,
    clippy::cargo,
    clippy::std_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::alloc_instead_of_core,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_html_tags
)]
#![allow(
    clippy::similar_names,
    clippy::cast_possible_truncation,
    // uncomment below to simplify editing, comment out again before committing
    // clippy::pedantic,
    // unused_imports,
    // unused_variables,
    // unused_mut,
    // unreachable_code,
    // dead_code,
)]

extern crate alloc;

use alloc::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::{sync::mpsc, thread};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
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
    /// If the `size` is 0.
    #[must_use]
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        Self {
            workers,
            sender: Some(sender),
        }
    }

    /// # Panics
    ///
    /// If [sending](Sender::send()) `f` to a worker errors.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.as_ref().unwrap().send(Box::new(f)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutting down worker {}.", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

type Job = Box<dyn FnOnce() + Send>;

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            if let Ok(job) = receiver.lock().unwrap().recv() {
                println!("Worker {id} got a job; executing.");
                job();
                println!("Worker {id} completed the job.");
            } else {
                println!("Worker {id} disconnected; shutting down.");
                break;
            }
        });
        Self {
            id,
            thread: Some(thread),
        }
    }
}
