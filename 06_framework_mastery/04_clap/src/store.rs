//! The tasks and the JSON file that keeps them. Nothing here knows about
//! clap.

use crate::cli::{Priority, Status};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub priority: Priority,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub due: Option<String>,
    #[serde(default)]
    pub done: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Store {
    next_id: u32,
    tasks: Vec<Task>,
}

/// Everything that can go wrong AFTER the arguments were parsed. Each one
/// makes the program exit with code 1 (clap uses 2 for bad arguments).
#[derive(Debug)]
pub enum AppError {
    NotFound(u32),
    Io { path: PathBuf, source: io::Error },
    Corrupt { path: PathBuf, message: String },
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(id) => write!(f, "no task with id {id}"),
            AppError::Io { path, source } => write!(f, "{}: {source}", path.display()),
            AppError::Corrupt { path, message } => {
                write!(f, "{} is not a valid task file: {message}", path.display())
            }
        }
    }
}

impl std::error::Error for AppError {}

/// The filters `list` applies.
#[derive(Debug, Clone, Default)]
pub struct Query {
    pub status: Option<Status>,
    pub tag: Option<String>,
    pub min_priority: Option<Priority>,
    pub limit: Option<usize>,
}

impl Store {
    /// Reads the file; a file that doesn't exist yet is an empty store.
    pub fn load(path: &Path) -> Result<Store, AppError> {
        match fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text).map_err(|e| AppError::Corrupt {
                path: path.to_path_buf(),
                message: e.to_string(),
            }),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Store::default()),
            Err(source) => Err(AppError::Io {
                path: path.to_path_buf(),
                source,
            }),
        }
    }

    /// Writes to a temporary file first, then renames it over the real one,
    /// so a crash half-way never leaves a half-written task file.
    pub fn save(&self, path: &Path) -> Result<(), AppError> {
        let io_error = |source| AppError::Io {
            path: path.to_path_buf(),
            source,
        };
        let json = serde_json::to_string_pretty(self).expect("tasks always serialize");
        let temp = path.with_extension("json.tmp");
        fs::write(&temp, json + "\n").map_err(io_error)?;
        fs::rename(&temp, path).map_err(io_error)
    }

    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    pub fn add(
        &mut self,
        title: String,
        priority: Priority,
        tags: Vec<String>,
        due: Option<String>,
    ) -> &Task {
        self.next_id += 1;
        self.tasks.push(Task {
            id: self.next_id,
            title,
            priority,
            tags,
            due,
            done: false,
        });
        self.tasks.last().expect("just pushed")
    }

    fn find_mut(&mut self, id: u32) -> Result<&mut Task, AppError> {
        self.tasks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or(AppError::NotFound(id))
    }

    /// All or nothing: if any id is unknown, nothing changes.
    pub fn mark_done(&mut self, ids: &[u32]) -> Result<(), AppError> {
        self.check_ids(ids)?;
        for id in ids {
            self.find_mut(*id)?.done = true;
        }
        Ok(())
    }

    /// All or nothing, like `mark_done`. Returns how many were removed.
    pub fn remove(&mut self, ids: &[u32]) -> Result<usize, AppError> {
        self.check_ids(ids)?;
        let before = self.tasks.len();
        self.tasks.retain(|t| !ids.contains(&t.id));
        Ok(before - self.tasks.len())
    }

    pub fn remove_done(&mut self) -> usize {
        let before = self.tasks.len();
        self.tasks.retain(|t| !t.done);
        before - self.tasks.len()
    }

    pub fn add_tags(&mut self, id: u32, tags: &[String]) -> Result<&Task, AppError> {
        let task = self.find_mut(id)?;
        for tag in tags {
            if !task.tags.contains(tag) {
                task.tags.push(tag.clone());
            }
        }
        Ok(task)
    }

    pub fn remove_tags(&mut self, id: u32, tags: &[String]) -> Result<&Task, AppError> {
        let task = self.find_mut(id)?;
        task.tags.retain(|t| !tags.contains(t));
        Ok(task)
    }

    pub fn query(&self, query: &Query) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| match query.status {
                Some(Status::Open) => !t.done,
                Some(Status::Done) => t.done,
                Some(Status::All) | None => true,
            })
            .filter(|t| query.tag.as_ref().is_none_or(|tag| t.tags.contains(tag)))
            .filter(|t| query.min_priority.is_none_or(|p| t.priority >= p))
            .take(query.limit.unwrap_or(usize::MAX))
            .collect()
    }

    fn check_ids(&self, ids: &[u32]) -> Result<(), AppError> {
        match ids
            .iter()
            .find(|id| !self.tasks.iter().any(|t| t.id == **id))
        {
            Some(missing) => Err(AppError::NotFound(*missing)),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Store {
        let mut store = Store::default();
        store.add("a".into(), Priority::Low, vec!["home".into()], None);
        store.add("b".into(), Priority::High, vec![], None);
        store.add("c".into(), Priority::Normal, vec!["home".into()], None);
        store
    }

    fn ids(tasks: Vec<&Task>) -> Vec<u32> {
        tasks.iter().map(|t| t.id).collect()
    }

    #[test]
    fn ids_count_up_from_one() {
        assert_eq!(ids(sample().tasks().iter().collect()), [1, 2, 3]);
    }

    #[test]
    fn query_filters() {
        let mut store = sample();
        store.mark_done(&[3]).unwrap();
        let all = Query::default();
        assert_eq!(ids(store.query(&all)), [1, 2, 3]);
        let open_home = Query {
            status: Some(Status::Open),
            tag: Some("home".into()),
            ..Query::default()
        };
        assert_eq!(ids(store.query(&open_home)), [1]);
        let urgent = Query {
            min_priority: Some(Priority::Normal),
            limit: Some(1),
            ..Query::default()
        };
        assert_eq!(ids(store.query(&urgent)), [2]);
    }

    #[test]
    fn unknown_ids_change_nothing() {
        let mut store = sample();
        assert!(matches!(
            store.mark_done(&[1, 9]),
            Err(AppError::NotFound(9))
        ));
        assert!(!store.tasks()[0].done, "all or nothing");
        assert!(matches!(store.remove(&[2, 8]), Err(AppError::NotFound(8))));
        assert_eq!(store.remove(&[1, 2]).unwrap(), 2);
        assert_eq!(store.tasks().len(), 1);
    }

    #[test]
    fn tags_are_added_once_and_removed() {
        let mut store = sample();
        let task = store
            .add_tags(2, &["work".into(), "work".into(), "q4".into()])
            .unwrap();
        assert_eq!(task.tags, ["work", "q4"]);
        let task = store.remove_tags(2, &["work".into()]).unwrap();
        assert_eq!(task.tags, ["q4"]);
    }

    #[test]
    fn save_then_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tasks.json");
        assert!(
            Store::load(&path).unwrap().tasks().is_empty(),
            "missing file = empty"
        );
        sample().save(&path).unwrap();
        let loaded = Store::load(&path).unwrap();
        assert_eq!(loaded.tasks(), sample().tasks());

        fs::write(&path, "{ not json").unwrap();
        assert!(matches!(Store::load(&path), Err(AppError::Corrupt { .. })));
    }
}
