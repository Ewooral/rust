//! States encoded as types (Book 18.3, second half), plus a title.
//!
//! DraftPost --request_review()--> PendingReviewPost --approve()--> Post
//!
//! Each transition takes `self` BY VALUE and returns a different type, so the
//! old value is used up and only the methods valid in the new state exist.

/// A published post: the only type with a `content()` method.
pub struct Post {
    title: String,
    content: String,
}

/// A post being written: the only type with `add_text()`.
///
/// A draft has no `content()`, and this doc test proves the compiler
/// rejects asking for it:
///
/// ```compile_fail,E0599
/// use trait_objects_and_oop::blog::typed::Post;
///
/// let draft = Post::new("Lunch");
/// println!("{}", draft.content()); // E0599: no method named `content` found for struct `DraftPost`
/// ```
pub struct DraftPost {
    title: String,
    content: String,
}

/// A post waiting for an editor: it can only be approved.
pub struct PendingReviewPost {
    title: String,
    content: String,
}

impl Post {
    /// Starting a post gives you a DRAFT. (Clippy expects `new` to return
    /// `Self`; returning the first state is the point of this design.)
    #[allow(clippy::new_ret_no_self)]
    pub fn new(title: &str) -> DraftPost {
        DraftPost {
            title: title.to_string(),
            content: String::new(),
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl DraftPost {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn add_text(&mut self, text: &str) {
        self.content.push_str(text);
    }

    pub fn request_review(self) -> PendingReviewPost {
        PendingReviewPost {
            title: self.title,
            content: self.content,
        }
    }
}

impl PendingReviewPost {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn approve(self) -> Post {
        Post {
            title: self.title,
            content: self.content,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_step_returns_the_next_type() {
        let mut post: DraftPost = Post::new("Lunch");
        post.add_text("I ate a salad for lunch today");

        let post: PendingReviewPost = post.request_review();
        assert_eq!(post.title(), "Lunch");

        let post: Post = post.approve();
        assert_eq!(post.content(), "I ate a salad for lunch today");
    }

    // There's no unit test for "a draft has no content": that mistake can't
    // even be written. The `compile_fail` doc test on `DraftPost` checks it.
}
