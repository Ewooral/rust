//! The notes themselves: the JSON shapes, the validation rules and an
//! in-memory store. Nothing in this file knows about HTTP, so it is tested
//! with plain `#[test]`s.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::RwLock;

/// Limits the validation enforces.
pub const MAX_TITLE: usize = 80;
pub const MAX_BODY: usize = 2_000;
pub const MAX_TAGS: usize = 5;
pub const MAX_TAG: usize = 20;

/// A stored note, as the API returns it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub done: bool,
}

/// The JSON a client sends to create a note. `body` and `tags` may be left
/// out (`#[serde(default)]`); any other unknown field is an error.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewNote {
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// The JSON for a partial update (PATCH): every field is optional, and only
/// the fields that are present change.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateNote {
    pub title: Option<String>,
    pub body: Option<String>,
    pub tags: Option<Vec<String>>,
    pub done: Option<bool>,
}

/// Query string for listing: `/api/notes?tag=rust&done=false&limit=10&offset=0`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListParams {
    pub tag: Option<String>,
    pub done: Option<bool>,
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: usize,
}

/// Default and largest page size for `GET /api/notes`.
pub const DEFAULT_LIMIT: usize = 20;
pub const MAX_LIMIT: usize = 100;

fn check_title(title: &str, problems: &mut Vec<String>) {
    let title = title.trim();
    if title.is_empty() {
        problems.push("title must not be empty".to_string());
    } else if title.chars().count() > MAX_TITLE {
        problems.push(format!("title must be at most {MAX_TITLE} characters"));
    }
}

fn check_body(body: &str, problems: &mut Vec<String>) {
    if body.chars().count() > MAX_BODY {
        problems.push(format!("body must be at most {MAX_BODY} characters"));
    }
}

fn check_tags(tags: &[String], problems: &mut Vec<String>) {
    if tags.len() > MAX_TAGS {
        problems.push(format!("at most {MAX_TAGS} tags are allowed"));
    }
    for tag in tags {
        let ok_chars = tag
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if tag.is_empty() || tag.len() > MAX_TAG || !ok_chars {
            problems.push(format!(
                "tag {tag:?} must be 1 to {MAX_TAG} characters of a-z, 0-9 or -"
            ));
        }
    }
}

impl NewNote {
    /// Checks every rule and reports ALL the problems at once, so a client
    /// can fix its form in one go. Returns the cleaned-up note on success.
    pub fn validate(self) -> Result<NewNote, Vec<String>> {
        let mut problems = Vec::new();
        check_title(&self.title, &mut problems);
        check_body(&self.body, &mut problems);
        check_tags(&self.tags, &mut problems);
        if problems.is_empty() {
            Ok(NewNote {
                title: self.title.trim().to_string(),
                ..self
            })
        } else {
            Err(problems)
        }
    }
}

impl UpdateNote {
    /// Same rules as [`NewNote::validate`], for the fields that are present.
    pub fn validate(self) -> Result<UpdateNote, Vec<String>> {
        let mut problems = Vec::new();
        if let Some(title) = &self.title {
            check_title(title, &mut problems);
        }
        if let Some(body) = &self.body {
            check_body(body, &mut problems);
        }
        if let Some(tags) = &self.tags {
            check_tags(tags, &mut problems);
        }
        if problems.is_empty() {
            Ok(UpdateNote {
                title: self.title.map(|t| t.trim().to_string()),
                ..self
            })
        } else {
            Err(problems)
        }
    }
}

/// An in-memory store, shared by every request through `Arc<NoteStore>`.
///
/// It uses a **std** `RwLock`: the lock is only ever held for a few
/// lines of plain code, never across an `.await`, so the async-aware
/// `tokio::sync::RwLock` isn't needed (and would be slower).
#[derive(Debug, Default)]
pub struct NoteStore {
    inner: RwLock<Inner>,
}

#[derive(Debug, Default)]
struct Inner {
    next_id: u64,
    notes: BTreeMap<u64, Note>, // BTreeMap: listed in id order
}

