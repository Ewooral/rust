//! `crate::library::books`.

pub struct Book {
    title: String,
    pages: u32,
}

impl Book {
    pub fn new(title: &str, pages: u32) -> Self {
        Self {
            title: title.to_string(),
            pages,
        }
    }

    pub fn summary(&self) -> String {
        format!("{} ({} pages)", self.title, self.pages)
    }
}
