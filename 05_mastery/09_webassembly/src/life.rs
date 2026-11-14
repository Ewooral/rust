//! Conway's Game of Life: the pure-Rust core.
//!
//! Nothing in this file knows about JavaScript or WebAssembly, which is why
//! it can be tested on the PC like any other Rust code. The design choices
//! that matter for the web are about **memory layout**:
//!
//! - The cells live in ONE flat `Vec<Cell>`, row after row, and `Cell` is
//!   `#[repr(u8)]` (0 or 1). JavaScript can then read the whole grid straight
//!   out of WebAssembly memory as a `Uint8Array`, without copying anything.
//! - `tick` writes into a second buffer and swaps, so a running animation
//!   never allocates.
//!
//! The rules: a live cell with 2 or 3 live neighbours lives on; a dead cell
//! with exactly 3 live neighbours comes alive; every other cell dies or
//! stays dead. The grid wraps around at the edges, like a doughnut.

use std::fmt;

use crate::host::Host;
use crate::pattern::Pattern;

/// One cell. `#[repr(u8)]` fixes its size and values, so other languages
/// can read it from memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Cell {
    /// 0
    Dead = 0,
    /// 1
    Alive = 1,
}

/// A grid of cells that wraps around at the edges.
#[derive(Clone, PartialEq, Eq)]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    next: Vec<Cell>,
    generation: u64,
}

impl Universe {
    /// An empty universe. A zero size is bumped up to 1.
    pub fn new(width: u32, height: u32) -> Universe {
        let (width, height) = (width.max(1), height.max(1));
        let size = (width * height) as usize;
        Universe {
            width,
            height,
            cells: vec![Cell::Dead; size],
            next: vec![Cell::Dead; size],
            generation: 0,
        }
    }

    /// Columns.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Rows.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// How many ticks have happened.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// How many cells are alive.
    pub fn population(&self) -> usize {
        self.cells.iter().filter(|&&c| c == Cell::Alive).count()
    }

    /// The flat index of a cell. Rows and columns wrap around.
    fn index(&self, row: u32, col: u32) -> usize {
        ((row % self.height) * self.width + (col % self.width)) as usize
    }

    /// The cell at `row`, `col` (wrapping).
    pub fn get(&self, row: u32, col: u32) -> Cell {
        self.cells[self.index(row, col)]
    }

    /// Sets the cell at `row`, `col` (wrapping).
    pub fn set(&mut self, row: u32, col: u32, cell: Cell) {
        let i = self.index(row, col);
        self.cells[i] = cell;
    }

    /// Flips one cell: what a mouse click does.
    pub fn toggle(&mut self, row: u32, col: u32) {
        let i = self.index(row, col);
        self.cells[i] = match self.cells[i] {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        };
    }

    /// Kills every cell and resets the generation count.
    pub fn clear(&mut self) {
        self.cells.fill(Cell::Dead);
        self.generation = 0;
    }

    fn live_neighbours(&self, row: u32, col: u32) -> u8 {
        let mut count = 0;
        // Adding (height - 1) is "minus one, wrapping", without going negative.
        for dr in [self.height - 1, 0, 1] {
            for dc in [self.width - 1, 0, 1] {
                if (dr, dc) != (0, 0) {
                    count += self.get(row + dr, col + dc) as u8;
                }
            }
        }
        count
    }

    /// One generation. Returns how many cells changed, so a screen only
    /// needs redrawing when something happened.
    pub fn tick(&mut self) -> u32 {
        let mut changed = 0;
        for row in 0..self.height {
            for col in 0..self.width {
                let i = self.index(row, col);
                let cell = self.cells[i];
                let next = match (cell, self.live_neighbours(row, col)) {
                    (Cell::Alive, 2 | 3) => Cell::Alive,
                    (Cell::Dead, 3) => Cell::Alive,
                    _ => Cell::Dead,
                };
                changed += u32::from(next != cell);
                self.next[i] = next;
            }
        }
        std::mem::swap(&mut self.cells, &mut self.next);
        self.generation += 1;
        changed
    }

