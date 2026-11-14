//! Solution to exercise 4: pagination arithmetic, the part every list endpoint needs.
//!
//! A client asks for `?page=3&per_page=10`. Implement:
//!
//! 1. `PageRequest::normalised`: fill in defaults and clamp nonsense.
//!
//!    | field      | missing | 0 | more than 100 |
//!    |------------|---------|---|---------------|
//!    | `page`     | 1       | 1 | (kept)        |
//!    | `per_page` | 20      | 1 | 100           |
//!
//! 2. `PageRequest::offset`: rows to skip (SQL `OFFSET`) for the normalised
//!    page. Page 1 skips 0. It's a `u64`, so even page 4,000,000,000 at 100
//!    a page can't overflow.
//! 3. `Page::new`: wrap the items and work out `total_pages` (0 when there
//!    are no items; a part-filled last page still counts).
//! 4. `Page::has_next`: is there a page after this one?
//!
//! Test: cargo test -p capstone_rest_api --features solutions --bin sol04_pagination

const DEFAULT_PER_PAGE: u32 = 20;
const MAX_PER_PAGE: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct PageRequest {
    page: Option<u32>,
    per_page: Option<u32>,
}

impl PageRequest {
    /// (page, per_page) after defaults and clamping.
    fn normalised(&self) -> (u32, u32) {
        let page = self.page.unwrap_or(1).max(1);
        let per_page = self
            .per_page
            .unwrap_or(DEFAULT_PER_PAGE)
            .clamp(1, MAX_PER_PAGE);
        (page, per_page)
    }

    /// Rows to skip for the normalised page.
    fn offset(&self) -> u64 {
        let (page, per_page) = self.normalised();
        // Widen BEFORE multiplying: u32 × u32 can overflow a u32.
        u64::from(page - 1) * u64::from(per_page)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Page<T> {
    items: Vec<T>,
    page: u32,
    per_page: u32,
    total: u64,
    total_pages: u64,
}

impl<T> Page<T> {
    fn new(items: Vec<T>, page: u32, per_page: u32, total: u64) -> Page<T> {
        Page {
            items,
            page,
            per_page,
            total,
            // Round up: 57 items at 10 a page is 6 pages.
            total_pages: total.div_ceil(u64::from(per_page.max(1))),
        }
    }

    fn has_next(&self) -> bool {
        u64::from(self.page) < self.total_pages
    }
}

fn main() {
    let request = PageRequest {
        page: Some(3),
        per_page: Some(10),
    };
    let (page, per_page) = request.normalised();
    println!(
        "page {page}, {per_page} per page, OFFSET {}",
        request.offset()
    );
    let page = Page::new(vec!["a review"; 10], page, per_page, 57);
    println!(
        "{} pages in total, another after this one: {}",
        page.total_pages,
        page.has_next()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(page: Option<u32>, per_page: Option<u32>) -> PageRequest {
        PageRequest { page, per_page }
    }

    #[test]
    fn defaults_and_clamping() {
        assert_eq!(request(None, None).normalised(), (1, 20));
        assert_eq!(request(Some(0), Some(0)).normalised(), (1, 1));
        assert_eq!(request(Some(7), Some(500)).normalised(), (7, 100));
        assert_eq!(request(Some(2), Some(50)).normalised(), (2, 50));
    }

    #[test]
    fn offsets() {
        assert_eq!(request(None, None).offset(), 0);
        assert_eq!(request(Some(3), Some(10)).offset(), 20);
        assert_eq!(request(Some(0), Some(10)).offset(), 0);
        assert_eq!(
            request(Some(u32::MAX), Some(100)).offset(),
            (u64::from(u32::MAX) - 1) * 100
        );
    }

    #[test]
    fn total_pages() {
        assert_eq!(Page::new(vec![1; 10], 1, 10, 57).total_pages, 6);
        assert_eq!(Page::new(vec![1; 10], 1, 10, 50).total_pages, 5);
        assert_eq!(Page::new(Vec::<i32>::new(), 1, 10, 0).total_pages, 0);
        assert_eq!(Page::new(vec![1], 1, 1, 1).total_pages, 1);
    }

    #[test]
    fn next_pages() {
        assert!(Page::new(vec![1; 10], 5, 10, 57).has_next());
        assert!(!Page::new(vec![1; 7], 6, 10, 57).has_next());
        assert!(!Page::new(Vec::<i32>::new(), 1, 10, 0).has_next());
    }
}
