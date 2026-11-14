//! The shell's command history: the Book's cons list, put to work.

/// A list of commands, newest first. Each entry owns the rest of the list
/// through a `Box`: without the box, `History` would contain a `History`
/// and have no fixed size (error E0072).
#[derive(Debug, Default)]
pub enum History {
    /// A command, then everything older.
    Entry(String, Box<History>),
    /// The end of the list.
    #[default]
    Empty,
}

impl History {
    /// Puts `command` at the front. The old list moves into a new box.
    pub fn push(&mut self, command: &str) {
        let older = std::mem::take(self); // leaves `Empty` behind for a moment
        *self = History::Entry(command.to_string(), Box::new(older));
    }

    /// The commands, newest first.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        let mut current = self;
        std::iter::from_fn(move || match current {
            History::Entry(command, older) => {
                current = older; // &Box<History> becomes &History: deref coercion
                Some(command.as_str())
            }
            History::Empty => None,
        })
    }

    /// How many commands are stored.
    pub fn len(&self) -> usize {
        self.iter().count()
    }

    /// Whether no command has been stored.
    pub fn is_empty(&self) -> bool {
        matches!(self, History::Empty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newest_first() {
        let mut history = History::default();
        assert!(history.is_empty());
        for command in ["mkdir a", "cd a", "pwd"] {
            history.push(command);
        }
        assert_eq!(
            history.iter().collect::<Vec<_>>(),
            ["pwd", "cd a", "mkdir a"]
        );
        assert_eq!(history.len(), 3);
    }
}
