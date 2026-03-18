/*
 * CAUTION: This is not meant as a replacement for your existing key.rs
 *          It just contains additional code that you should copy into your own file.
 */

use nolock::queues::mpmc;

/// Represents a first in first out queue for key events.
/// It uses a multi-producer multi-consumer queue from the nolock crate,
/// allowing thread safe access without needing a Lock.
pub struct KeyEventQueue {
    /// Keys can be popped from the queue via the receiver.
    receiver:  mpmc::bounded::scq::Receiver<KeyEvent>,
    /// Keys can be pushed to the queue via the sender.
    sender:  mpmc::bounded::scq::Sender<KeyEvent>
}

impl KeyEventQueue {
    /// Create a new empty queue.
    /// Unfortunately, this cannot be done in a const function.
    pub fn new() -> KeyEventQueue {
        let (receiver, sender) = mpmc::bounded::scq::queue(128);
        KeyEventQueue { receiver, sender }
    }

    /// Push a key to the queue.
    /// If the queue is full, the key is silently discarded.
    pub fn push_key_event(&self, key: KeyEvent) {
        if self.receiver.is_closed() {
            // Should never haven
            panic!("KeyQueue is closed!");
        }

        // Enqueue the key into the queue.
        // If the queue is full, we ignore the key.
        self.sender.try_enqueue(key).ok();
    }

    /// Pop a key from the queue.
    /// If the queue is empty, None is returned.
    pub fn pop_key_event(&self) -> Option<KeyEvent> {
        if self.receiver.is_closed() {
            // Should never haven
            panic!("KeyQueue is closed!");
        }

        match self.receiver.try_dequeue() {
            Ok(key) => Some(key),
            Err(_) => None
        }
    }

    /// Pop a key event from the queue.
    /// If the queue is empty, the function blocks until a key is available.
    pub fn poll_key_event(&self) -> KeyEvent {
        if self.receiver.is_closed() {
            // Should never haven
            panic!("KeyQueue is closed!");
        }

        loop {
            match self.receiver.try_dequeue() {
                Ok(key) => return key,
                Err(_) => {}
            }
        }
    }

    /// Pop key events from the queue until a key press event is found.
    /// The function blocks until a key press event is available.
    pub fn poll_key_press(&self) -> KeyEvent {
        loop {
            let key = self.poll_key_event();
            if key.pressed() {
                return key;
            }
        }
    }

    /// Poll key presses from the queue until a key with an ascii code is found.
    /// The function blocks until a key with an ascii code is available.
    pub fn poll_char(&self) -> char {
        loop {
            let key = self.poll_key_press();
            if let Some(c) = key.ascii() {
                return c;
            }
        }
    }
}