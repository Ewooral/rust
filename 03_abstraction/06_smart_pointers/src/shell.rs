//! The shell: runs one command line and returns the text to print.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::LazyLock;

use crate::fs::{Dir, DropLog, File, Name, Node};
use crate::history::History;

/// Every command and its help line. A `LazyLock` builds the map the first
/// time anyone uses it; every later use gets the same map.
pub static COMMANDS: LazyLock<BTreeMap<&'static str, &'static str>> = LazyLock::new(|| {
    BTreeMap::from([
        ("cat", "cat PATH           print a file"),
        (
            "cd",
            "cd PATH            change directory (`..` goes up, `/` is the root)",
        ),
        ("help", "help               this list"),
        (
            "history",
            "history            the commands so far, newest first",
        ),
        (
            "ln",
            "ln FILE DIR        list FILE in DIR too: one file, two names",
        ),
        ("ls", "ls [PATH]          list a directory"),
        ("mkdir", "mkdir PATH         make a directory"),
        ("pwd", "pwd                print the current directory"),
        (
            "rm",
            "rm NAME            remove an entry from the current directory",
        ),
        (
            "stat",
            "stat PATH          owners and counters of a file or directory",
        ),
        ("touch", "touch PATH         make an empty file"),
        (
            "tree",
            "tree               everything below the current directory",
        ),
        (
            "write",
            "write PATH TEXT    add a line to a file (made if missing)",
        ),
    ])
});

/// A pretend file system and the state of one session in it.
pub struct Shell {
    root: Rc<RefCell<Dir>>,
    /// A second owner of one directory: the one we are "in".
    cwd: Rc<RefCell<Dir>>,
    history: History,
    log: DropLog,
}

impl Default for Shell {
    fn default() -> Self {
        Shell::new()
    }
}

impl Shell {
    /// An empty file system, standing in `/`.
    pub fn new() -> Shell {
        let log = DropLog::default();
        let root = Dir::new_root(Rc::clone(&log));
        Shell {
            cwd: Rc::clone(&root),
            root,
            history: History::default(),
            log,
        }
    }

    /// The log that `Drop` writes to. It is shared (`Rc`), so you can keep
    /// it and read it after the shell itself is gone.
    pub fn drop_log(&self) -> DropLog {
        Rc::clone(&self.log)
    }

    /// The current directory's path.
    pub fn pwd(&self) -> String {
        self.cwd.borrow().path()
    }

    /// Runs one command line. `Ok` holds the output (maybe empty), `Err`
    /// the error message.
    pub fn run(&mut self, line: &str) -> Result<String, String> {
        let words: Vec<&str> = line.split_whitespace().collect();
        let Some((&command, args)) = words.split_first() else {
            return Ok(String::new());
        };
        self.history.push(line.trim());
        match (command, args) {
            ("help", []) => Ok(COMMANDS.values().copied().collect::<Vec<_>>().join("\n")),
            ("pwd", []) => Ok(self.pwd()),
            ("mkdir", [path]) => self.mkdir(path),
            ("cd", [path]) => {
                self.cwd = self.resolve_dir(path)?; // the old cwd loses an owner
                Ok(String::new())
            }
            ("ls", []) => self.ls("."),
            ("ls", [path]) => self.ls(path),
            ("touch", [path]) => self.file_at(path, true).map(|_| String::new()),
            ("write", [path, text @ ..]) if !text.is_empty() => {
                self.file_at(path, true)?.append(&text.join(" "));
                Ok(String::new())
            }
            ("cat", [path]) => {
                let text = self.file_at(path, false)?.read();
                Ok(text.trim_end().to_string())
            }
            ("ln", [file, dir]) => self.ln(file, dir),
            ("rm", [name]) => self.rm(name),
            ("stat", [path]) => self.stat(path),
            ("tree", []) => {
                let mut out = self.pwd();
                draw_tree(&self.cwd.borrow(), "", &mut out);
                Ok(out)
            }
            ("history", []) => {
                let count = self.history.len();
                let lines: Vec<String> = (1..=count)
                    .rev()
                    .zip(self.history.iter())
                    .map(|(n, command)| format!("{n:>3}  {command}"))
                    .collect();
                Ok(lines.join("\n"))
            }
            (known, _) if COMMANDS.contains_key(known) => {
                Err(format!("usage: {}", COMMANDS[known]))
            }
            (unknown, _) => Err(format!("{unknown}: command not found (try `help`)")),
        }
    }

    /// Follows a path from the root (`/a/b`) or from the current directory
    /// (`a/b`, `..`). Returns a new owner of whatever is there.
    fn resolve(&self, path: &str) -> Result<Node, String> {
        let start = if path.starts_with('/') {
            &self.root
        } else {
            &self.cwd
        };
        let mut node = Node::Dir(Rc::clone(start));
        for part in path.split('/').filter(|p| !p.is_empty() && *p != ".") {
            let Node::Dir(dir) = &node else {
                return Err(format!("{path}: not a directory"));
            };
            let next = if part == ".." {
                // Upgrade the Weak parent link. The root has no parent: stay.
                let parent = dir.borrow().parent();
                Node::Dir(parent.unwrap_or_else(|| Rc::clone(dir)))
            } else {
                let child = dir.borrow().get(part);
                child.ok_or_else(|| format!("{path}: no such file or directory"))?
            };
            node = next;
        }
        Ok(node)
    }

