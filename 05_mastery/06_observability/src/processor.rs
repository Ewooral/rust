//! The mini project: an instrumented job processor.
//!
//! Every job runs inside a `job` span carrying its id and kind, so every
//! line logged while working on it, at any depth, says which job it was
//! about. That matters because jobs run concurrently and their log lines
//! interleave.
//!
//! Levels are chosen for the person reading them at 3 a.m.:
//! - `ERROR`: a job failed for good: someone may need to act;
//! - `WARN`: something went wrong but was handled (a retry);
//! - `INFO`: the story of the batch: started, each job done, finished;
//! - `DEBUG`: each attempt (a span of its own);
//! - `TRACE`: every attempt's raw result.

use std::time::Duration;

use tracing::{Instrument, debug, error, field, info, instrument, trace, warn};

use crate::jobs::{Job, JobError, run_once};

/// Runs jobs, retrying temporary failures.
#[derive(Debug, Clone)]
pub struct JobProcessor {
    max_attempts: u32,
    work_time: Duration,
}

/// What happened to a batch.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BatchReport {
    /// Jobs that finished.
    pub succeeded: u32,
    /// Jobs that failed for good.
    pub failed: u32,
}

impl JobProcessor {
    /// A processor that tries each job at most `max_attempts` times.
    pub fn new(max_attempts: u32) -> JobProcessor {
        JobProcessor {
            max_attempts,
            work_time: Duration::from_millis(5),
        }
    }

    /// Runs one job, with retries.
    ///
    /// `#[instrument]` opens a span called `job` for the whole call, also
    /// across every `.await`. `skip(self, job)` keeps the arguments out of
    /// the span (`job` is recorded through chosen fields instead), and
    /// `attempts = field::Empty` declares a field filled in later.
    #[instrument(
        name = "job",
        skip(self, job),
        fields(job.id = job.id, job.kind = %job.kind, attempts = field::Empty)
    )]
    pub async fn process(&self, job: &Job) -> Result<String, JobError> {
        let mut attempt = 1;
        loop {
            let result = self.attempt(job, attempt).await;
            match result {
                Ok(summary) => {
                    // Record a field ONCE: the fmt layer appends every
                    // record() it sees ("attempts=1 attempts=2").
                    tracing::Span::current().record("attempts", attempt);
                    info!(outcome = "succeeded", %summary, "job done");
                    return Ok(summary);
                }
                Err(e) if e.is_retryable() && attempt < self.max_attempts => {
                    warn!(error = %e, attempt, "attempt failed, retrying");
                    attempt += 1;
                }
                Err(e) => {
                    tracing::Span::current().record("attempts", attempt);
                    error!(outcome = "failed", error = %e, "job failed");
                    return Err(e);
                }
            }
        }
    }

    /// One attempt: a DEBUG-level span, so it only exists when DEBUG is on.
    /// `attempt` isn't skipped, so it becomes a field of the span.
    #[instrument(level = "debug", skip(self, job))]
    async fn attempt(&self, job: &Job, attempt: u32) -> Result<String, JobError> {
        debug!("working");
        tokio::time::sleep(self.work_time).await; // pretend to do I/O
        let result = run_once(job, attempt);
        trace!(?result, "attempt finished");
        result
    }

    /// Runs every job concurrently, each in its own task.
    #[instrument(skip(self, jobs), fields(jobs = jobs.len()))]
    pub async fn run_batch(&self, jobs: Vec<Job>) -> BatchReport {
        info!("batch started");
        let handles: Vec<_> = jobs
            .into_iter()
            .map(|job| {
                let processor = self.clone();
                let task = async move { processor.process(&job).await };
                // A spawned task does NOT inherit the current span.
                // `in_current_span()` attaches it, so each `job` span is a
                // child of this `run_batch` span.
                tokio::spawn(task.in_current_span())
            })
            .collect();

        let mut report = BatchReport::default();
        for handle in handles {
            match handle.await.expect("a job task panicked") {
                Ok(_) => report.succeeded += 1,
                Err(_) => report.failed += 1,
            }
        }
        info!(
            succeeded = report.succeeded,
            failed = report.failed,
            "batch finished"
        );
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::CapturedLogs;
    use crate::jobs::{JobKind, sample_jobs};
    use tracing::subscriber::set_default;

    /// A subscriber that writes plain, timestamp-free lines into `logs`.
    fn capture(logs: &CapturedLogs, level: tracing::Level) -> impl tracing::Subscriber {
        tracing_subscriber::fmt()
            .with_writer(logs.clone())
            .with_max_level(level)
            .with_ansi(false)
            .without_time()
            .finish()
    }

    fn report_job() -> Job {
        Job {
            id: 7,
            kind: JobKind::BuildReport { rows: 9_000 },
        }
    }

    // #[tokio::test] runs on ONE thread, so `set_default` (thread-local)
    // covers the whole test, spawned tasks included.
    #[tokio::test]
    async fn a_retry_is_a_warning_inside_the_job_span() {
        let logs = CapturedLogs::default();
        let _guard = set_default(capture(&logs, tracing::Level::INFO));

        let result = JobProcessor::new(3).process(&report_job()).await;

        assert_eq!(result, Ok("report of 9000 rows".into()));
        let lines = logs.lines();
        assert_eq!(lines.len(), 2, "{lines:#?}");
        // `attempts` is Empty until recorded, and empty fields aren't shown.
        assert!(
            lines[0].starts_with(" WARN job{job.id=7 job.kind=report}: observability::processor:")
        );
        assert!(lines[0].ends_with(
            "attempt failed, retrying error=temporary failure: database busy attempt=1"
        ));
        assert!(
            lines[1].contains("job{job.id=7 job.kind=report attempts=2}"),
            "{}",
            lines[1]
        );
        assert!(
            lines[1].contains("job done outcome=\"succeeded\""),
            "{}",
            lines[1]
        );
    }

    #[tokio::test]
    async fn giving_up_is_an_error() {
        let logs = CapturedLogs::default();
        let _guard = set_default(capture(&logs, tracing::Level::INFO));

        let result = JobProcessor::new(1).process(&report_job()).await;

        assert!(result.is_err());
        assert!(
            logs.contents().starts_with("ERROR job{"),
            "{}",
            logs.contents()
        );
    }

    #[tokio::test]
    async fn debug_level_adds_the_attempt_spans() {
        let logs = CapturedLogs::default();
        let _guard = set_default(capture(&logs, tracing::Level::DEBUG));

        JobProcessor::new(3).process(&report_job()).await.unwrap();

        let contents = logs.contents();
        assert!(contents.contains("job{job.id=7 job.kind=report}:attempt{attempt=1}: "));
        assert!(contents.contains("attempt{attempt=2}: "), "{contents}");
        assert!(!contents.contains("TRACE"), "TRACE is still off");
    }

    #[tokio::test]
    async fn a_batch_counts_and_logs_every_job() {
        let logs = CapturedLogs::default();
        let _guard = set_default(capture(&logs, tracing::Level::INFO));

        let report = JobProcessor::new(3).run_batch(sample_jobs()).await;

        assert_eq!(
            report,
            BatchReport {
                succeeded: 4,
                failed: 2
            }
        );
        let contents = logs.contents();
        assert_eq!(contents.matches("job done").count(), 4);
        assert_eq!(contents.matches("job failed").count(), 2);
        // Spawned tasks kept their parent span thanks to in_current_span().
        assert!(contents.contains("run_batch{jobs=6}:job{job.id=104 job.kind=email attempts=1}"));
    }
}
