//! The pieces of the pretend file system: names, files and directories.

use std::cell::{Cell, OnceCell, RefCell};
use std::collections::BTreeMap;
use std::fmt;
use std::ops::Deref;
use std::rc::{Rc, Weak};

/// A checked file or directory name: not empty, not `.` or `..`, no `/`.
///
/// `Name` implements `Deref<Target = str>`, so a `&Name` works anywhere a
/// `&str` is expected (deref coercion), and every `str` method works on it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name(String);

impl Name {
    /// Checks `name` and wraps it.
    pub fn new(name: &str) -> Result<Name, String> {
        if name.is_empty() || name == "." || name == ".." || name.contains('/') {
            Err(format!("`{name}` is not a valid name"))
        } else {
            Ok(Name(name.to_string()))
        }
    }
}

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

/// Lets a `BTreeMap<Name, _>` be searched with a plain `&str`.
///
/// Written out in full instead of `use std::borrow::Borrow`: with that
/// trait in scope, `rc_refcell.borrow()` becomes ambiguous (error E0282).
impl std::borrow::Borrow<str> for Name {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}

/// Messages written by `Drop`, shared by every directory of one file system:
/// `Rc` so they can all hold it, `RefCell` so they can all write to it.
pub type DropLog = Rc<RefCell<Vec<String>>>;

/// A file. It is always handled through an `Rc<File>`, so several
/// directories can share it (a hard link). That is why it changes through
/// `&self`: its contents are in a `RefCell` and its read counter in a `Cell`.
#[derive(Debug, Default)]
pub struct File {
    contents: RefCell<String>,
    reads: Cell<u32>,
}

impl File {
    /// An empty file.
    pub fn new() -> File {
        File::default()
    }

    /// Adds a line at the end. Takes `&self`: interior mutability.
    pub fn append(&self, line: &str) {
        let mut contents = self.contents.borrow_mut();
        contents.push_str(line);
        contents.push('\n');
    }

    /// The contents. Each read is counted.
    pub fn read(&self) -> String {
        self.reads.set(self.reads.get() + 1);
        self.contents.borrow().clone()
    }

    /// How many times [`File::read`] was called.
    pub fn reads(&self) -> u32 {
        self.reads.get()
    }

    /// The size in bytes.
    pub fn len(&self) -> usize {
        self.contents.borrow().len()
    }

    /// Whether the file is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// An entry in a directory: a shared directory or a shared file.
/// Cloning a `Node` clones the `Rc` inside: one more owner, no copied data.
#[derive(Debug, Clone)]
pub enum Node {
    /// A directory. `RefCell` because its children change while it is shared.
    Dir(Rc<RefCell<Dir>>),
    /// A file, possibly listed in several directories.
    File(Rc<File>),
}

/// A directory.
#[derive(Debug)]
pub struct Dir {
    name: String,
    /// `Weak`: a child must not keep its parent alive. With an `Rc` here,
    /// parent and child would own each other and never be freed.
    parent: Weak<RefCell<Dir>>,
    children: BTreeMap<Name, Node>,
    /// Worked out the first time it is asked for, then kept.
    path: OnceCell<String>,
    log: DropLog,
}

impl Dir {
    /// A new root directory (no parent), writing to `log` when freed.
    pub fn new_root(log: DropLog) -> Rc<RefCell<Dir>> {
        Rc::new(RefCell::new(Dir {
            name: String::new(),
            parent: Weak::new(),
            children: BTreeMap::new(),
            path: OnceCell::new(),
            log,
        }))
    }

    /// Makes a new, empty directory called `name` inside `parent`.
    pub fn create_in(parent: &Rc<RefCell<Dir>>, name: Name) -> Result<Rc<RefCell<Dir>>, String> {
        let log = Rc::clone(&parent.borrow().log);
        let child = Rc::new(RefCell::new(Dir {
            name: name.to_string(),
            parent: Rc::downgrade(parent),
            children: BTreeMap::new(),
            path: OnceCell::new(),
            log,
        }));
        parent
            .borrow_mut()
            .insert(name, Node::Dir(Rc::clone(&child)))?;
        Ok(child)
    }

