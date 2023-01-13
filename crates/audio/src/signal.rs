use std::{
    sync::atomic::{AtomicBool, Ordering::SeqCst}, task::Waker
};

use parking_lot::Mutex;

pub trait Signal: Send + Sync {
    fn fire(&self);
}

pub struct AsyncSignal {
    waker: Mutex<Waker>,
    woken: AtomicBool,
}

impl AsyncSignal {
    pub fn new(waker: Waker) -> Self {
        Self {
            waker: Mutex::new(waker),
            woken: AtomicBool::new(false),
        }
    }

    pub fn is_woken(&self) -> bool {
        self.woken.load(SeqCst)
    }

    pub fn set_waker(&self, waker: Waker) {
        *self.waker.lock() = waker;
    }
}

impl Signal for AsyncSignal {
    fn fire(&self) {
        self.woken.store(true, SeqCst);
        self.waker.lock().wake_by_ref();
    }
}

pub struct BlockingSignal {
    thread: std::thread::Thread,
}

impl BlockingSignal {
    pub fn new(thread: std::thread::Thread) -> Self {
        Self { thread }
    }
}

impl Signal for BlockingSignal {
    fn fire(&self) {
        self.thread.unpark()
    }
}
