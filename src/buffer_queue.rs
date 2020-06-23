/*
A queue of persistent buffers, to be (re)used by scoped worker threads.
*/
use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

const QUERY_INTERVAL_MS: u64 = 250;

pub struct BufferQueue {
    buffers: Vec<Mutex<String>>,
    queue: Mutex<VecDeque<usize>>,
}

impl BufferQueue {
    pub fn new(size: usize, buffer_size: usize) -> Self {
        let mut buffers = Vec::with_capacity(size);
        for _ in 0..size {
            buffers.push(Mutex::new(String::with_capacity(buffer_size)));
        }

        let mut queue = VecDeque::with_capacity(size);
        queue.extend(0..size);

        Self {
            buffers,
            queue: Mutex::new(queue),
        }
    }

    pub fn pop(&self) -> Buffer {
        let id = self.await_next_id();
        Buffer {
            id,
            inner: &self.buffers[id],
            queue: &self.queue,
        }
    }

    fn await_next_id(&self) -> usize {
        loop {
            if let Ok(mut queue) = self.queue.try_lock() {
                if let Some(id) = queue.pop_front() {
                    return id;
                }
            }
            thread::sleep(Duration::from_millis(QUERY_INTERVAL_MS));
        }
    }
}

/*
A Buffer gets an ID from the queue, along with a reference into the corresponding buffers index.
Once released, it pushes its ID back into the shared queue, allowing the resource to be reclaimed.
*/
pub struct Buffer<'a> {
    id: usize,
    inner: &'a Mutex<String>,
    queue: &'a Mutex<VecDeque<usize>>,
}

impl<'a> Buffer<'a> {
    pub fn borrow(&self) -> MutexGuard<String> {
        self.inner.lock().unwrap()
    }

    pub fn release(&self) {
        self.queue.lock().unwrap().push_back(self.id);
    }
}
