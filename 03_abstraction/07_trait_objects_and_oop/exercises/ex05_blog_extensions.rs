//! Exercise 5: extend the state pattern (the Book's own suggestions, 18.3).
//!
//! This is the trait-object blog post. Add three rules, each by changing
//! STATES, not by adding `if`s about states to `Post`:
//!
//! 1. `reject()` sends a post that is pending review back to Draft.
//! 2. Publishing needs TWO calls to `approve()`. (`PendingReview` counts its
//!    approvals; rejecting starts the count again.)
//! 3. Text can only be added while the post is a Draft. `Post::add_text`
//!    asks the state (`allows_editing`) and does nothing if it says no.
//!
//! Every `todo!()` is yours to replace.
//!
//! Test: cargo test -p trait_objects_and_oop --features exercises --bin ex05_blog_extensions

#![allow(unused_variables, dead_code)]

pub struct Post {
    state: Option<Box<dyn State>>,
    content: String,
}

impl Post {
    pub fn new() -> Post {
        Post {
            state: Some(Box::new(Draft)),
            content: String::new(),
        }
    }

    /// Rule 3: only if the current state allows editing.
    pub fn add_text(&mut self, text: &str) {
        todo!()
    }

    pub fn content(&self) -> &str {
        self.current().content(self)
    }

    pub fn status(&self) -> &'static str {
        self.current().name()
    }

    pub fn request_review(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.request_review())
        }
    }

    pub fn approve(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.approve())
        }
    }

    /// Rule 1: same shape as `approve`.
    pub fn reject(&mut self) {
        todo!()
    }

    fn current(&self) -> &dyn State {
        self.state.as_deref().expect("a post always has a state")
    }
}

impl Default for Post {
    fn default() -> Self {
        Post::new()
    }
}

trait State {
    fn name(&self) -> &'static str;
    fn request_review(self: Box<Self>) -> Box<dyn State>;
    fn approve(self: Box<Self>) -> Box<dyn State>;
    fn reject(self: Box<Self>) -> Box<dyn State>;

    fn content<'a>(&self, _post: &'a Post) -> &'a str {
        ""
    }

    /// May text be added in this state? Most states say no.
    fn allows_editing(&self) -> bool {
        false
    }
}

struct Draft;

struct PendingReview {
    approvals: u32,
}

struct Published;

impl State for Draft {
    fn name(&self) -> &'static str {
        "draft"
    }
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        Box::new(PendingReview { approvals: 0 })
    }
    fn approve(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn reject(self: Box<Self>) -> Box<dyn State> {
        todo!()
    }
    // TODO: override `allows_editing` (rule 3)
}

impl State for PendingReview {
    fn name(&self) -> &'static str {
        "pending review"
    }
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn approve(self: Box<Self>) -> Box<dyn State> {
        todo!() // rule 2
    }
    fn reject(self: Box<Self>) -> Box<dyn State> {
        todo!() // rule 1
    }
}

impl State for Published {
    fn name(&self) -> &'static str {
        "published"
    }
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn approve(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn reject(self: Box<Self>) -> Box<dyn State> {
        todo!()
    }
    fn content<'a>(&self, post: &'a Post) -> &'a str {
        &post.content
    }
}

fn main() {
    let mut post = Post::new();
    post.add_text("Rust has no classes. ");
    post.request_review();
    post.add_text("(ignored: not a draft) ");
    post.reject();
    post.add_text("And that's fine.");
    post.request_review();
    post.approve();
    println!("after one approval: {}", post.status());
    post.approve();
    println!("after two: {} -> {:?}", post.status(), post.content());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_sends_the_post_back_to_draft() {
        let mut post = Post::new();
        post.add_text("hello");
        post.request_review();
        post.reject();
        assert_eq!(post.status(), "draft");
        assert_eq!(post.content(), "");

        post.reject(); // rejecting a draft changes nothing
        assert_eq!(post.status(), "draft");
    }

    #[test]
    fn publishing_needs_two_approvals() {
        let mut post = Post::new();
        post.add_text("hello");
        post.request_review();
        post.approve();
        assert_eq!(post.status(), "pending review");
        post.approve();
        assert_eq!(post.status(), "published");
        assert_eq!(post.content(), "hello");

        // a rejection in between starts the count again
        let mut post = Post::new();
        post.request_review();
        post.approve();
        post.reject();
        post.request_review();
        post.approve();
        assert_eq!(post.status(), "pending review");
    }

    #[test]
    fn text_can_only_be_added_to_a_draft() {
        let mut post = Post::new();
        post.add_text("a");
        post.request_review();
        post.add_text("b"); // ignored
        post.reject();
        post.add_text("c");
        post.request_review();
        post.approve();
        post.approve();
        post.add_text("d"); // ignored
        assert_eq!(post.content(), "ac");
    }
}
