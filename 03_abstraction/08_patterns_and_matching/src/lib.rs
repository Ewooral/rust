//! The robot interpreter: a toy robot on a grid that follows a script of
//! text commands, one per line.
//!
//! ```text
//! # comments and blank lines are skipped
//! move 3          (also: forward 3, f 3, or just `move` for one step)
//! turn left       (turn right, turn around, or just `left` / `right`)
//! paint red       (red, green, blue, white, black, or `paint 10 200 30`)
//! teleport -2 5
//! say hello there
//! undo
//! ```
//!
//! Parsing turns each line into a [`Command`]; running applies each command
//! to a [`Robot`] and records an [`Event`]. Every kind of pattern from Book
//! chapter 19 is used somewhere below, and the comments name each one.

use std::collections::VecDeque;
use std::fmt;

/// The world runs from -20 to 20 on both axes.
pub const MIN_COORD: i32 = -20;
pub const MAX_COORD: i32 = 20;
/// The most steps one `move` may take.
pub const MAX_STEPS: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub const HOME: Point = Point { x: 0, y: 0 };

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Point { x, y } = self; // let + struct pattern
        write!(f, "({x}, {y})")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    /// One step in this direction, as (dx, dy).
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, 1),
            Direction::East => (1, 0),
            Direction::South => (0, -1),
            Direction::West => (-1, 0),
        }
    }

    pub fn left(self) -> Direction {
        match self {
            Direction::North => Direction::West,
            Direction::West => Direction::South,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
        }
    }

    pub fn arrow(self) -> char {
        match self {
            Direction::North => '^',
            Direction::East => '>',
            Direction::South => 'v',
            Direction::West => '<',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    Left,
    Right,
    Around,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Colour {
    Red,
    Green,
    Blue,
    Rgb(u8, u8, u8),
}

impl Colour {
    /// The letter the robot leaves on the map in this colour.
    pub fn mark(&self) -> char {
        match self {
            Colour::Red => 'r',
            Colour::Green => 'g',
            Colour::Blue => 'b',
            Colour::Rgb(0, 0, 0) => '#', // literals inside a variant
            Colour::Rgb(255, 255, 255) => 'o', //
            Colour::Rgb(r, g, b) if r == g && g == b => '+', // a guard: any other grey
            Colour::Rgb(..) => '*',      // `..`: ignore all three
        }
    }
}

/// One line of the script, understood.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Move(u32),
    Turn(Turn),
    Paint(Colour),
    Teleport { x: i32, y: i32 },
    Say(String),
    Undo,
}

/// Why a line couldn't be understood.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Unknown(String),
    Usage(&'static str),
    NotANumber(String),
    TooFar(u32),
    OffTheMap(i32),
    BadColour(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Unknown(word) => write!(f, "unknown command {word:?}"),
            ParseError::Usage(usage) => write!(f, "usage: {usage}"),
            ParseError::NotANumber(text) => write!(f, "{text:?} is not a number"),
            ParseError::TooFar(steps) => write!(f, "{steps} steps is too far (max {MAX_STEPS})"),
            ParseError::OffTheMap(n) => {
                write!(f, "{n} is off the map ({MIN_COORD} to {MAX_COORD})")
            }
            ParseError::BadColour(name) => write!(f, "no such colour {name:?}"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parses one line. `Ok(None)` means "nothing to do": a blank line or a comment.
pub fn parse_line(line: &str) -> Result<Option<Command>, ParseError> {
    let words: Vec<&str> = line.split_whitespace().collect();

    // Slice patterns on the words, with literals, `|`, `@` and guards.
    let command = match words.as_slice() {
        [] => return Ok(None),
        [first, ..] if first.starts_with('#') => return Ok(None),

        ["move" | "forward" | "f"] => Command::Move(1),
        ["move" | "forward" | "f", steps] => Command::Move(parse_steps(steps)?),

        ["turn", "left" | "l"] | ["left"] => Command::Turn(Turn::Left),
        ["turn", "right" | "r"] | ["right"] => Command::Turn(Turn::Right),
        ["turn", "around"] => Command::Turn(Turn::Around),

        ["paint", name] => Command::Paint(parse_colour(name)?),
        ["paint", r, g, b] => Command::Paint(Colour::Rgb(
            parse_number(r)?,
            parse_number(g)?,
            parse_number(b)?,
        )),

        ["teleport", x, y] => Command::Teleport {
            x: parse_coord(x)?,
            y: parse_coord(y)?,
        },

        ["say", words @ ..] if !words.is_empty() => Command::Say(words.join(" ")),
        ["undo"] => Command::Undo,

        // A known command with the wrong arguments: `@` keeps which one.
        [
            known @ ("move" | "forward" | "f" | "turn" | "paint" | "teleport" | "say" | "undo"),
            ..,
        ] => {
            return Err(ParseError::Usage(usage(known)));
        }
        [unknown, ..] => return Err(ParseError::Unknown(unknown.to_string())),
    };
    Ok(Some(command))
}

fn usage(command: &str) -> &'static str {
    match command {
        "move" | "forward" | "f" => "move [steps]",
        "turn" => "turn left|right|around",
        "paint" => "paint <colour> | paint <r> <g> <b>",
        "teleport" => "teleport <x> <y>",
        "say" => "say <words...>",
        _ => "undo",
    }
}

fn parse_steps(text: &str) -> Result<u32, ParseError> {
    let steps = parse_number::<u32>(text)?;
    match steps {
        0 => Err(ParseError::Usage("move [steps], with at least 1 step")),
        n @ 1..=MAX_STEPS => Ok(n), // `@` + a range using a constant
        too_far => Err(ParseError::TooFar(too_far)),
    }
}

fn parse_coord(text: &str) -> Result<i32, ParseError> {
    match parse_number::<i32>(text)? {
        n @ MIN_COORD..=MAX_COORD => Ok(n),
        off => Err(ParseError::OffTheMap(off)),
    }
}

fn parse_number<T: std::str::FromStr>(text: &str) -> Result<T, ParseError> {
    // let ... else: bind the number, or leave early with an error.
    let Ok(n) = text.parse::<T>() else {
        return Err(ParseError::NotANumber(text.to_string()));
    };
    Ok(n)
}

fn parse_colour(name: &str) -> Result<Colour, ParseError> {
    match name.to_lowercase().as_str() {
        "red" => Ok(Colour::Red),
        "green" => Ok(Colour::Green),
        "blue" => Ok(Colour::Blue),
        "white" => Ok(Colour::Rgb(255, 255, 255)),
        "black" => Ok(Colour::Rgb(0, 0, 0)),
        other => Err(ParseError::BadColour(other.to_string())),
    }
}

/// What happened when the robot ran one command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Moved { from: Point, to: Point, steps: u32 },
    Bumped { wall: Point, steps_taken: u32 },
    Turned(Direction),
    Painted(Colour),
    Teleported(Point),
    Said(String),
    Undid,
    NothingToUndo,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::Moved { from, to, steps: 1 } => write!(f, "stepped from {from} to {to}"),
            Event::Moved { from, to, steps } => {
                write!(f, "walked {steps} steps from {from} to {to}")
            }
            Event::Bumped {
                wall,
                steps_taken: 0,
            } => write!(f, "bumped into the edge at {wall} without moving"),
            Event::Bumped { wall, steps_taken } => {
                write!(
                    f,
                    "took {steps_taken} steps, then bumped into the edge at {wall}"
                )
            }
            Event::Turned(direction) => write!(f, "now facing {direction:?}"),
            Event::Painted(colour) => write!(f, "pen is now {colour:?} ({})", colour.mark()),
            // `@` on a struct pattern: test for home AND keep the whole point.
            Event::Teleported(home @ Point { x: 0, y: 0 }) => {
                write!(f, "teleported home to {home}")
            }
            Event::Teleported(point) => {
                write!(f, "teleported to {point}, {}", describe_position(point))
            }
            Event::Said(text) => write!(f, "says {text:?}"),
            Event::Undid => write!(f, "undid the last command"),
            Event::NothingToUndo => write!(f, "nothing to undo"),
        }
    }
}

