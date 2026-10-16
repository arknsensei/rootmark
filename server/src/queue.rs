//! One writer queue per vault (K-D1.4). Every call that writes — the
//! vault (`propose`, `promote`, `checkpoint`, watched edits, compaction) or
//! a consumption trace (`search`, `read_block`) — takes a ticket when it
//! arrives and runs when the tickets before it are done, one at a time, in
//! arrival order, whatever the transport (MCP and HTTP share the queue). A
//! pipelining client therefore gets the same log as a lock-step one.
//! Read-only calls take no ticket; they run in parallel with each other,
//! and never while a writer runs (`gate`).
//!
//! A ticket that is dropped without running (a request answered before it
//! reached its tool) is skipped, so it never blocks the queue.

use std::collections::BTreeSet;
use std::sync::{Arc, Condvar, Mutex, RwLock};

#[derive(Default)]
struct State {
    /// the next ticket to hand out
    next: u64,
    /// the ticket whose turn it is
    serving: u64,
    /// tickets dropped before their turn
    skipped: BTreeSet<u64>,
}

#[derive(Default)]
pub struct Queue {
    state: Mutex<State>,
    turn: Condvar,
    /// writers hold it exclusively, readers shared
    gate: RwLock<()>,
}

/// A place in the queue, taken at arrival.
pub struct Ticket {
    queue: Arc<Queue>,
    n: u64,
    used: bool,
}

/// The running writer's turn; dropping it lets the next ticket run.
pub struct Turn<'a> {
    queue: &'a Queue,
    n: u64,
    _gate: std::sync::RwLockWriteGuard<'a, ()>,
}

/// A reader's shared access.
pub type Read<'a> = std::sync::RwLockReadGuard<'a, ()>;

impl Queue {
    pub fn new() -> Arc<Queue> {
        Arc::new(Queue::default())
    }

    /// A ticket at the end of the queue.
    pub fn ticket(self: &Arc<Self>) -> Ticket {
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        let n = s.next;
        s.next += 1;
        Ticket {
            queue: self.clone(),
            n,
            used: false,
        }
    }

    /// Shared access for a read-only call.
    pub fn read(&self) -> Read<'_> {
        self.gate.read().unwrap_or_else(|p| p.into_inner())
    }

    /// Marks `n` done and advances past skipped tickets.
    fn done(&self, n: u64) {
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        if s.serving == n {
            s.serving += 1;
            loop {
                let next = s.serving;
                if !s.skipped.remove(&next) {
                    break;
                }
                s.serving += 1;
            }
        } else {
            s.skipped.insert(n);
        }
        self.turn.notify_all();
    }
}

impl Ticket {
    /// The ticket's number (arrival order).
    pub fn number(&self) -> u64 {
        self.n
    }

    /// Runs `f` in this ticket's turn: after every earlier ticket, alone.
    pub fn run<T>(mut self, f: impl FnOnce() -> T) -> T {
        self.used = true;
        let q = self.queue.clone();
        let n = self.n;
        {
            let mut s = q.state.lock().unwrap_or_else(|p| p.into_inner());
            while s.serving != n {
                s = q.turn.wait(s).unwrap_or_else(|p| p.into_inner());
            }
        }
        let turn = Turn {
            queue: &q,
            n,
            _gate: q.gate.write().unwrap_or_else(|p| p.into_inner()),
        };
        let out = f();
        drop(turn);
        out
    }
}

impl Drop for Turn<'_> {
    fn drop(&mut self) {
        self.queue.done(self.n);
    }
}

impl Drop for Ticket {
    fn drop(&mut self) {
        if !self.used {
            self.queue.done(self.n);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn tickets_run_in_arrival_order() {
        let q = Queue::new();
        let log = Arc::new(Mutex::new(Vec::new()));
        let tickets: Vec<Ticket> = (0..20).map(|_| q.ticket()).collect();
        let mut hs = Vec::new();
        // started in reverse order, they still run 0, 1, 2, …
        for t in tickets.into_iter().rev() {
            let log = log.clone();
            hs.push(std::thread::spawn(move || {
                let n = t.number();
                t.run(|| {
                    std::thread::sleep(Duration::from_millis(1));
                    log.lock().unwrap().push(n)
                })
            }));
        }
        for h in hs {
            h.join().unwrap();
        }
        assert_eq!(*log.lock().unwrap(), (0..20).collect::<Vec<u64>>());
    }

    #[test]
    fn dropped_tickets_are_skipped() {
        let q = Queue::new();
        let a = q.ticket();
        let b = q.ticket();
        let c = q.ticket();
        drop(b);
        assert_eq!(a.run(|| 1), 1);
        assert_eq!(c.run(|| 3), 3);
        let d = q.ticket();
        drop(d);
        assert_eq!(q.ticket().run(|| 5), 5);
    }

    #[test]
    fn readers_share_and_wait_for_a_writer() {
        let q = Queue::new();
        let r1 = q.read();
        let r2 = q.read();
        drop((r1, r2));
        let t = q.ticket();
        let q2 = q.clone();
        let out = t.run(move || {
            // a reader cannot enter while the writer runs
            q2.gate.try_read().is_err()
        });
        assert!(out);
    }
}