    /// The cells, row after row.
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Where the cells start in memory. In WebAssembly this number is an
    /// offset into the module's linear memory, which JavaScript can view as
    /// `new Uint8Array(memory.buffer, ptr, width * height)`.
    pub fn cells_ptr(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    /// Stamps a pattern with its top-left corner at `row`, `col` (wrapping).
    pub fn insert(&mut self, pattern: &Pattern, row: u32, col: u32) {
        for &(r, c) in &pattern.alive {
            self.set(row + r, col + c, Cell::Alive);
        }
    }

    /// Fills the grid at random: each cell is alive with chance `density`.
    /// The randomness comes from the [`Host`]: `Math.random` in a browser.
    pub fn randomize(&mut self, host: &mut impl Host, density: f64) {
        for cell in &mut self.cells {
            *cell = if host.random() < density {
                Cell::Alive
            } else {
                Cell::Dead
            };
        }
        self.generation = 0;
    }
}

/// `#` for alive, `.` for dead, one line per row.
impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.cells.chunks(self.width as usize) {
            for &cell in row {
                f.write_str(if cell == Cell::Alive { "#" } else { "." })?;
            }
            f.write_str("\n")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Universe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Universe {}x{}, generation {}:\n{self}",
            self.width, self.height, self.generation
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::ScriptedHost;
    use crate::pattern;

    fn universe_from(text: &str) -> Universe {
        let p = pattern::parse(text).unwrap();
        let mut u = Universe::new(p.width, p.height);
        u.insert(&p, 0, 0);
        u
    }

    #[test]
    fn a_blinker_blinks() {
        let start = universe_from(".....\n..O..\n..O..\n..O..\n.....");
        let mut u = start.clone();
        assert_eq!(u.tick(), 4, "2 cells die, 2 are born");
        assert_eq!(u.to_string(), ".....\n.....\n.###.\n.....\n.....\n");
        u.tick();
        assert_eq!(u.cells(), start.cells(), "back where it started");
        assert_eq!(u.generation(), 2);
    }

    #[test]
    fn a_block_never_changes() {
        let mut u = universe_from("....\n.OO.\n.OO.\n....");
        assert_eq!(u.tick(), 0);
        assert_eq!(u.population(), 4);
        assert_eq!(u.generation(), 1);
    }

    #[test]
    fn a_glider_wraps_around_the_edges() {
        let glider = pattern::builtin("glider").unwrap();
        let mut u = Universe::new(6, 6);
        u.insert(&glider, 0, 0);
        let start = u.to_string();
        // A glider moves one cell diagonally every 4 generations, so after
        // 24 it has crossed the 6x6 doughnut and come home.
        for _ in 0..24 {
            u.tick();
        }
        assert_eq!(u.to_string(), start);
        assert_eq!(u.population(), 5);
    }

    #[test]
    fn coordinates_wrap() {
        let mut u = Universe::new(3, 2);
        u.toggle(2, 4); // row 2 -> 0, col 4 -> 1
        assert_eq!(u.get(0, 1), Cell::Alive);
        u.toggle(0, 1);
        assert_eq!(u.population(), 0);
    }

    #[test]
    fn cells_are_one_byte_each_for_javascript() {
        assert_eq!(std::mem::size_of::<Cell>(), 1);
        let mut u = Universe::new(2, 2);
        u.set(1, 0, Cell::Alive);
        let ptr = u.cells_ptr().cast::<u8>();
        // SAFETY: the Vec holds 4 initialised one-byte Cells.
        let bytes = unsafe { std::slice::from_raw_parts(ptr, 4) };
        assert_eq!(bytes, [0, 0, 1, 0]);
    }

    #[test]
    fn randomize_uses_the_host() {
        let mut host = ScriptedHost::new(&[0.1, 0.9, 0.4, 0.6]);
        let mut u = Universe::new(2, 2);
        u.randomize(&mut host, 0.5);
        assert_eq!(
            u.cells(),
            [Cell::Alive, Cell::Dead, Cell::Alive, Cell::Dead]
        );
    }

    #[test]
    fn zero_sizes_become_one() {
        let u = Universe::new(0, 0);
        assert_eq!((u.width(), u.height()), (1, 1));
    }
}
