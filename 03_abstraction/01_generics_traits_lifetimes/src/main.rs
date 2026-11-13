//! Mini project for Book chapter 10: a news feed, built from the Book's
//! `Summary` trait and extended with every tool of the chapter.
//!
//! - TRAITS: `Summary` (with a default method) and `Scored`, implemented by
//!   three different item types.
//! - GENERICS: `top_n` ranks ANY list of `Scored` items; `Ranked<T>` wraps any
//!   item and gets `Display` only if the item is a `Summary` (a conditional impl).
//! - DYN: `Feed` holds a MIX of item types as `Box<dyn FeedItem>`.
//! - LIFETIMES: `Highlight<'a>` borrows a quote straight out of an article's
//!   text, and `top_n` returns references into the caller's slice.
//!
//! Run: cargo run -p generics_traits_lifetimes
//! Test: cargo test -p generics_traits_lifetimes

use std::fmt;

// ---------------- traits ----------------

trait Summary {
    fn summarize_author(&self) -> String;

    fn summarize(&self) -> String {
        format!("(Read more from {}...)", self.summarize_author())
    }
}

trait Scored {
    fn score(&self) -> u32;
}

/// Blanket impls for references: if T is Summary, so is &T. Now a borrowed
/// item can be used anywhere an owned one can (e.g. `Ranked<&NewsArticle>`).
impl<T: Summary + ?Sized> Summary for &T {
    fn summarize_author(&self) -> String {
        (**self).summarize_author()
    }
    fn summarize(&self) -> String {
        (**self).summarize()
    }
}
impl<T: Scored + ?Sized> Scored for &T {
    fn score(&self) -> u32 {
        (**self).score()
    }
}

/// A supertrait: a FeedItem is anything that is both Summary and Scored.
trait FeedItem: Summary + Scored {}
/// A blanket impl: every type that is Summary + Scored is a FeedItem
/// automatically; no type has to opt in by hand.
impl<T: Summary + Scored> FeedItem for T {}

// ---------------- three item types ----------------

struct NewsArticle {
    headline: String,
    author: String,
    text: String,
    reads: u32,
}

struct SocialPost {
    username: String,
    content: String,
    likes: u32,
    reposts: u32,
}

struct Video {
    title: String,
    channel: String,
    minutes: u32,
    views: u32,
}

impl Summary for NewsArticle {
    fn summarize_author(&self) -> String {
        self.author.clone()
    }
    fn summarize(&self) -> String {
        format!("{}, by {}", self.headline, self.author)
    }
}

impl Summary for SocialPost {
    fn summarize_author(&self) -> String {
        format!("@{}", self.username)
    }
    // uses the default summarize()
}

impl Summary for Video {
    fn summarize_author(&self) -> String {
        self.channel.clone()
    }
    fn summarize(&self) -> String {
        format!("{} ({} min) on {}", self.title, self.minutes, self.channel)
    }
}

impl Scored for NewsArticle {
    fn score(&self) -> u32 {
        self.reads
    }
}
impl Scored for SocialPost {
    fn score(&self) -> u32 {
        self.likes + 3 * self.reposts // a repost counts more than a like
    }
}
impl Scored for Video {
    fn score(&self) -> u32 {
        self.views / 10
    }
}

// ---------------- generics ----------------

/// The `n` highest-scoring items, best first. Generic over any `Scored` type,
/// and the references it returns borrow from `items` (lifetime elision:
/// one input reference, so the output gets its lifetime).
fn top_n<T: Scored>(items: &[T], n: usize) -> Vec<&T> {
    let mut ranked: Vec<&T> = items.iter().collect();
    ranked.sort_by_key(|item| std::cmp::Reverse(item.score()));
    ranked.truncate(n);
    ranked
}

/// Wraps any item with its position in a ranking.
struct Ranked<T> {
    position: usize,
    item: T,
}

/// A conditional impl: `Ranked<T>` can be displayed only when T is both
/// Summary and Scored. `Ranked<i32>` exists, but cannot be printed.
impl<T: Summary + Scored> fmt::Display for Ranked<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{} [{}] {}",
            self.position,
            self.item.score(),
            self.item.summarize()
        )
    }
}

// ---------------- dynamic dispatch ----------------

/// A feed of mixed item types: generics can't do this, trait objects can.
#[derive(Default)]
struct Feed {
    items: Vec<Box<dyn FeedItem>>,
}

impl Feed {
    fn add(&mut self, item: impl FeedItem + 'static) {
        self.items.push(Box::new(item));
    }

