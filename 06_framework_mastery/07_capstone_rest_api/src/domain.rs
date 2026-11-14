//! The domain: what users and reviews ARE, the rules their fields follow, and
//! pagination. No HTTP and no SQL here, so all of it is tested with plain `#[test]`s.

use serde::{Deserialize, Serialize};

use crate::error::FieldError;

/// What a user may do (see `service::policy`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Writes reviews, and edits or deletes their own.
    Member,
    /// May also edit or delete anyone's reviews.
    Admin,
}

impl Role {
    /// The text stored in the `role` column.
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Member => "member",
            Role::Admin => "admin",
        }
    }

    /// Reads the `role` column back.
    pub fn from_db(text: &str) -> Option<Role> {
        match text {
            "member" => Some(Role::Member),
            "admin" => Some(Role::Admin),
            _ => None,
        }
    }
}

/// A user as the API shows it. There is deliberately no password hash in
/// this type, so it can never be serialised into a response by mistake.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    /// The user's id.
    pub id: i64,
    /// Lower-case e-mail address.
    pub email: String,
    /// The name shown next to their reviews.
    pub display_name: String,
    /// Member or admin.
    pub role: Role,
    /// When they registered (UTC, ISO 8601).
    pub created_at: String,
}

/// A book review, with its author's name joined in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Review {
    /// The review's id.
    pub id: i64,
    /// Who wrote it.
    pub author_id: i64,
    /// Their display name.
    pub author_name: String,
    /// The book.
    pub book_title: String,
    /// 1 to 5 stars.
    pub rating: i64,
    /// The text (may be empty).
    pub body: String,
    /// When it was written.
    pub created_at: String,
    /// When it last changed.
    pub updated_at: String,
}

/// `POST /api/v1/auth/register`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisterRequest {
    /// Becomes the login name.
    pub email: String,
    /// 12 to 128 characters.
    pub password: String,
    /// 1 to 50 characters.
    pub display_name: String,
}

/// `POST /api/v1/auth/login`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    /// The registered e-mail.
    pub email: String,
    /// The password.
    pub password: String,
}

/// The answer to a successful login.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenResponse {
    /// The JWT to send as `Authorization: Bearer <token>`.
    pub access_token: String,
    /// Always `"Bearer"`.
    pub token_type: String,
    /// Seconds until it expires.
    pub expires_in: u64,
}

/// `POST /api/v1/reviews`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewReview {
    /// 1 to 200 characters.
    pub book_title: String,
    /// 1 to 5.
    pub rating: i64,
    /// Up to 5,000 characters.
    #[serde(default)]
    pub body: String,
}

/// `PATCH /api/v1/reviews/{id}`: only the fields that are present change.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewPatch {
    /// A new title.
    pub book_title: Option<String>,
    /// A new rating.
    pub rating: Option<i64>,
    /// New text.
    pub body: Option<String>,
}

/// Limits the validation enforces.
pub mod limits {
    /// Shortest password.
    pub const PASSWORD_MIN: usize = 12;
    /// Longest password (Argon2 is slow on purpose; don't hash megabytes).
    pub const PASSWORD_MAX: usize = 128;
    /// Longest display name.
    pub const NAME_MAX: usize = 50;
    /// Longest book title.
    pub const TITLE_MAX: usize = 200;
    /// Longest review text.
    pub const BODY_MAX: usize = 5_000;
}

fn check_email(email: &str, problems: &mut Vec<FieldError>) {
    // Deliberately simple: one `@`, something before it, a dot after it.
    // The only real test of an e-mail address is sending it a message.
    let ok = match email.split_once('@') {
        Some((local, domain)) => {
            !local.is_empty()
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
                && !email.contains(char::is_whitespace)
                && email.matches('@').count() == 1
        }
        None => false,
    };
    if !ok || email.len() > 254 {
        problems.push(FieldError::new("email", "must be an e-mail address"));
    }
}

fn check_len(
    field: &'static str,
    value: &str,
    min: usize,
    max: usize,
    problems: &mut Vec<FieldError>,
) {
    let len = value.chars().count();
    if len < min || len > max {
        let message = if min == 0 {
            format!("must be at most {max} characters")
        } else {
            format!("must be {min} to {max} characters")
        };
        problems.push(FieldError::new(field, message));
    }
}

fn check_rating(rating: i64, problems: &mut Vec<FieldError>) {
    if !(1..=5).contains(&rating) {
        problems.push(FieldError::new("rating", "must be from 1 to 5"));
    }
}

impl RegisterRequest {
    /// Checks every field, reporting ALL problems; trims and lower-cases.
    pub fn validate(self) -> Result<RegisterRequest, Vec<FieldError>> {
        let email = self.email.trim().to_lowercase();
        let display_name = self.display_name.trim().to_string();
        let mut problems = Vec::new();
        check_email(&email, &mut problems);
        check_len(
            "password",
            &self.password,
            limits::PASSWORD_MIN,
            limits::PASSWORD_MAX,
            &mut problems,
        );
        check_len(
            "display_name",
            &display_name,
            1,
            limits::NAME_MAX,
            &mut problems,
        );
        if problems.is_empty() {
            Ok(RegisterRequest {
                email,
                display_name,
                password: self.password,
            })
        } else {
            Err(problems)
        }
    }
}

impl NewReview {
    /// Checks every field, reporting ALL problems; trims the title.
    pub fn validate(self) -> Result<NewReview, Vec<FieldError>> {
        let book_title = self.book_title.trim().to_string();
        let mut problems = Vec::new();
        check_len(
            "book_title",
            &book_title,
            1,
            limits::TITLE_MAX,
            &mut problems,
        );
        check_rating(self.rating, &mut problems);
        check_len("body", &self.body, 0, limits::BODY_MAX, &mut problems);
        if problems.is_empty() {
            Ok(NewReview { book_title, ..self })
        } else {
            Err(problems)
        }
    }
}

