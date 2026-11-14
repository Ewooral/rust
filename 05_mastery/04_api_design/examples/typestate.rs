//! Typestate: the state of a value lives in its TYPE, so calling a method
//! in the wrong state is a compile error, not a run-time check.
//!
//! Run: cargo run -p api_design --example typestate

use std::marker::PhantomData;

use api_design::{Method, Request};

// States are empty types: they exist only for the compiler.
struct Draft;
struct InReview;
struct Published;

/// A blog post that must be reviewed before it's published.
struct Post<State> {
    text: String,
    approvals: u8,
    state: PhantomData<State>, // zero bytes: "this post is in `State`"
}

impl Post<Draft> {
    fn new(text: &str) -> Self {
        Post {
            text: text.into(),
            approvals: 0,
            state: PhantomData,
        }
    }

    fn edit(&mut self, more: &str) {
        self.text.push_str(more);
    }

    /// Consumes the draft: after this, there's no `Post<Draft>` to edit.
    fn submit(self) -> Post<InReview> {
        Post {
            text: self.text,
            approvals: 0,
            state: PhantomData,
        }
    }
}

impl Post<InReview> {
    fn approve(mut self) -> Self {
        self.approvals += 1;
        self
    }

    /// Two approvals publish the post; fewer send it back as a draft.
    fn decide(self) -> Result<Post<Published>, Post<Draft>> {
        // Two different types, so two different struct literals.
        if self.approvals >= 2 {
            Ok(Post {
                text: self.text,
                approvals: self.approvals,
                state: PhantomData,
            })
        } else {
            Err(Post {
                text: self.text,
                approvals: 0,
                state: PhantomData,
            })
        }
    }
}

impl Post<Published> {
    fn url(&self) -> String {
        let slug: String = self
            .text
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join("-");
        format!("/posts/{}", slug.to_lowercase())
    }
}

// Methods for every state.
impl<State> Post<State> {
    fn words(&self) -> usize {
        self.text.split_whitespace().count()
    }
}

fn main() {
    let mut post = Post::new("Typestate makes mistakes");
    post.edit(" impossible to compile");
    // post.url();
    // ^ E0599: no method named `url` found for struct `Post<Draft>`
    let in_review = post.submit();
    // post.edit("!");
    // ^ E0382: borrow of moved value: `post` (submit consumed the draft)
    match in_review.approve().decide() {
        Ok(published) => println!("published at {}", published.url()),
        Err(draft) => println!("back to draft ({} words)", draft.words()),
    }
    let published = Post::new("Second post well reviewed")
        .submit()
        .approve()
        .approve()
        .decide();
    if let Ok(post) = published {
        println!("published at {} ({} words)", post.url(), post.words());
    }
    println!(
        "a Post<Draft> is {} bytes; its PhantomData is {} bytes",
        std::mem::size_of::<Post<Draft>>(),
        std::mem::size_of::<PhantomData<Draft>>()
    );

    // The library's builder works the same way: `build` only exists once
    // there's a URL.
    //     Request::builder().build();
    //     error[E0599]: no method named `build` found for struct `RequestBuilder<NoUrl>`
    //     = note: the method was found for `RequestBuilder<HasUrl>`
    let request = Request::builder()
        .method(Method::Delete)
        .url("http://example.org/posts/1")
        .build()
        .expect("a valid request");
    println!("\n{request}");
}