    /// Item summaries, highest score first.
    fn ranked(&self) -> Vec<String> {
        let mut items: Vec<&dyn FeedItem> = self.items.iter().map(|b| b.as_ref()).collect();
        items.sort_by_key(|item| std::cmp::Reverse(item.score()));
        items
            .iter()
            .map(|item| format!("[{}] {}", item.score(), item.summarize()))
            .collect()
    }
}

// ---------------- lifetimes ----------------

/// A quote borrowed from an article. It cannot outlive the article.
struct Highlight<'a> {
    quote: &'a str,
    author: &'a str,
}

impl NewsArticle {
    /// The first sentence of the text, without copying it.
    fn highlight(&self) -> Highlight<'_> {
        let quote = self.text.split_inclusive('.').next().unwrap_or("").trim();
        Highlight {
            quote,
            author: &self.author,
        }
    }
}

impl fmt::Display for Highlight<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\" ({})", self.quote, self.author)
    }
}

fn main() {
    let articles = vec![
        NewsArticle {
            headline: "Penguins win the Stanley Cup".into(),
            author: "Iceburgh".into(),
            text: "The Penguins won again. Fans celebrated all night.".into(),
            reads: 1_200,
        },
        NewsArticle {
            headline: "Rust 2024 edition released".into(),
            author: "Ferris".into(),
            text: "A new edition is here. Upgrading is easy.".into(),
            reads: 4_800,
        },
        NewsArticle {
            headline: "Local bakery wins award".into(),
            author: "Ama".into(),
            text: "The bread really is that good. Queue early.".into(),
            reads: 300,
        },
    ];

    println!("top 2 articles (generic top_n):");
    for (i, article) in top_n(&articles, 2).into_iter().enumerate() {
        println!(
            "  {}",
            Ranked {
                position: i + 1,
                item: article // a &NewsArticle: Summary + Scored via the blanket impls
            }
        );
        println!("     highlight: {}", article.highlight());
    }

    let mut feed = Feed::default();
    let post = SocialPost {
        username: "horse_ebooks".into(),
        content: "of course, as you probably already know".into(),
        likes: 900,
        reposts: 200,
    };
    println!(
        "
new post: {:?}",
        post.content
    );
    feed.add(post);
    feed.add(Video {
        title: "Lifetimes in 10 minutes".into(),
        channel: "Let's Get Rusty".into(),
        minutes: 10,
        views: 25_000,
    });
    for article in articles {
        feed.add(article); // moved into the feed
    }

    println!("\nthe whole feed, mixed types (dyn FeedItem):");
    for line in feed.ranked() {
        println!("  {line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn post(likes: u32, reposts: u32) -> SocialPost {
        SocialPost {
            username: "u".into(),
            content: "c".into(),
            likes,
            reposts,
        }
    }

    #[test]
    fn default_and_overridden_summaries() {
        assert_eq!(post(0, 0).summarize(), "(Read more from @u...)");
        let v = Video {
            title: "T".into(),
            channel: "C".into(),
            minutes: 3,
            views: 0,
        };
        assert_eq!(v.summarize(), "T (3 min) on C");
    }

    #[test]
    fn top_n_is_generic_and_borrows() {
        let posts = [post(1, 0), post(10, 0), post(0, 5)];
        let best = top_n(&posts, 2);
        assert_eq!(best.len(), 2);
        assert_eq!(best[0].score(), 15);
        assert_eq!(best[1].score(), 10);
        // The result points INTO `posts`: no copies were made.
        assert!(std::ptr::eq(best[0], &posts[2]));
    }

    #[test]
    fn ranked_display_uses_both_traits() {
        let r = Ranked {
            position: 1,
            item: post(2, 1),
        };
        assert_eq!(r.to_string(), "#1 [5] (Read more from @u...)");
    }

    #[test]
    fn feed_mixes_types_and_ranks_them() {
        let mut feed = Feed::default();
        feed.add(post(1, 0));
        feed.add(Video {
            title: "V".into(),
            channel: "C".into(),
            minutes: 1,
            views: 100,
        });
        assert_eq!(
            feed.ranked(),
            ["[10] V (1 min) on C", "[1] (Read more from @u...)"]
        );
    }

    #[test]
    fn highlight_borrows_the_first_sentence() {
        let a = NewsArticle {
            headline: "H".into(),
            author: "A".into(),
            text: "First one. Second one.".into(),
            reads: 0,
        };
        let h = a.highlight();
        assert_eq!(h.quote, "First one.");
        assert_eq!(h.to_string(), "\"First one.\" (A)");
    }
}