/// Where a point is, in words. The PARAMETER is a pattern: `&Point { x, y }`
/// takes the borrowed point apart into two plain integers.
pub fn describe_position(&Point { x, y }: &Point) -> &'static str {
    match (x, y) {
        (0, 0) => "at home",
        (0, _) | (_, 0) => "on a main road",
        (x, y) if x == y || x == -y => "on a diagonal",
        (1.., 1..) => "in the north-east",
        (..0, 1..) => "in the north-west",
        (..0, ..0) => "in the south-west",
        (1.., ..0) => "in the south-east",
    }
}

/// Everything `undo` needs to put back.
#[derive(Debug, Clone, PartialEq)]
struct Snapshot {
    position: Point,
    facing: Direction,
    colour: Colour,
    trail_len: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Robot {
    pub position: Point,
    pub facing: Direction,
    pub colour: Colour,
    /// Every square the robot has walked onto, with the pen's mark.
    pub trail: Vec<(Point, char)>,
    history: Vec<Snapshot>,
}

impl Default for Robot {
    fn default() -> Self {
        Robot::new()
    }
}

impl Robot {
    pub fn new() -> Robot {
        Robot {
            position: HOME,
            facing: Direction::North,
            colour: Colour::Blue,
            trail: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Runs one command and reports what happened.
    pub fn apply(&mut self, command: &Command) -> Event {
        // Remember the state first, so any change can be undone.
        if !matches!(command, Command::Undo | Command::Say(_)) {
            self.history.push(self.snapshot());
        }

        match command {
            Command::Move(steps) => self.walk(*steps),
            // Nested enum patterns:
            Command::Turn(Turn::Left) => self.turn(self.facing.left()),
            Command::Turn(Turn::Right) => self.turn(self.facing.left().left().left()),
            Command::Turn(Turn::Around) => self.turn(self.facing.left().left()),
            Command::Paint(colour) => {
                self.colour = colour.clone();
                Event::Painted(colour.clone())
            }
            // A struct variant: the fields x and y become variables.
            Command::Teleport { x, y } => {
                self.position = Point { x: *x, y: *y };
                self.trail.push((self.position, self.colour.mark()));
                Event::Teleported(self.position)
            }
            Command::Say(text) => Event::Said(text.clone()),
            Command::Undo => self.undo(),
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            position: self.position,
            facing: self.facing,
            colour: self.colour.clone(),
            trail_len: self.trail.len(),
        }
    }

    fn turn(&mut self, to: Direction) -> Event {
        self.facing = to;
        Event::Turned(to)
    }

    fn walk(&mut self, steps: u32) -> Event {
        let (dx, dy) = self.facing.delta(); // let + tuple pattern
        let from = self.position;
        for taken in 0..steps {
            let next = Point {
                x: self.position.x + dx,
                y: self.position.y + dy,
            };
            match next {
                // Ranges inside a struct pattern: "both coordinates on the map".
                Point {
                    x: MIN_COORD..=MAX_COORD,
                    y: MIN_COORD..=MAX_COORD,
                } => {
                    self.position = next;
                    self.trail.push((next, self.colour.mark()));
                }
                wall => {
                    return Event::Bumped {
                        wall,
                        steps_taken: taken,
                    };
                }
            }
        }
        Event::Moved {
            from,
            to: self.position,
            steps,
        }
    }

    fn undo(&mut self) -> Event {
        // if let + a struct pattern that takes the snapshot apart.
        if let Some(Snapshot {
            position,
            facing,
            colour,
            trail_len,
        }) = self.history.pop()
        {
            self.position = position;
            self.facing = facing;
            self.colour = colour;
            self.trail.truncate(trail_len);
            Event::Undid
        } else {
            Event::NothingToUndo
        }
    }

    /// A little map of everything the robot has visited.
    pub fn map(&self) -> String {
        // Closure parameter pattern: `&(point, _)` takes each trail entry apart.
        let points: Vec<Point> = self
            .trail
            .iter()
            .map(|&(point, _)| point)
            .chain([self.position, HOME])
            .collect();
        let min_x = points.iter().map(|p| p.x).min().unwrap_or(0);
        let max_x = points.iter().map(|p| p.x).max().unwrap_or(0);
        let min_y = points.iter().map(|p| p.y).min().unwrap_or(0);
        let max_y = points.iter().map(|p| p.y).max().unwrap_or(0);

        let mut rows = Vec::new();
        for y in (min_y..=max_y).rev() {
            let mut row = String::new();
            for x in min_x..=max_x {
                let here = Point { x, y };
                // if / else if let / else if / else, all in one chain.
                let cell = if here == self.position {
                    self.facing.arrow()
                } else if let Some(&(_, mark)) = self.trail.iter().rev().find(|(p, _)| *p == here) {
                    mark
                } else if here == HOME {
                    'H'
                } else {
                    '.'
                };
                row.push(cell);
            }
            rows.push(row);
        }
        rows.join("\n")
    }
}

/// The result of running a whole script.
#[derive(Debug)]
pub struct Report {
    pub robot: Robot,
    /// (line number, what happened)
    pub events: Vec<(usize, Event)>,
    /// (line number, why it was skipped)
    pub errors: Vec<(usize, ParseError)>,
}

/// Parses every line, then runs the good ones in order.
pub fn run_script(script: &str) -> Report {
    let mut queue = VecDeque::new();
    let mut errors = Vec::new();

    // for + a tuple pattern from enumerate().
    for (index, line) in script.lines().enumerate() {
        match parse_line(line) {
            Ok(Some(command)) => queue.push_back((index + 1, command)), // nested patterns
            Ok(None) => {}
            Err(error) => errors.push((index + 1, error)),
        }
    }

    let mut robot = Robot::new();
    let mut events = Vec::new();
    // while let: keep going as long as the queue has another command.
    while let Some((line, command)) = queue.pop_front() {
        events.push((line, robot.apply(&command)));
    }

    Report {
        robot,
        events,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_command_form() {
        assert_eq!(parse_line("move"), Ok(Some(Command::Move(1))));
        assert_eq!(parse_line("  f 3 "), Ok(Some(Command::Move(3))));
        assert_eq!(parse_line("left"), Ok(Some(Command::Turn(Turn::Left))));
        assert_eq!(
            parse_line("turn around"),
            Ok(Some(Command::Turn(Turn::Around)))
        );
        assert_eq!(
            parse_line("paint 10 20 30"),
            Ok(Some(Command::Paint(Colour::Rgb(10, 20, 30))))
        );
        assert_eq!(
            parse_line("teleport -2 5"),
            Ok(Some(Command::Teleport { x: -2, y: 5 }))
        );
        assert_eq!(
            parse_line("say hello   there"),
            Ok(Some(Command::Say("hello there".into())))
        );
        assert_eq!(parse_line(""), Ok(None));
        assert_eq!(parse_line("# a comment"), Ok(None));
    }

    #[test]
    fn explains_bad_lines() {
        assert_eq!(parse_line("fly"), Err(ParseError::Unknown("fly".into())));
        assert_eq!(parse_line("move 99"), Err(ParseError::TooFar(99)));
        assert_eq!(
            parse_line("move lots"),
            Err(ParseError::NotANumber("lots".into()))
        );
        assert_eq!(parse_line("teleport 1 50"), Err(ParseError::OffTheMap(50)));
        assert_eq!(
            parse_line("paint purple"),
            Err(ParseError::BadColour("purple".into()))
        );
        assert_eq!(
            parse_line("teleport 1"),
            Err(ParseError::Usage("teleport <x> <y>"))
        );
        assert_eq!(parse_line("say"), Err(ParseError::Usage("say <words...>")));
    }

    #[test]
    fn colour_marks() {
        assert_eq!(Colour::Red.mark(), 'r');
        assert_eq!(Colour::Rgb(0, 0, 0).mark(), '#');
        assert_eq!(Colour::Rgb(128, 128, 128).mark(), '+');
        assert_eq!(Colour::Rgb(1, 2, 3).mark(), '*');
    }

    #[test]
    fn positions_in_words() {
        assert_eq!(describe_position(&HOME), "at home");
        assert_eq!(describe_position(&Point { x: 0, y: -4 }), "on a main road");
        assert_eq!(describe_position(&Point { x: -3, y: 3 }), "on a diagonal");
        assert_eq!(
            describe_position(&Point { x: 2, y: 5 }),
            "in the north-east"
        );
        assert_eq!(
            describe_position(&Point { x: -2, y: -5 }),
            "in the south-west"
        );
    }

    #[test]
    fn walks_turns_and_bumps_into_the_edge() {
        let mut robot = Robot::new();
        robot.apply(&Command::Turn(Turn::Right)); // now East
        let event = robot.apply(&Command::Move(3));
        assert_eq!(
            event,
            Event::Moved {
                from: HOME,
                to: Point { x: 3, y: 0 },
                steps: 3
            }
        );

        robot.apply(&Command::Teleport { x: 19, y: 0 });
        let event = robot.apply(&Command::Move(5));
        assert_eq!(
            event,
            Event::Bumped {
                wall: Point { x: 21, y: 0 },
                steps_taken: 1
            }
        );
        assert_eq!(robot.position, Point { x: 20, y: 0 });
    }

    #[test]
    fn undo_puts_everything_back() {
        let mut robot = Robot::new();
        robot.apply(&Command::Move(2));
        robot.apply(&Command::Paint(Colour::Red));
        robot.apply(&Command::Move(1));
        assert_eq!(robot.trail.len(), 3);

        assert_eq!(robot.apply(&Command::Undo), Event::Undid); // undo the move
        assert_eq!(robot.apply(&Command::Undo), Event::Undid); // undo the paint
        assert_eq!(robot.position, Point { x: 0, y: 2 });
        assert_eq!(robot.colour, Colour::Blue);
        assert_eq!(robot.trail.len(), 2);

        robot.apply(&Command::Undo);
        assert_eq!(robot.apply(&Command::Undo), Event::NothingToUndo);
        assert_eq!(robot, Robot::new());
    }

    #[test]
    fn runs_a_whole_script() {
        let report = run_script("# test\nmove 2\nright\nfly\nmove 1\nsay done\nmove 0");
        assert_eq!(
            report.errors,
            [
                (4, ParseError::Unknown("fly".into())),
                (7, ParseError::Usage("move [steps], with at least 1 step"))
            ]
        );
        assert_eq!(report.events.len(), 4);
        assert_eq!(report.robot.position, Point { x: 1, y: 2 });
        // north 2 (in blue), then east 1; H marks home
        assert_eq!(report.robot.map(), "b>\nb.\nH.");
    }

    #[test]
    fn events_read_well() {
        let moved = Event::Moved {
            from: HOME,
            to: Point { x: 0, y: 1 },
            steps: 1,
        };
        assert_eq!(moved.to_string(), "stepped from (0, 0) to (0, 1)");
        assert_eq!(
            Event::Teleported(HOME).to_string(),
            "teleported home to (0, 0)"
        );
        assert_eq!(
            Event::Teleported(Point { x: 4, y: -4 }).to_string(),
            "teleported to (4, -4), on a diagonal"
        );
    }
}