impl NoteStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&self, new: NewNote) -> Note {
        let mut inner = self.inner.write().expect("lock poisoned");
        inner.next_id += 1;
        let note = Note {
            id: inner.next_id,
            title: new.title,
            body: new.body,
            tags: new.tags,
            done: false,
        };
        inner.notes.insert(note.id, note.clone());
        note
    }

    pub fn get(&self, id: u64) -> Option<Note> {
        self.inner
            .read()
            .expect("lock poisoned")
            .notes
            .get(&id)
            .cloned()
    }

    pub fn list(&self, params: &ListParams) -> Vec<Note> {
        let limit = params.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
        let inner = self.inner.read().expect("lock poisoned");
        inner
            .notes
            .values()
            .filter(|n| params.tag.as_ref().is_none_or(|t| n.tags.contains(t)))
            .filter(|n| params.done.is_none_or(|d| n.done == d))
            .skip(params.offset)
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn update(&self, id: u64, change: UpdateNote) -> Option<Note> {
        let mut inner = self.inner.write().expect("lock poisoned");
        let note = inner.notes.get_mut(&id)?;
        if let Some(title) = change.title {
            note.title = title;
        }
        if let Some(body) = change.body {
            note.body = body;
        }
        if let Some(tags) = change.tags {
            note.tags = tags;
        }
        if let Some(done) = change.done {
            note.done = done;
        }
        Some(note.clone())
    }

    pub fn delete(&self, id: u64) -> bool {
        self.inner
            .write()
            .expect("lock poisoned")
            .notes
            .remove(&id)
            .is_some()
    }

    pub fn len(&self) -> usize {
        self.inner.read().expect("lock poisoned").notes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_note(title: &str, tags: &[&str]) -> NewNote {
        NewNote {
            title: title.to_string(),
            body: String::new(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
        }
    }

    #[test]
    fn validation_trims_the_title() {
        let ok = new_note("  Learn axum  ", &["rust"]).validate().unwrap();
        assert_eq!(ok.title, "Learn axum");
    }

    #[test]
    fn validation_reports_every_problem() {
        let problems = new_note("   ", &["Bad Tag", "ok"]).validate().unwrap_err();
        assert_eq!(
            problems,
            vec![
                "title must not be empty".to_string(),
                "tag \"Bad Tag\" must be 1 to 20 characters of a-z, 0-9 or -".to_string(),
            ]
        );
        let long = "x".repeat(MAX_TITLE + 1);
        let too_many = ["a", "b", "c", "d", "e", "f"];
        let problems = new_note(&long, &too_many).validate().unwrap_err();
        assert_eq!(problems.len(), 2, "{problems:?}");
    }

    #[test]
    fn update_validates_only_present_fields() {
        assert!(UpdateNote::default().validate().is_ok());
        let bad = UpdateNote {
            title: Some(String::new()),
            ..Default::default()
        };
        assert_eq!(bad.validate().unwrap_err(), ["title must not be empty"]);
    }

    #[test]
    fn store_crud_round_trip() {
        let store = NoteStore::new();
        let a = store.create(new_note("a", &["rust"]));
        let b = store.create(new_note("b", &[]));
        assert_eq!((a.id, b.id), (1, 2));
        assert_eq!(store.get(1), Some(a));
        let done = UpdateNote {
            done: Some(true),
            ..Default::default()
        };
        assert!(store.update(2, done).unwrap().done);
        assert!(store.delete(1));
        assert!(!store.delete(1));
        assert_eq!(store.len(), 1);
        assert_eq!(store.update(99, UpdateNote::default()), None);
    }

    #[test]
    fn list_filters_and_pages() {
        let store = NoteStore::new();
        for i in 0..5 {
            let tags: &[&str] = if i % 2 == 0 { &["even"] } else { &[] };
            store.create(new_note(&format!("note {i}"), tags));
        }
        let even = ListParams {
            tag: Some("even".to_string()),
            ..Default::default()
        };
        let ids: Vec<u64> = store.list(&even).iter().map(|n| n.id).collect();
        assert_eq!(ids, [1, 3, 5]);
        let page = ListParams {
            limit: Some(2),
            offset: 1,
            ..Default::default()
        };
        let ids: Vec<u64> = store.list(&page).iter().map(|n| n.id).collect();
        assert_eq!(ids, [2, 3]);
    }
}
