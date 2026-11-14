//! The state pattern with trait objects (Book 18.3, first half), plus a
//! title and a `status()` so the dashboard can show where each post is.
//!
//! Draft --request_review--> PendingReview --approve--> Published
//!
//! The rules live in the state types, not in `Post`: `Post` only forwards each
//! request to whatever state it currently holds.

/// A blog post whose behaviour depends on its current state.
pub struct Post {
    title: String,
    // `Option` so a method can `take()` the old state out, leaving `None` for
    // a moment, and put the new state back.
    state: Option<Box<dyn State>>,
    content: String,
}

impl Post {
    /// Every post starts as a draft.
    pub fn new(title: &str) -> Post {
        Post {
            title: title.to_string(),
            state: Some(Box::new(Draft)),
            content: String::new(),
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn add_text(&mut self, text: &str) {
        self.content.push_str(text);
    }

    /// Empty until the post is published: the state decides.
    pub fn content(&self) -> &str {
        self.current().content(self)
    }

    /// "draft", "pending review" or "published".
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

    /// The state is only `None` for a moment inside the methods above.
    fn current(&self) -> &dyn State {
        self.state
            .as_deref()
            .expect("a post always has a state between method calls")
    }
}

/// Private: callers can't see the states, only the behaviour of `Post`.
trait State {
    fn name(&self) -> &'static str;

    // `self: Box<Self>` consumes the boxed state: the old state is used up
    // and replaced by whatever we return.
    fn request_review(self: Box<Self>) -> Box<dyn State>;
    fn approve(self: Box<Self>) -> Box<dyn State>;

    /// Default: hide the content. Only `Published` overrides it.
    fn content<'a>(&self, _post: &'a Post) -> &'a str {
        ""
    }
}

struct Draft;
struct PendingReview;
struct Published;

impl State for Draft {
    fn name(&self) -> &'static str {
        "draft"
    }
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        Box::new(PendingReview)
    }
    fn approve(self: Box<Self>) -> Box<dyn State> {
        self // a draft can't be approved
    }
}

impl State for PendingReview {
    fn name(&self) -> &'static str {
        "pending review"
    }
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn approve(self: Box<Self>) -> Box<dyn State> {
        Box::new(Published)
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
    fn content<'a>(&self, post: &'a Post) -> &'a str {
        &post.content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follows_the_book_workflow() {
        let mut post = Post::new("Lunch");
        post.add_text("I ate a salad for lunch today");
        assert_eq!(post.status(), "draft");
        assert_eq!(post.content(), "");

        post.request_review();
        assert_eq!(post.status(), "pending review");
        assert_eq!(post.content(), "");

        post.approve();
        assert_eq!(post.status(), "published");
        assert_eq!(post.content(), "I ate a salad for lunch today");
    }

    #[test]
    fn invalid_steps_are_ignored_at_run_time() {
        let mut post = Post::new("Skipping ahead");
        post.add_text("text");
        post.approve(); // approving a draft does nothing
        assert_eq!(post.status(), "draft");

        post.request_review();
        post.request_review(); // asking twice changes nothing
        assert_eq!(post.status(), "pending review");
    }

    #[test]
    fn the_title_is_always_visible() {
        let post = Post::new("Hello");
        assert_eq!(post.title(), "Hello");
    }
}
