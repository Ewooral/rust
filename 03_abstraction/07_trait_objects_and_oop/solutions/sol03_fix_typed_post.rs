//! Solution to exercise 3: make it compile (E0599).
//!
//! The typed blog post from Book 18.3: each step takes the post BY VALUE and
//! returns a value of a NEW type. `main` calls the steps but throws away what
//! they return, so as far as the compiler knows, `post` is still a
//! `DraftPost`, which has no `approve` and no `content`.
//! Fix `main` by keeping each new value (shadow `post` with `let`).
//! Don't add or change any methods in `blog`.
//!
//! Run: cargo run -p trait_objects_and_oop --features solutions --bin sol03_fix_typed_post

mod blog {
    pub struct Post {
        content: String,
    }

    pub struct DraftPost {
        content: String,
    }

    pub struct PendingReviewPost {
        content: String,
    }

    impl Post {
        #[allow(clippy::new_ret_no_self)]
        pub fn new() -> DraftPost {
            DraftPost {
                content: String::new(),
            }
        }

        pub fn content(&self) -> &str {
            &self.content
        }
    }

    impl DraftPost {
        pub fn add_text(&mut self, text: &str) {
            self.content.push_str(text);
        }

        pub fn request_review(self) -> PendingReviewPost {
            PendingReviewPost {
                content: self.content,
            }
        }
    }

    impl PendingReviewPost {
        pub fn approve(self) -> Post {
            Post {
                content: self.content,
            }
        }
    }
}

fn main() {
    let mut post = blog::Post::new();
    post.add_text("I ate a salad for lunch today");

    // Each step uses up the old value and returns a new type. Shadowing
    // keeps the name `post` while its TYPE moves along:
    // DraftPost -> PendingReviewPost -> Post.
    let post = post.request_review();
    let post = post.approve();

    println!("published: {}", post.content());
}