    fn resolve_dir(&self, path: &str) -> Result<Rc<RefCell<Dir>>, String> {
        match self.resolve(path)? {
            Node::Dir(dir) => Ok(dir),
            Node::File(_) => Err(format!("{path}: not a directory")),
        }
    }

    fn ls(&self, path: &str) -> Result<String, String> {
        let Node::Dir(dir) = self.resolve(path)? else {
            return Ok(path.to_string()); // `ls file` just names the file
        };
        let lines: Vec<String> = dir
            .borrow()
            .entries()
            .map(|(name, node)| match node {
                Node::Dir(_) => format!("{name}/"),
                Node::File(file) if Rc::strong_count(file) > 1 => format!(
                    "{name}  ({} bytes, {} links)",
                    file.len(),
                    Rc::strong_count(file)
                ),
                Node::File(file) => format!("{name}  ({} bytes)", file.len()),
            })
            .collect();
        Ok(lines.join("\n"))
    }

    fn mkdir(&mut self, path: &str) -> Result<String, String> {
        let (parent, name) = split_parent(path);
        let dir = self.resolve_dir(parent)?;
        Dir::create_in(&dir, Name::new(name)?)?;
        Ok(String::new())
    }

    /// The file at `path`, made empty first if `create` is true and it is
    /// missing.
    fn file_at(&self, path: &str, create: bool) -> Result<Rc<File>, String> {
        let (parent, name) = split_parent(path);
        let dir = self.resolve_dir(parent)?;
        // Look first, in a statement of its own, so the shared borrow ends
        // here. Holding it while calling borrow_mut() below would panic
        // with "already borrowed".
        let existing = dir.borrow().get(name);
        match existing {
            Some(Node::File(file)) => Ok(file),
            Some(Node::Dir(_)) => Err(format!("{path}: is a directory")),
            None if create => {
                let file = Rc::new(File::new());
                dir.borrow_mut()
                    .insert(Name::new(name)?, Node::File(Rc::clone(&file)))?;
                Ok(file)
            }
            None => Err(format!("{path}: no such file or directory")),
        }
    }

    fn ln(&mut self, file_path: &str, dir_path: &str) -> Result<String, String> {
        let file = self.file_at(file_path, false)?;
        let destination = self.resolve_dir(dir_path)?;
        let (_, name) = split_parent(file_path);
        destination
            .borrow_mut()
            .insert(Name::new(name)?, Node::File(Rc::clone(&file)))?;
        // Every directory entry is one Rc owner; `file` here is one more.
        let links = Rc::strong_count(&file) - 1;
        Ok(format!("{name}: {links} links"))
    }

    fn rm(&mut self, name: &str) -> Result<String, String> {
        let freed_before = self.log.borrow().len();
        let removed = self
            .cwd
            .borrow_mut()
            .remove(name)
            .ok_or_else(|| format!("{name}: no such file or directory"))?;
        let mut report = match &removed {
            Node::File(file) => match Rc::strong_count(file) - 1 {
                0 => format!("removed {name}: that was its last name, the file is freed"),
                others => format!("removed {name}: the file lives on under {others} other name(s)"),
            },
            Node::Dir(_) => format!("removed {name}/"),
        };
        drop(removed); // std::mem::drop: free it NOW, not at the end of the function
        for line in &self.log.borrow()[freed_before..] {
            report.push('\n');
            report.push_str(line);
        }
        Ok(report)
    }

    fn stat(&self, path: &str) -> Result<String, String> {
        // `node` is one more owner while we look, so subtract it.
        let node = self.resolve(path)?;
        Ok(match &node {
            Node::File(file) => format!(
                "file: {} bytes, {} link(s), read {} time(s)",
                file.len(),
                Rc::strong_count(file) - 1,
                file.reads()
            ),
            Node::Dir(dir) => format!(
                "directory {}: entries {}, Rc owners {}, Weak refs {}",
                dir.borrow().path(),
                dir.borrow().entries().count(),
                Rc::strong_count(dir) - 1,
                Rc::weak_count(dir)
            ),
        })
    }
}

/// `"a/b/c"` gives `("a/b", "c")`, `"/c"` gives `("/", "c")`, and `"c"`
/// gives `(".", "c")`.
fn split_parent(path: &str) -> (&str, &str) {
    match path.rsplit_once('/') {
        Some(("", name)) => ("/", name),
        Some((parent, name)) => (parent, name),
        None => (".", path),
    }
}

