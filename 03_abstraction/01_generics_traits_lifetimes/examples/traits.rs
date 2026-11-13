//! Book 10.2: defining shared behaviour with traits.
//! Run: cargo run -p generics_traits_lifetimes --example traits

use std::fmt::{self, Display};

// ---- Defining a trait ----
pub trait Summary {
    // Required: every implementor must write this.
    fn summarize_author(&self) -> String;

    // Default implementation: implementors get it for free, or override it.
    // A default method may call other trait methods, even required ones.
    fn summarize(&self) -> String {
        format!("(Read more from {}...)", self.summarize_author())
    }
}

pub struct NewsArticle {
    pub headline: String,
    pub location: String,
    pub author: String,
}

pub struct SocialPost {
    pub username: String,
    pub content: String,
    pub reply: bool,
    pub repost: bool,
}

// ---- Implementing a trait on a type ----
impl Summary for NewsArticle {
    fn summarize_author(&self) -> String {
        self.author.clone()
    }

    // Override the default.
    fn summarize(&self) -> String {
        format!("{}, by {} ({})", self.headline, self.author, self.location)
    }
}

impl Summary for SocialPost {
    fn summarize_author(&self) -> String {
        format!("@{}", self.username)
    }
    // summarize() not written: the default is used.
}

// ---- Traits as parameters: three ways to write the same thing ----

// 1. `impl Trait`: short and sweet.
fn notify(item: &impl Summary) {
    println!("Breaking news! {}", item.summarize());
}

// 2. A trait bound on a generic. Needed when two parameters must be the SAME type.
fn notify_pair<T: Summary>(item1: &T, item2: &T) {
    println!("Pair: {} | {}", item1.summarize(), item2.summarize());
}

// 3. Several bounds with `+`, written in a `where` clause when they get long.
fn notify_and_show<T>(item: &T) -> String
where
    T: Summary + Display,
{
    format!("{item} :: {}", item.summarize())
}

impl Display for SocialPost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "post({}{}{})",
            self.content,
            if self.reply { ", reply" } else { "" },
            if self.repost { ", repost" } else { "" }
        )
    }
}

// ---- Returning a type that implements a trait ----
// The caller only knows "some type that implements Summary". It must be ONE
// concrete type: returning a NewsArticle on one path and a SocialPost on
// another does not compile with `impl Trait` (see the dispatch example).
fn returns_summarizable() -> impl Summary {
    SocialPost {
        username: String::from("horse_ebooks"),
        content: String::from("of course, as you probably already know, people"),
        reply: false,
        repost: false,
    }
}

// ---- Conditionally implementing methods ----
struct Pair<T> {
    x: T,
    y: T,
}

impl<T> Pair<T> {
    fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

// Only pairs whose T can be displayed AND compared get this method.
impl<T: Display + PartialOrd> Pair<T> {
    fn cmp_display(&self) {
        if self.x >= self.y {
            println!("The largest member is x = {}", self.x);
        } else {
            println!("The largest member is y = {}", self.y);
        }
    }
}

// ---- Blanket implementations ----
// The standard library contains, roughly:
//     impl<T: Display> ToString for T { ... }
// so EVERY type that implements Display gets `.to_string()` for free.
// We can write our own blanket impl over our own trait:
trait Shout {
    fn shout(&self) -> String;
}
impl<T: Display> Shout for T {
    fn shout(&self) -> String {
        self.to_string().to_uppercase() + "!"
    }
}

fn main() {
    let article = NewsArticle {
        headline: String::from("Penguins win the Stanley Cup Championship!"),
        location: String::from("Pittsburgh, PA, USA"),
        author: String::from("Iceburgh"),
    };
    let post = SocialPost {
        username: String::from("horse_ebooks"),
        content: String::from("of course"),
        reply: false,
        repost: true,
    };

    println!("1 new post: {}", post.summarize());
    notify(&article);
    notify(&post);
    notify_pair(&post, &post);
    // notify_pair(&post, &article); // E0308: both must be the same type T
    println!("{}", notify_and_show(&post));
    println!("{}", returns_summarizable().summarize());

    Pair::new(3, 7).cmp_display();
    Pair::new("b", "a").cmp_display();

    println!("{}", 42.shout());
    println!("{}", "hello".shout());
}
