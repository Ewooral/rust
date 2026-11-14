//! Mini project for Book chapter 16: a parallel word counter.
//!
//! The same job, counting every word in a pile of documents, done five ways:
//!
//! 1. `count_sequential`: one thread, the baseline.
//! 2. `count_with_worker_pool`: a hand-built worker pool. Jobs go out through
//!    ONE channel whose receiver the workers share behind `Arc<Mutex<_>>`,
//!    and results come back through a second channel with many producers.
//! 3. `count_with_shared_map`: `thread::spawn`, `Arc<[String]>` for the
//!    documents and `Arc<Mutex<WordCounts>>` for the shared answer.
//! 4. `count_with_scoped_threads`: `thread::scope`, borrowing chunks of the
//!    documents directly; each thread returns its own map.
//! 5. `count_with_rayon`: `par_iter` + `reduce`.
//!
//! Every strategy must give exactly the same answer as the sequential one;
//! the tests check that.

use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

/// How many times each word appears.
pub type WordCounts = HashMap<String, usize>;

/// Counts the words in one text. Words are lowercased, and anything that
/// isn't a letter, a digit or an apostrophe separates words.
pub fn count_words(text: &str) -> WordCounts {
    let mut counts = WordCounts::new();
    for word in text
        .split(|c: char| !(c.is_alphanumeric() || c == '\''))
        .map(|w| w.trim_matches('\''))
        .filter(|w| !w.is_empty())
    {
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    counts
}

/// Adds every count in `from` into `into`.
pub fn merge(into: &mut WordCounts, from: WordCounts) {
    for (word, n) in from {
        *into.entry(word).or_insert(0) += n;
    }
}

/// The `n` most common words, most common first (ties: alphabetical).
pub fn top_words(counts: &WordCounts, n: usize) -> Vec<(&str, usize)> {
    let mut all: Vec<(&str, usize)> = counts.iter().map(|(w, c)| (w.as_str(), *c)).collect();
    all.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    all.truncate(n);
    all
}

/// Strategy 1: one thread does everything.
pub fn count_sequential(docs: &[String]) -> WordCounts {
    let mut total = WordCounts::new();
    for doc in docs {
        merge(&mut total, count_words(doc));
    }
    total
}

/// What each worker in the pool reports back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerReport {
    pub worker: usize,
    pub docs_counted: usize,
}

