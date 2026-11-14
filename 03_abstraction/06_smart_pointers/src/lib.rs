//! # tinyfs: a pretend file system built from smart pointers
//!
//! The mini project for Book chapter 15. Directories and files live only in
//! memory, and every smart pointer in the chapter has a real job:
//!
//! | Pointer | Job here |
//! |---|---|
//! | `Box<T>` | the command [`History`] is a cons list |
//! | `Deref` | a [`Name`] can be used wherever a `&str` is expected |
//! | `Drop` | a freed [`Dir`] writes "freed directory ..." to a log |
//! | `Rc<T>` | a [`File`] can be listed in two directories (a hard link) |
//! | `RefCell<T>` | directories and files change while they are shared |
//! | `Weak<T>` | a directory points to its parent without owning it, so `cd ..` works and nothing leaks |
//! | `Cell<T>`, `OnceCell<T>` | a file's read counter; a directory's cached path |
//! | `LazyLock<T>` | the global table of commands |
//!
//! ```
//! use smart_pointers::Shell;
//!
//! let mut shell = Shell::new();
//! shell.run("mkdir projects").unwrap();
//! shell.run("cd projects").unwrap();
//! assert_eq!(shell.run("pwd").unwrap(), "/projects");
//! shell.run("cd ..").unwrap(); // follows a Weak link back up
//! assert_eq!(shell.pwd(), "/");
//! ```

pub mod fs;
pub mod history;
pub mod shell;

pub use fs::{Dir, DropLog, File, Name, Node};
pub use history::History;
pub use shell::{COMMANDS, Shell};
