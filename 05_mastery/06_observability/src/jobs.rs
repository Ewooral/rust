//! The work the processor does: a few kinds of job, some of which fail.
//!
//! Failures are deterministic so the logs, metrics and tests are too:
//! - an email to an address containing "bounce" fails for good;
//! - an image with a zero width or height is invalid (never retried);
//! - a report of more than 5 000 rows fails its first attempt ("database
//!   busy") and succeeds on the retry.

use std::fmt;

/// One unit of work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    /// A number that identifies the job in every log line about it.
    pub id: u32,
    /// What to do.
    pub kind: JobKind,
}

/// The kinds of work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobKind {
    /// Send a welcome email.
    SendEmail {
        /// The address.
        to: String,
    },
    /// Make a thumbnail.
    ResizeImage {
        /// Target width in pixels.
        width: u32,
        /// Target height in pixels.
        height: u32,
    },
    /// Build a report from the database.
    BuildReport {
        /// How many rows it reads.
        rows: u32,
    },
}

impl fmt::Display for JobKind {
    // A short, stable name: good for log fields and metric labels.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            JobKind::SendEmail { .. } => "email",
            JobKind::ResizeImage { .. } => "resize",
            JobKind::BuildReport { .. } => "report",
        })
    }
}

/// Why a job failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobError {
    /// The job itself is wrong; retrying can't help.
    Invalid(String),
    /// Something outside failed for good.
    Permanent(String),
    /// Something outside failed this time; a retry may work.
    Transient(String),
}

impl JobError {
    /// Whether trying again could help.
    pub fn is_retryable(&self) -> bool {
        matches!(self, JobError::Transient(_))
    }
}

impl fmt::Display for JobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobError::Invalid(why) => write!(f, "invalid job: {why}"),
            JobError::Permanent(why) => write!(f, "failed permanently: {why}"),
            JobError::Transient(why) => write!(f, "temporary failure: {why}"),
        }
    }
}

impl std::error::Error for JobError {}

/// The jobs `cargo run -p observability` processes.
pub fn sample_jobs() -> Vec<Job> {
    let kinds = [
        JobKind::SendEmail {
            to: "ama@example.com".into(),
        },
        JobKind::ResizeImage {
            width: 640,
            height: 480,
        },
        JobKind::BuildReport { rows: 12_000 },
        JobKind::SendEmail {
            to: "bounce@example.com".into(),
        },
        JobKind::ResizeImage {
            width: 0,
            height: 480,
        },
        JobKind::BuildReport { rows: 300 },
    ];
    kinds
        .into_iter()
        .enumerate()
        .map(|(i, kind)| Job {
            id: 101 + i as u32,
            kind,
        })
        .collect()
}

/// One attempt at a job. `attempt` starts at 1.
pub fn run_once(job: &Job, attempt: u32) -> Result<String, JobError> {
    match &job.kind {
        JobKind::SendEmail { to } if to.contains("bounce") => {
            Err(JobError::Permanent(format!("mailbox {to} does not exist")))
        }
        JobKind::SendEmail { to } => Ok(format!("sent to {to}")),
        JobKind::ResizeImage { width, height } if *width == 0 || *height == 0 => Err(
            JobError::Invalid(format!("cannot resize to {width}x{height}")),
        ),
        JobKind::ResizeImage { width, height } => Ok(format!("thumbnail {width}x{height}")),
        JobKind::BuildReport { rows } if *rows > 5_000 && attempt == 1 => {
            Err(JobError::Transient("database busy".into()))
        }
        JobKind::BuildReport { rows } => Ok(format!("report of {rows} rows")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn big_reports_fail_once_then_succeed() {
        let job = Job {
            id: 1,
            kind: JobKind::BuildReport { rows: 9_000 },
        };
        assert!(run_once(&job, 1).unwrap_err().is_retryable());
        assert_eq!(run_once(&job, 2), Ok("report of 9000 rows".into()));
    }

    #[test]
    fn invalid_and_permanent_errors_are_not_retryable() {
        let jobs = sample_jobs();
        assert!(!run_once(&jobs[3], 1).unwrap_err().is_retryable());
        assert!(!run_once(&jobs[4], 1).unwrap_err().is_retryable());
    }

    #[test]
    fn kinds_have_short_names() {
        let names: Vec<String> = sample_jobs().iter().map(|j| j.kind.to_string()).collect();
        assert_eq!(
            names,
            ["email", "resize", "report", "email", "resize", "report"]
        );
    }
}