impl ReviewPatch {
    /// The same rules as [`NewReview::validate`], for the fields present.
    pub fn validate(self) -> Result<ReviewPatch, Vec<FieldError>> {
        let book_title = self.book_title.map(|t| t.trim().to_string());
        let mut problems = Vec::new();
        if let Some(title) = &book_title {
            check_len("book_title", title, 1, limits::TITLE_MAX, &mut problems);
        }
        if let Some(rating) = self.rating {
            check_rating(rating, &mut problems);
        }
        if let Some(body) = &self.body {
            check_len("body", body, 0, limits::BODY_MAX, &mut problems);
        }
        if problems.is_empty() {
            Ok(ReviewPatch { book_title, ..self })
        } else {
            Err(problems)
        }
    }
}

/// Query string for `GET /api/v1/reviews?page=2&per_page=10&book=emma&min_rating=4`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListParams {
    /// 1-based page number (default 1).
    pub page: Option<u32>,
    /// Items per page (default 20, at most 100).
    pub per_page: Option<u32>,
    /// Only reviews whose book title contains this text.
    pub book: Option<String>,
    /// Only reviews with at least this many stars.
    pub min_rating: Option<i64>,
}

/// Default page size.
pub const DEFAULT_PER_PAGE: u32 = 20;
/// Largest page size a client may ask for.
pub const MAX_PER_PAGE: u32 = 100;

impl ListParams {
    /// The page and page size to use: defaults filled in, nonsense clamped.
    pub fn page_and_size(&self) -> (u32, u32) {
        let page = self.page.unwrap_or(1).max(1);
        let per_page = self
            .per_page
            .unwrap_or(DEFAULT_PER_PAGE)
            .clamp(1, MAX_PER_PAGE);
        (page, per_page)
    }

    /// Rows to skip for this page (`OFFSET`).
    pub fn offset(&self) -> u64 {
        let (page, per_page) = self.page_and_size();
        u64::from(page - 1) * u64::from(per_page)
    }
}

/// One page of results, and enough to find the others.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page<T> {
    /// This page's items.
    pub items: Vec<T>,
    /// This page's number (from 1).
    pub page: u32,
    /// The page size used.
    pub per_page: u32,
    /// Matching items on all pages together.
    pub total: u64,
    /// How many pages there are (0 when there are no items).
    pub total_pages: u64,
}

impl<T> Page<T> {
    /// Wraps one page of `items` out of `total`.
    pub fn new(items: Vec<T>, page: u32, per_page: u32, total: u64) -> Page<T> {
        Page {
            items,
            page,
            per_page,
            total,
            total_pages: total.div_ceil(u64::from(per_page.max(1))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn register(email: &str, password: &str, name: &str) -> RegisterRequest {
        RegisterRequest {
            email: email.into(),
            password: password.into(),
            display_name: name.into(),
        }
    }

    #[test]
    fn registration_is_cleaned_up() {
        let ok = register("  Ama@Example.COM ", "correct horse battery", " Ama ")
            .validate()
            .unwrap();
        assert_eq!(
            (ok.email.as_str(), ok.display_name.as_str()),
            ("ama@example.com", "Ama")
        );
    }

    #[test]
    fn registration_reports_every_problem() {
        let problems = register("not-an-email", "short", "")
            .validate()
            .unwrap_err();
        let fields: Vec<&str> = problems.iter().map(|p| p.field).collect();
        assert_eq!(fields, ["email", "password", "display_name"]);
        for bad in [
            "a@b",
            "@example.com",
            "a@@example.com",
            "a b@example.com",
            "a@example.",
        ] {
            assert!(
                register(bad, "correct horse battery", "A")
                    .validate()
                    .is_err(),
                "{bad}"
            );
        }
    }

    #[test]
    fn reviews_and_patches_are_checked() {
        let review = NewReview {
            book_title: "  Emma ".into(),
            rating: 5,
            body: String::new(),
        };
        assert_eq!(review.validate().unwrap().book_title, "Emma");
        let bad = NewReview {
            book_title: " ".into(),
            rating: 6,
            body: "x".repeat(limits::BODY_MAX + 1),
        };
        assert_eq!(bad.validate().unwrap_err().len(), 3);
        let patch = ReviewPatch {
            rating: Some(0),
            ..ReviewPatch::default()
        };
        assert_eq!(patch.validate().unwrap_err()[0].field, "rating");
        assert!(ReviewPatch::default().validate().is_ok());
    }

    #[test]
    fn page_parameters_are_clamped() {
        let params = |page, per_page| ListParams {
            page,
            per_page,
            ..ListParams::default()
        };
        assert_eq!(params(None, None).page_and_size(), (1, 20));
        assert_eq!(params(Some(0), Some(0)).page_and_size(), (1, 1));
        assert_eq!(params(Some(3), Some(500)).page_and_size(), (3, 100));
        assert_eq!(params(Some(3), Some(10)).offset(), 20);
    }

    #[test]
    fn pages_know_how_many_there_are() {
        assert_eq!(Page::new(vec![1, 2], 1, 2, 5).total_pages, 3);
        assert_eq!(Page::new(Vec::<i32>::new(), 1, 20, 0).total_pages, 0);
        assert_eq!(Page::new(vec![1], 2, 20, 40).total_pages, 2);
    }
}
