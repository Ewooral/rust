//! Book 18.3, part 2: encoding states and behaviour as TYPES.
//! Run: cargo run -p trait_objects_and_oop --example blog_state_as_types
//!
//! Instead of one `Post` that changes its hidden state, each state is its own
//! type with only the methods that make sense for it:
//!
//!   DraftPost          add_text, request_review -> PendingReviewPost
//!   PendingReviewPost  approve -> Post
//!   Post               content
//!
//! Asking a draft for its content is no longer "returns empty": it's a
//! compile error, because `DraftPost` has no `content` method at all.

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
        // The only way to start is a draft. (Clippy expects `new` to return
        // Self; returning the first state is the whole point here.)
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

        /// Takes `self` by value: the draft is used up, and you get back a
        /// value of a different type.
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
    let mut post = blog::Post::new(); // a DraftPost

    post.add_text("I ate a salad for lunch today");
    // println!("{}", post.content());
    //   // E0599: no method named `content` found for struct `DraftPost`

    // Each transition returns a NEW type, so we shadow `post` with it.
    let post = post.request_review(); // a PendingReviewPost
    // post.add_text("more");
    //   // E0599: no method named `add_text` found for struct `PendingReviewPost`

    let post = post.approve(); // a Post
    println!("published: {:?}", post.content());

    // Trade-offs compared with the trait-object version:
    // + invalid steps are compile errors, and there's no `Option`/`take()`.
    // + no `Box`, no vtable: plain structs moving from one type to the next.
    // - the caller must re-bind (`let post = ...`) after each step.
    // - you can't keep posts in DIFFERENT states in one Vec<...> without an
    //   enum or a trait object again.
}
