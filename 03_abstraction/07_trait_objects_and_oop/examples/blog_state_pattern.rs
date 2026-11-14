//! Book 18.3, part 1: the STATE PATTERN with trait objects.
//! Run: cargo run -p trait_objects_and_oop --example blog_state_pattern
//!
//! A blog post goes Draft -> PendingReview -> Published. Only a published
//! post shows its content. Each state is its own type implementing `State`,
//! and the post holds the current one as a `Box<dyn State>`. The post itself
//! never asks "which state am I in?": it just forwards each request.

mod blog {
    pub struct Post {
        // Option so a method can `take()` the old state out, leaving None
        // for a moment, and put the new state back.
        state: Option<Box<dyn State>>,
        content: String,
    }

    impl Post {
        pub fn new() -> Post {
            Post {
                state: Some(Box::new(Draft {})),
                content: String::new(),
            }
        }

        pub fn add_text(&mut self, text: &str) {
            self.content.push_str(text);
        }

        /// Asks the current state; only Published returns the real text.
        pub fn content(&self) -> &str {
            self.state.as_ref().unwrap().content(self)
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
    }

    trait State {
        // `self: Box<Self>`: the method TAKES OWNERSHIP of the boxed state,
        // so the old state is used up and can't be used again. (A plain
        // `self` can't be called on a `Box<dyn State>`: E0161, the size of
        // `dyn State` cannot be statically determined.)
        fn request_review(self: Box<Self>) -> Box<dyn State>;
        fn approve(self: Box<Self>) -> Box<dyn State>;

        // A default: most states hide the content.
        fn content<'a>(&self, _post: &'a Post) -> &'a str {
            ""
        }
    }

    struct Draft {}

    impl State for Draft {
        fn request_review(self: Box<Self>) -> Box<dyn State> {
            Box::new(PendingReview {})
        }
        fn approve(self: Box<Self>) -> Box<dyn State> {
            self // can't approve a draft: stay as we are
        }
    }

    struct PendingReview {}

    impl State for PendingReview {
        fn request_review(self: Box<Self>) -> Box<dyn State> {
            self
        }
        fn approve(self: Box<Self>) -> Box<dyn State> {
            Box::new(Published {})
        }
    }

    struct Published {}

    impl State for Published {
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
}

fn main() {
    let mut post = blog::Post::new();

    post.add_text("I ate a salad for lunch today");
    println!("draft:     {:?}", post.content());

    post.request_review();
    println!("in review: {:?}", post.content());

    post.approve();
    println!("published: {:?}", post.content());

    // The downsides the Book points out:
    // - States know about each other (Draft creates PendingReview), so adding
    //   a state between two others means editing both.
    // - request_review/approve are repeated in every state, and so are the
    //   `take()` dances in Post.
    // - Nothing stops you calling `content()` on a draft: you just get "".
    //   The next example turns that mistake into a COMPILE error.
}
