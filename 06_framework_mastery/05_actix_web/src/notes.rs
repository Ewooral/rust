//! The notes themselves: JSON shapes, validation and an in-memory store.
//!
//! Nothing here knows about HTTP or Actix, which is the point: the same file
//! could sit behind Axum (lesson 06/01 has an equivalent one) or a CLI, and it
//! is tested with plain `#[test]`s.

use std::collections::BTreeMap;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

/// Longest title, in characters.
pub const MAX_TITLE: usize = 80;
/// Longest body, in characters.
pub const MAX_BODY: usize = 2_000;
/// Most tags on one note.
pub const MAX_TAGS: usize = 5;
/// Longest tag, in bytes.
pub const MAX_TAG: usize = 20;
/// Page size when the client doesn't ask for one.
pub const DEFAULT_LIMIT: usize = 20;
/// Largest page a client may ask for.
pub const MAX_LIMIT: usize = 100;

/// A stored note, as the API returns it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    /// Given by the store, starting at 1.
    pub id: u64,
    /// A short title.
    pub title: String,
    /// Free text (may be empty).
    pub body: String,
    /// Lower-case labels such as `rust`.
    pub tags: Vec<String>,
    /// Ticked off?
    pub done: bool,
}

/// The JSON a client sends to create a note. `body` and `tags` may be left out.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewNote {
    /// Required.
    pub title: String,
    /// Optional, defaults to "".
    #[serde(default)]
    pub body: String,
    /// Optional, defaults to none.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A partial update (PATCH): only the fields that are present change.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateNote {
    /// New title.
    pub title: Option<String>,
    /// New body.
    pub body: Option<String>,
    /// New tags (replace the old ones).
    pub tags: Option<Vec<String>>,
    /// Tick or untick.
    pub done: Option<bool>,
}

/// Query string for listing: `/api/notes?tag=rust&done=false&limit=10&offset=0`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListParams {
    /// Only notes with this tag.
    pub tag: Option<String>,
    /// Only done (or not done) notes.
    pub done: Option<bool>,
    /// Page size (at most [`MAX_LIMIT`]).
    pub limit: Option<usize>,
    /// Notes to skip.
    #[serde(default)]
    pub offset: usize,
}

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
    /// Checks every rule and reports ALL the problems at once.
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
    /// The same rules as [`NewNote::validate`], for the fields that are present.
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

/// An in-memory store, shared by every worker thread through `web::Data`.
///
/// A std `RwLock` is fine: it is never held across an `.await`.
#[derive(Debug, Default)]
pub struct NoteStore {
    inner: RwLock<Inner>,
}

#[derive(Debug, Default)]
struct Inner {
    next_id: u64,
    notes: BTreeMap<u64, Note>,
}

impl NoteStore {
    /// An empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores a (validated) note and returns it with its new id.
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

    /// The note with this id.
    pub fn get(&self, id: u64) -> Option<Note> {
        let inner = self.inner.read().expect("lock poisoned");
        inner.notes.get(&id).cloned()
    }

    /// One page of notes, filtered, in id order.
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

    /// Applies a (validated) partial update; `None` if there's no such note.
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

    /// Removes a note; `false` if there was no such note.
    pub fn delete(&self, id: u64) -> bool {
        let mut inner = self.inner.write().expect("lock poisoned");
        inner.notes.remove(&id).is_some()
    }

    /// How many notes are stored.
    pub fn len(&self) -> usize {
        self.inner.read().expect("lock poisoned").notes.len()
    }

    /// True when there are no notes.
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
    fn validation_trims_and_reports_every_problem() {
        assert_eq!(
            new_note("  Learn actix  ", &[]).validate().unwrap().title,
            "Learn actix"
        );
        let problems = new_note(" ", &["Bad Tag"]).validate().unwrap_err();
        assert_eq!(
            problems,
            [
                "title must not be empty",
                "tag \"Bad Tag\" must be 1 to 20 characters of a-z, 0-9 or -"
            ]
        );
    }

    #[test]
    fn partial_updates_only_check_what_is_present() {
        let ok = UpdateNote {
            done: Some(true),
            ..UpdateNote::default()
        };
        assert!(ok.validate().is_ok());
        let bad = UpdateNote {
            title: Some(String::new()),
            ..UpdateNote::default()
        };
        assert_eq!(bad.validate().unwrap_err(), ["title must not be empty"]);
    }

    #[test]
    fn the_store_creates_lists_updates_and_deletes() {
        let store = NoteStore::new();
        let a = store.create(new_note("a", &["rust"]));
        let b = store.create(new_note("b", &[]));
        assert_eq!((a.id, b.id), (1, 2));
        let rust = ListParams {
            tag: Some("rust".into()),
            ..ListParams::default()
        };
        assert_eq!(store.list(&rust), std::slice::from_ref(&a));
        let done = UpdateNote {
            done: Some(true),
            ..UpdateNote::default()
        };
        assert!(store.update(2, done).unwrap().done);
        assert!(store.delete(1));
        assert!(!store.delete(1));
        assert_eq!(store.len(), 1);
        assert_eq!(store.get(1), None);
    }

    #[test]
    fn pages_are_capped() {
        let store = NoteStore::new();
        for i in 0..150 {
            store.create(new_note(&format!("n{i}"), &[]));
        }
        let all = ListParams {
            limit: Some(1000),
            ..ListParams::default()
        };
        assert_eq!(store.list(&all).len(), MAX_LIMIT);
        assert_eq!(store.list(&ListParams::default()).len(), DEFAULT_LIMIT);
    }
}
