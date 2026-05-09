// Bounded thread-pool helper for fan-out work (e.g. GitHub API reads).
//
// Uses std::thread::scope so we can borrow `forge` and other shared resources without
// `Arc` plumbing or an async runtime — consistent with the spec's "ureq, sync, no async".

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;

/// Default worker count for HTTP fan-out. 8 is polite to GitHub's rate limiter while
/// still a clear win over fully sequential calls.
pub const HTTP_WORKERS: usize = 8;

/// Apply `f` to each item across at most `workers` threads. Returns results in **input
/// order** — the helper assigns each thread a slot and writes into it, then collects.
///
/// `f` runs on a worker thread so it must be `Send + Sync`. The closure receives a
/// reference to the input item to avoid moving it.
pub fn parallel_map<T, R, F>(items: Vec<T>, workers: usize, f: F) -> Vec<R>
where
    T: Send + Sync,
    R: Send,
    F: Fn(&T) -> R + Send + Sync,
{
    let n = items.len();
    if n == 0 {
        return Vec::new();
    }
    let workers = workers.min(n).max(1);

    // (index, value) pairs let us reassemble in input order at the end.
    let next = AtomicUsize::new(0);
    let (tx, rx) = mpsc::channel::<(usize, R)>();

    thread::scope(|s| {
        for _ in 0..workers {
            let tx = tx.clone();
            let next = &next;
            let items = &items;
            let f = &f;
            s.spawn(move || loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= n {
                    break;
                }
                let r = f(&items[i]);
                let _ = tx.send((i, r));
            });
        }
        drop(tx);
    });

    let mut buf: Vec<Option<R>> = (0..n).map(|_| None).collect();
    for (i, r) in rx {
        buf[i] = Some(r);
    }
    buf.into_iter()
        .map(|x| x.expect("worker produced result"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn empty_input_returns_empty() {
        let out: Vec<i32> = parallel_map(Vec::<i32>::new(), 4, |x| *x * 2);
        assert!(out.is_empty());
    }

    #[test]
    fn single_worker_preserves_order() {
        let out: Vec<i32> = parallel_map((0..10).collect(), 1, |x| *x);
        assert_eq!(out, (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn many_workers_preserve_input_order() {
        let out: Vec<i32> = parallel_map((0..100).collect(), 16, |x| *x * 3);
        let expected: Vec<i32> = (0..100).map(|x| x * 3).collect();
        assert_eq!(out, expected);
    }

    #[test]
    fn worker_count_capped_at_input_length() {
        // Should not panic or hang when workers > items.
        let out: Vec<i32> = parallel_map(vec![1, 2, 3], 100, |x| *x);
        assert_eq!(out, vec![1, 2, 3]);
    }

    #[test]
    fn workers_run_concurrently() {
        // If workers run sequentially this would take ~10*50ms = 500ms.
        // With 4+ workers it should comfortably finish in under 200ms.
        let counter = AtomicUsize::new(0);
        let start = std::time::Instant::now();
        let out: Vec<usize> = parallel_map((0..10).collect(), 4, |_| {
            std::thread::sleep(Duration::from_millis(50));
            counter.fetch_add(1, Ordering::Relaxed)
        });
        let elapsed = start.elapsed();
        assert_eq!(out.len(), 10);
        assert_eq!(counter.load(Ordering::Relaxed), 10);
        assert!(elapsed < Duration::from_millis(300), "took {:?}", elapsed);
    }
}
