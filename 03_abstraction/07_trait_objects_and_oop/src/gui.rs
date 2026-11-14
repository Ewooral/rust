//! A tiny text-mode GUI library: the Book's `gui` crate (18.2), but each
//! component really draws itself, as lines of text.
//!
//! The library defines the `Draw` trait and a `Screen` that holds
//! `Box<dyn Draw>`. It ships a few components, and code OUTSIDE this library
//! can add its own (see `PostCard` in `main.rs`) just by implementing `Draw`.

/// Anything that can draw itself as lines of text.
pub trait Draw {
    /// The component's picture, one `String` per line.
    fn draw(&self) -> Vec<String>;

    /// How many lines the picture is. A default method: written once here,
    /// and every component gets it for free.
    fn height(&self) -> usize {
        self.draw().len()
    }
}

/// A screen full of components of DIFFERENT types.
#[derive(Default)]
pub struct Screen {
    components: Vec<Box<dyn Draw>>,
}

impl Screen {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds any component. `'static` because the Box will own it.
    /// Returns `&mut Self` so calls can be chained.
    pub fn add(&mut self, component: impl Draw + 'static) -> &mut Self {
        self.components.push(Box::new(component));
        self
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Draws every component, top to bottom. Each `draw` call is dynamic
    /// dispatch: the screen neither knows nor cares what the types are.
    pub fn render(&self) -> String {
        let mut lines = Vec::new();
        for component in &self.components {
            lines.extend(component.draw());
        }
        lines.join("\n")
    }

    /// Total height of everything on the screen.
    pub fn height(&self) -> usize {
        self.components.iter().map(|c| c.height()).sum()
    }
}

/// A line of text.
pub struct Label {
    pub text: String,
}

impl Draw for Label {
    fn draw(&self) -> Vec<String> {
        vec![self.text.clone()]
    }
}

/// A button with a border. Width is at least the label plus padding.
pub struct Button {
    pub width: usize,
    pub label: String,
}

impl Draw for Button {
    fn draw(&self) -> Vec<String> {
        let inner = self.width.max(self.label.chars().count() + 2);
        let edge = format!("+{}+", "-".repeat(inner));
        vec![edge.clone(), format!("|{:^inner$}|", self.label), edge]
    }
}

/// A list of options with one selected.
pub struct SelectBox {
    pub options: Vec<String>,
    pub selected: usize,
}

impl Draw for SelectBox {
    fn draw(&self) -> Vec<String> {
        self.options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let mark = if i == self.selected { '*' } else { ' ' };
                format!("({mark}) {option}")
            })
            .collect()
    }
}

/// `[####......] 2/5`
pub struct ProgressBar {
    pub done: usize,
    pub total: usize,
    pub width: usize,
}

impl Draw for ProgressBar {
    fn draw(&self) -> Vec<String> {
        // checked_div is None when total is 0: an empty bar, not a panic.
        let filled = (self.width * self.done.min(self.total))
            .checked_div(self.total)
            .unwrap_or(0);
        vec![format!(
            "[{}{}] {}/{}",
            "#".repeat(filled),
            ".".repeat(self.width - filled),
            self.done,
            self.total
        )]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn components_draw_themselves() {
        let ok = Button {
            width: 6,
            label: "OK".into(),
        };
        assert_eq!(ok.draw(), ["+------+", "|  OK  |", "+------+"]);
        assert_eq!(ok.height(), 3); // the default method

        let pick = SelectBox {
            options: vec!["Yes".into(), "No".into()],
            selected: 1,
        };
        assert_eq!(pick.draw(), ["( ) Yes", "(*) No"]);

        let bar = ProgressBar {
            done: 1,
            total: 4,
            width: 8,
        };
        assert_eq!(bar.draw(), ["[##......] 1/4"]);

        let empty = ProgressBar {
            done: 0,
            total: 0,
            width: 4,
        };
        assert_eq!(empty.draw(), ["[....] 0/0"]);
    }

    #[test]
    fn a_screen_holds_different_types() {
        let mut screen = Screen::new();
        assert!(screen.is_empty());
        screen
            .add(Label {
                text: "Hello".into(),
            })
            .add(Button {
                width: 4,
                label: "Go".into(),
            });
        assert_eq!(screen.len(), 2);
        assert_eq!(screen.height(), 4);
        assert_eq!(screen.render(), "Hello\n+----+\n| Go |\n+----+");
    }

    #[test]
    fn button_grows_to_fit_its_label() {
        let b = Button {
            width: 2,
            label: "Cancel".into(),
        };
        assert_eq!(b.draw()[1], "| Cancel |");
    }
}
