//! Mini project for Book chapter 18: a blog dashboard.
//!
//! The library (`src/lib.rs`) has two parts:
//!
//! - `gui`: a tiny text GUI, with the `Draw` trait and a `Screen` of `Box<dyn Draw>`
//! - `blog`: the Book's post workflow, as a state pattern (`blog::dynamic`)
//!   and as types (`blog::typed`)
//!
//! This file is a USER of that library. It invents its own component,
//! `PostCard`, and puts it on the library's screen next to the built-in ones,
//! without the library knowing `PostCard` exists.
//!
//!   cargo run -p trait_objects_and_oop
//!   cargo test -p trait_objects_and_oop

use trait_objects_and_oop::blog::{dynamic, typed};
use trait_objects_and_oop::gui::{Button, Draw, Label, ProgressBar, Screen, SelectBox};

/// How much of a published post the card shows.
const PREVIEW_CHARS: usize = 24;

/// A component the gui library has never heard of.
struct PostCard {
    title: String,
    status: &'static str,
    preview: String,
}

impl PostCard {
    fn from_post(post: &dynamic::Post) -> PostCard {
        let preview = match post.content() {
            "" => "(hidden until published)".to_string(),
            text if text.chars().count() > PREVIEW_CHARS => {
                let start: String = text.chars().take(PREVIEW_CHARS).collect();
                format!("{start}...")
            }
            text => text.to_string(),
        };
        PostCard {
            title: post.title().to_string(),
            status: post.status(),
            preview,
        }
    }
}

impl Draw for PostCard {
    fn draw(&self) -> Vec<String> {
        vec![
            format!("* {} [{}]", self.title, self.status),
            format!("    {}", self.preview),
        ]
    }
}

/// Builds one screen out of five different component types.
fn dashboard(posts: &[dynamic::Post]) -> Screen {
    let published = posts.iter().filter(|p| p.status() == "published").count();

    let mut screen = Screen::new();
    screen.add(Label {
        text: "=== Blog dashboard ===".into(),
    });
    for post in posts {
        screen.add(PostCard::from_post(post));
    }
    screen
        .add(Label {
            text: "Published:".into(),
        })
        .add(ProgressBar {
            done: published,
            total: posts.len(),
            width: 12,
        })
        .add(SelectBox {
            options: vec!["Request review".into(), "Approve".into(), "Leave".into()],
            selected: 1,
        })
        .add(Button {
            width: 10,
            label: "Apply".into(),
        });
    screen
}

fn main() {
    // ---- The state-pattern posts: one type, state changes at run time ----
    let mut lunch = dynamic::Post::new("Lunch");
    lunch.add_text("I ate a salad for lunch today");
    lunch.request_review();
    lunch.approve();

    let mut why_rust = dynamic::Post::new("Why Rust");
    why_rust.add_text("Because the compiler has my back.");
    why_rust.request_review();

    let mut holiday = dynamic::Post::new("Holiday plans");
    holiday.add_text("Somewhere warm.");
    holiday.approve(); // ignored: a draft can't be approved

    let posts = [lunch, why_rust, holiday];
    let screen = dashboard(&posts);
    println!("{}", screen.render());
    println!(
        "({} components of 5 different types, {} lines)",
        screen.len(),
        screen.height()
    );

    // ---- The typed posts: each state is a different type ----
    println!();
    println!("The same workflow with states as types:");
    let mut draft = typed::Post::new("Typed lunch");
    draft.add_text("Same salad, checked by the compiler.");
    println!("  {:?} is a DraftPost", draft.title());
    // draft.content();  // E0599: no method named `content` found for struct `DraftPost`
    let pending = draft.request_review();
    println!("  {:?} is a PendingReviewPost", pending.title());
    let published = pending.approve();
    println!(
        "  {:?} is a Post: {:?}",
        published.title(),
        published.content()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn published(title: &str, text: &str) -> dynamic::Post {
        let mut post = dynamic::Post::new(title);
        post.add_text(text);
        post.request_review();
        post.approve();
        post
    }

    #[test]
    fn a_card_hides_unpublished_text() {
        let mut draft = dynamic::Post::new("Secret");
        draft.add_text("not yet");
        let card = PostCard::from_post(&draft);
        assert_eq!(
            card.draw(),
            ["* Secret [draft]", "    (hidden until published)"]
        );
    }

    #[test]
    fn a_card_shortens_long_posts() {
        let card = PostCard::from_post(&published("Long", &"a".repeat(30)));
        assert_eq!(card.draw()[1], format!("    {}...", "a".repeat(24)));

        let card = PostCard::from_post(&published("Short", "hi"));
        assert_eq!(card.draw()[1], "    hi");
    }

    #[test]
    fn the_dashboard_mixes_library_and_user_components() {
        let posts = [published("One", "1"), dynamic::Post::new("Two")];
        let screen = dashboard(&posts);
        // 1 label + 2 cards + label + bar + select + button
        assert_eq!(screen.len(), 7);
        let text = screen.render();
        assert!(text.contains("* One [published]"));
        assert!(text.contains("[######......] 1/2"));
        assert!(text.contains("(*) Approve"));
    }
}