fn draw_tree(dir: &Dir, indent: &str, out: &mut String) {
    let count = dir.entries().count();
    for (i, (name, node)) in dir.entries().enumerate() {
        let last = i + 1 == count;
        let (branch, more) = if last {
            ("└── ", "    ")
        } else {
            ("├── ", "│   ")
        };
        out.push_str(&format!("\n{indent}{branch}{name}"));
        match node {
            Node::File(file) if Rc::strong_count(file) > 1 => {
                out.push_str(&format!(" ({} links)", Rc::strong_count(file)));
            }
            Node::File(_) => {}
            Node::Dir(child) => {
                out.push('/');
                draw_tree(&child.borrow(), &format!("{indent}{more}"), out);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shell_with(commands: &[&str]) -> Shell {
        let mut shell = Shell::new();
        for command in commands {
            shell.run(command).unwrap();
        }
        shell
    }

    #[test]
    fn mkdir_cd_and_dot_dot_follow_the_weak_parent() {
        let mut sh = shell_with(&["mkdir a", "mkdir a/b", "cd a/b"]);
        assert_eq!(sh.pwd(), "/a/b");
        sh.run("cd ..").unwrap();
        assert_eq!(sh.pwd(), "/a");
        sh.run("cd ../..").unwrap(); // the root's parent is the root
        assert_eq!(sh.pwd(), "/");
        sh.run("cd /a/b").unwrap();
        assert_eq!(sh.run("pwd").unwrap(), "/a/b");
    }

    #[test]
    fn ls_lists_sorted_entries() {
        let mut sh = shell_with(&["mkdir zoo", "touch apple", "write b hi"]);
        assert_eq!(
            sh.run("ls").unwrap(),
            "apple  (0 bytes)\nb  (3 bytes)\nzoo/"
        );
    }

    #[test]
    fn a_hard_link_is_one_file_with_two_owners() {
        let mut sh = shell_with(&["mkdir docs", "write docs/notes hello", "ln docs/notes /"]);
        sh.run("write notes again").unwrap(); // write through the second name
        assert_eq!(sh.run("cat docs/notes").unwrap(), "hello\nagain");
        assert!(sh.run("stat notes").unwrap().contains("2 link(s)"));
        let report = sh.run("rm notes").unwrap();
        assert!(report.contains("lives on under 1 other name"), "{report}");
        assert!(sh.run("stat docs/notes").unwrap().contains("1 link(s)"));
    }

    #[test]
    fn rm_frees_a_directory_and_everything_in_it() {
        let mut sh = shell_with(&["mkdir a", "mkdir a/b", "mkdir a/b/c"]);
        let report = sh.run("rm a").unwrap();
        assert_eq!(
            report,
            "removed a/\nfreed directory a\nfreed directory b\nfreed directory c"
        );
    }

    #[test]
    fn dropping_the_shell_frees_everything_so_nothing_leaks() {
        let sh = shell_with(&["mkdir a", "mkdir a/b", "cd a/b", "write f x", "ln f /"]);
        let log = sh.drop_log();
        assert!(log.borrow().is_empty());
        drop(sh);
        assert_eq!(log.borrow().len(), 3, "/, a and b were all freed");
        assert_eq!(Rc::strong_count(&log), 1, "only our handle is left");
    }

    #[test]
    fn cat_counts_reads_through_a_shared_reference() {
        let mut sh = shell_with(&["write f x", "cat f", "cat f"]);
        assert!(sh.run("stat f").unwrap().contains("read 2 time(s)"));
    }

    #[test]
    fn stat_shows_strong_and_weak_counts_of_a_directory() {
        let mut sh = shell_with(&["mkdir a", "mkdir a/b", "mkdir a/c"]);
        // Owners of `a`: the root's entry. Weak refs: the parent links of b and c.
        assert_eq!(
            sh.run("stat a").unwrap(),
            "directory /a: entries 2, Rc owners 1, Weak refs 2"
        );
        sh.run("cd a").unwrap(); // now the shell's cwd owns it too
        assert!(sh.run("stat .").unwrap().contains("Rc owners 2"));
    }

    #[test]
    fn mistakes_are_errors_not_panics() {
        let mut sh = shell_with(&["touch f", "mkdir d"]);
        assert_eq!(sh.run("cd f"), Err("f: not a directory".to_string()));
        assert_eq!(sh.run("cat d"), Err("d: is a directory".to_string()));
        assert!(sh.run("mkdir d").unwrap_err().contains("already exists"));
        assert!(sh.run("mkdir ..").unwrap_err().contains("not a valid name"));
        assert!(sh.run("rm nothing").is_err());
        assert!(sh.run("fly").unwrap_err().contains("command not found"));
        assert!(sh.run("cd").unwrap_err().starts_with("usage: cd PATH"));
    }

    #[test]
    fn history_and_tree() {
        let mut sh = shell_with(&["mkdir a", "write a/f x", "ln a/f /"]);
        assert_eq!(
            sh.run("history").unwrap(),
            "  4  history\n  3  ln a/f /\n  2  write a/f x\n  1  mkdir a"
        );
        assert_eq!(
            sh.run("tree").unwrap(),
            "/\n├── a/\n│   └── f (2 links)\n└── f (2 links)"
        );
    }
}