/// Strategy 2: a worker pool built from two channels.
///
/// - The **job channel**: main sends every document into it. The workers
///   share its single `Receiver` behind `Arc<Mutex<_>>`, so each document is
///   taken by exactly one worker.
/// - The **result channel**: every worker has its own clone of the
///   `Sender` and sends back one `(WorkerReport, WordCounts)` when the jobs
///   run out.
///
/// An `AtomicUsize` counts finished documents, as a progress meter would.
/// Returns the merged counts and one report per worker, sorted by worker id.
pub fn count_with_worker_pool(
    docs: Vec<String>,
    workers: usize,
) -> (WordCounts, Vec<WorkerReport>) {
    let workers = workers.max(1);
    let (job_tx, job_rx) = mpsc::channel::<String>();
    let job_rx = Arc::new(Mutex::new(job_rx));
    let (result_tx, result_rx) = mpsc::channel::<(WorkerReport, WordCounts)>();
    let finished = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::with_capacity(workers);
    for id in 0..workers {
        let job_rx = Arc::clone(&job_rx);
        let result_tx = result_tx.clone();
        let finished = Arc::clone(&finished);
        handles.push(thread::spawn(move || {
            let mut mine = WordCounts::new();
            let mut docs_counted = 0;
            loop {
                // The guard is a temporary, dropped at the end of THIS
                // statement, so the lock is held only while taking one job.
                // (`while let Ok(doc) = job_rx.lock().unwrap().recv()` would
                // keep it locked for the whole loop body.)
                let job = job_rx.lock().unwrap().recv();
                match job {
                    Ok(doc) => {
                        merge(&mut mine, count_words(&doc));
                        docs_counted += 1;
                        finished.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => break, // channel closed and empty: no more jobs
                }
            }
            let report = WorkerReport {
                worker: id,
                docs_counted,
            };
            result_tx
                .send((report, mine))
                .expect("main stopped listening");
        }));
    }
    // Main keeps no result sender of its own, so `result_rx` ends once
    // every worker has sent its answer and dropped its clone.
    drop(result_tx);

    let total_docs = docs.len();
    for doc in docs {
        job_tx.send(doc).expect("every worker has died");
    }
    drop(job_tx); // close the job channel: workers stop when it's empty

    let mut total = WordCounts::new();
    let mut reports = Vec::with_capacity(workers);
    for (report, counts) in result_rx {
        merge(&mut total, counts);
        reports.push(report);
    }
    for handle in handles {
        handle.join().expect("a worker panicked");
    }
    assert_eq!(finished.load(Ordering::Relaxed), total_docs);
    reports.sort_by_key(|r| r.worker);
    (total, reports)
}

/// Strategy 3: `thread::spawn` with `Arc` for shared ownership and a
/// `Mutex` around the shared answer (the Book's 16.3 pattern).
pub fn count_with_shared_map(docs: Arc<[String]>, threads: usize) -> WordCounts {
    let threads = threads.max(1);
    let total = Arc::new(Mutex::new(WordCounts::new()));
    let mut handles = Vec::with_capacity(threads);
    for t in 0..threads {
        let docs = Arc::clone(&docs);
        let total = Arc::clone(&total);
        handles.push(thread::spawn(move || {
            // Thread t takes documents t, t + threads, t + 2*threads, ...
            let mut mine = WordCounts::new();
            for doc in docs.iter().skip(t).step_by(threads) {
                merge(&mut mine, count_words(doc));
            }
            // Count locally, lock ONCE: far less waiting than locking for
            // every single word.
            merge(&mut total.lock().unwrap(), mine);
        }));
    }
    for handle in handles {
        handle.join().expect("a counting thread panicked");
    }
    // Every other Arc owner (the threads) is gone, so we can take the map
    // out of the Arc and the Mutex without cloning it.
    Arc::try_unwrap(total)
        .expect("all threads have finished")
        .into_inner()
        .expect("no thread panicked while holding the lock")
}

/// Strategy 4: scoped threads borrow chunks of `docs` directly. No `Arc`,
/// no `Mutex`, no channels: each thread RETURNS its map through its join
/// handle.
pub fn count_with_scoped_threads(docs: &[String], threads: usize) -> WordCounts {
    if docs.is_empty() {
        return WordCounts::new();
    }
    let chunk_size = docs.len().div_ceil(threads.max(1));
    thread::scope(|s| {
        let handles: Vec<_> = docs
            .chunks(chunk_size)
            .map(|chunk| s.spawn(move || count_sequential(chunk)))
            .collect();
        let mut total = WordCounts::new();
        for handle in handles {
            merge(&mut total, handle.join().expect("a scoped thread panicked"));
        }
        total
    })
}

/// Strategy 5: let rayon split the work. `reduce` merges the partial maps.
pub fn count_with_rayon(docs: &[String]) -> WordCounts {
    docs.par_iter()
        .map(|doc| count_words(doc))
        .reduce(WordCounts::new, |mut a, b| {
            merge(&mut a, b);
            a
        })
}

/// Builds `n_docs` deterministic documents of `words_per_doc` words each,
/// with a few capitals and punctuation marks to keep `count_words` honest.
/// (A tiny pseudo-random generator, so no extra crate is needed.)
pub fn sample_corpus(n_docs: usize, words_per_doc: usize) -> Vec<String> {
    const WORDS: [&str; 24] = [
        "the", "a", "thread", "channel", "mutex", "arc", "lock", "send", "sync", "data", "race",
        "rust", "fearless", "worker", "message", "share", "move", "join", "spawn", "atomic",
        "scope", "rayon", "core", "it's",
    ];
    let mut seed: u64 = 0x5EED_1234;
    let mut next = move || {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (seed >> 33) as usize
    };
    (0..n_docs)
        .map(|_| {
            let mut doc = String::new();
            for i in 0..words_per_doc {
                // Pick twice and keep the smaller index, so the early words
                // ("the", "a") come up more often, like in real text.
                let word = WORDS[(next() % WORDS.len()).min(next() % WORDS.len())];
                if i % 9 == 0 {
                    let mut chars = word.chars();
                    if let Some(first) = chars.next() {
                        doc.extend(first.to_uppercase());
                        doc.push_str(chars.as_str());
                    }
                } else {
                    doc.push_str(word);
                }
                doc.push_str(match next() % 10 {
                    0 => ". ",
                    1 => ", ",
                    _ => " ",
                });
            }
            doc
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn docs(texts: &[&str]) -> Vec<String> {
        texts.iter().map(|t| t.to_string()).collect()
    }

    #[test]
    fn count_words_lowercases_and_strips_punctuation() {
        let counts = count_words("The cat, the HAT. 'Quoted' isn't it?");
        assert_eq!(counts["the"], 2);
        assert_eq!(counts["hat"], 1);
        assert_eq!(counts["quoted"], 1);
        assert_eq!(counts["isn't"], 1);
        assert_eq!(counts.len(), 6);
    }

    #[test]
    fn merge_adds_counts() {
        let mut a = count_words("a a b");
        merge(&mut a, count_words("b c"));
        assert_eq!(top_words(&a, 3), [("a", 2), ("b", 2), ("c", 1)]);
    }

    #[test]
    fn every_strategy_agrees_with_sequential() {
        let corpus = sample_corpus(200, 50);
        let expected = count_sequential(&corpus);
        assert_eq!(expected.values().sum::<usize>(), 200 * 50);

        for workers in [1, 2, 3, 8] {
            let (pooled, reports) = count_with_worker_pool(corpus.clone(), workers);
            assert_eq!(pooled, expected, "worker pool with {workers} workers");
            assert_eq!(reports.len(), workers);
            let counted: usize = reports.iter().map(|r| r.docs_counted).sum();
            assert_eq!(counted, corpus.len(), "each document counted exactly once");

            let shared: Arc<[String]> = corpus.clone().into();
            assert_eq!(count_with_shared_map(shared, workers), expected);
            assert_eq!(count_with_scoped_threads(&corpus, workers), expected);
        }
        assert_eq!(count_with_rayon(&corpus), expected);
    }

    #[test]
    fn more_threads_than_documents_is_fine() {
        let corpus = docs(&["one two", "two"]);
        let expected = count_sequential(&corpus);
        assert_eq!(count_with_worker_pool(corpus.clone(), 16).0, expected);
        assert_eq!(count_with_shared_map(corpus.clone().into(), 16), expected);
        assert_eq!(count_with_scoped_threads(&corpus, 16), expected);
    }

    #[test]
    fn empty_input_and_zero_threads() {
        assert!(count_with_worker_pool(Vec::new(), 4).0.is_empty());
        assert!(count_with_shared_map(Vec::new().into(), 0).is_empty());
        assert!(count_with_scoped_threads(&[], 0).is_empty());
        assert!(count_with_rayon(&[]).is_empty());
        // 0 threads is treated as 1
        let corpus = docs(&["x y x"]);
        assert_eq!(count_with_scoped_threads(&corpus, 0)["x"], 2);
    }

    #[test]
    fn sample_corpus_is_deterministic() {
        assert_eq!(sample_corpus(5, 20), sample_corpus(5, 20));
    }
}