    /// Adds an entry. Fails if the name is taken.
    pub fn insert(&mut self, name: Name, node: Node) -> Result<(), String> {
        if self.children.contains_key(&*name) {
            return Err(format!("`{name}` already exists"));
        }
        self.children.insert(name, node);
        Ok(())
    }

    /// Takes an entry out and hands it back. If nobody else owns it, it is
    /// freed when the caller drops it.
    pub fn remove(&mut self, name: &str) -> Option<Node> {
        self.children.remove(name)
    }

    /// The entry called `name`, as a new owner (a cheap `Rc` clone).
    pub fn get(&self, name: &str) -> Option<Node> {
        self.children.get(name).cloned()
    }

    /// The entries, sorted by name.
    pub fn entries(&self) -> impl Iterator<Item = (&Name, &Node)> {
        self.children.iter()
    }

    /// The parent directory, if it still exists. `None` for the root.
    pub fn parent(&self) -> Option<Rc<RefCell<Dir>>> {
        self.parent.upgrade()
    }

    /// The directory's own name (empty for the root).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The full path, such as `/projects/rust`. Worked out by walking up
    /// the `Weak` parent links once, then cached in a `OnceCell`.
    pub fn path(&self) -> String {
        self.path
            .get_or_init(|| match self.parent() {
                None => "/".to_string(),
                Some(parent) => {
                    let above = parent.borrow().path();
                    if above == "/" {
                        format!("/{}", self.name)
                    } else {
                        format!("{above}/{}", self.name)
                    }
                }
            })
            .clone()
    }
}

/// Freeing a directory writes a line to the shared log. `drop` runs first,
/// then Rust drops the fields, so subdirectories are freed after this one.
impl Drop for Dir {
    fn drop(&mut self) {
        let name = if self.name.is_empty() {
            "/"
        } else {
            &self.name
        };
        self.log
            .borrow_mut()
            .push(format!("freed directory {name}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_bytes(text: &str) -> usize {
        text.len()
    }

    #[test]
    fn a_name_derefs_to_str() {
        let name = Name::new("notes.txt").unwrap();
        assert!(name.ends_with(".txt")); // a str method, through Deref
        assert_eq!(count_bytes(&name), 9); // &Name -> &str: deref coercion
        assert!(Name::new("a/b").is_err());
        assert!(Name::new("..").is_err());
    }

    #[test]
    fn a_files_contents_change_through_a_shared_reference() {
        let file = Rc::new(File::new());
        let link = Rc::clone(&file);
        link.append("hello");
        assert_eq!(file.read(), "hello\n");
        assert_eq!(file.reads(), 1);
        assert_eq!(Rc::strong_count(&file), 2);
    }

    #[test]
    fn the_path_is_worked_out_once_through_weak_parents() {
        let log = DropLog::default();
        let root = Dir::new_root(Rc::clone(&log));
        let a = Dir::create_in(&root, Name::new("a").unwrap()).unwrap();
        let b = Dir::create_in(&a, Name::new("b").unwrap()).unwrap();
        assert!(b.borrow().path.get().is_none());
        assert_eq!(b.borrow().path(), "/a/b");
        assert!(b.borrow().path.get().is_some(), "now cached");
        // Only `root` owns the root. `a` points back at it with a Weak.
        assert_eq!(Rc::strong_count(&root), 1);
        assert_eq!(Rc::weak_count(&root), 1);
        assert_eq!(
            Rc::strong_count(&a),
            2,
            "the variable `a` + root's children"
        );
    }

    #[test]
    fn dropping_the_root_frees_every_directory() {
        let log = DropLog::default();
        let root = Dir::new_root(Rc::clone(&log));
        let a = Dir::create_in(&root, Name::new("a").unwrap()).unwrap();
        Dir::create_in(&a, Name::new("b").unwrap()).unwrap();
        let watch = Rc::downgrade(&a);
        drop(a);
        assert!(log.borrow().is_empty(), "the root still owns `a`");
        drop(root);
        assert_eq!(
            *log.borrow(),
            [
                "freed directory /",
                "freed directory a",
                "freed directory b"
            ]
        );
        assert!(watch.upgrade().is_none(), "no leak: `a` is really gone");
    }
}
