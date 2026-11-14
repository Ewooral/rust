//! The Rust types that rows turn into, and the input types that become rows.

use crate::error::InventoryError;
use crate::isbn::Isbn;
use crate::money::Pence;

/// A book on the shelf, with its author's name joined in.
///
/// `#[derive(sqlx::FromRow)]` writes the code that copies each column of a
/// result row into the field with the same name. The query must return
/// every field's column (extra columns are ignored).
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Book {
    /// The database's id for the row.
    pub id: i64,
    /// Decoded through `Isbn`'s `#[sqlx(transparent)]` impl.
    pub isbn: Isbn,
    /// The title.
    pub title: String,
    /// The author's name, from a `JOIN` with `authors`.
    pub author: String,
    /// The column is called `price_pence`; the field is called `price`.
    #[sqlx(rename = "price_pence")]
    pub price: Pence,
    /// Copies on the shelf.
    pub stock: i64,
    /// `NULL` in the database becomes `None` (migration 3 added this column).
    pub published_year: Option<i64>,
}

/// A book that isn't in the database yet. Its fields are already validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewBook {
    /// A checked ISBN.
    pub isbn: Isbn,
    /// A non-empty title.
    pub title: String,
    /// A non-empty author name.
    pub author: String,
    /// The price per copy.
    pub price: Pence,
    /// Copies to start with (zero or more).
    pub stock: i64,
    /// Optional year of publication.
    pub published_year: Option<i64>,
}

impl NewBook {
    /// Validates raw text (from a form, a CSV file, a command line…) into a
    /// `NewBook`, so bad data is refused before any SQL runs.
    pub fn parse(
        isbn: &str,
        title: &str,
        author: &str,
        price: &str,
        stock: i64,
    ) -> Result<NewBook, InventoryError> {
        let title = title.trim();
        let author = author.trim();
        if title.is_empty() {
            return Err(InventoryError::Invalid("the title is empty".into()));
        }
        if author.is_empty() {
            return Err(InventoryError::Invalid("the author is empty".into()));
        }
        if stock < 0 {
            return Err(InventoryError::Invalid(format!(
                "stock can't be negative ({stock})"
            )));
        }
        Ok(NewBook {
            isbn: Isbn::parse(isbn)?,
            title: title.to_string(),
            author: author.to_string(),
            price: price.parse()?,
            stock,
            published_year: None,
        })
    }

    /// Sets the year of publication (builder style).
    pub fn published(mut self, year: i64) -> NewBook {
        self.published_year = Some(year);
        self
    }
}

/// The receipt for a sale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sale {
    /// The new row in `sales`.
    pub sale_id: i64,
    /// What was sold.
    pub isbn: Isbn,
    /// How many copies.
    pub quantity: i64,
    /// Price × quantity.
    pub total: Pence,
    /// Copies left on the shelf afterwards.
    pub stock_left: i64,
}

/// One line of the sales report: an aggregate query mapped with `FromRow`.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct SalesLine {
    /// The book's title.
    pub title: String,
    /// `SUM(quantity)`.
    pub copies_sold: i64,
    /// `SUM(total_pence)`.
    #[sqlx(rename = "revenue_pence")]
    pub revenue: Pence,
}

/// Optional search filters, combined with `AND`, plus a page.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BookFilter {
    /// Part of the title, any case.
    pub title_contains: Option<String>,
    /// The author's exact name.
    pub author: Option<String>,
    /// The highest price to include.
    pub max_price: Option<Pence>,
    /// Leave out books with no copies.
    pub in_stock_only: bool,
    /// Page size; `None` means 50.
    pub limit: Option<i64>,
    /// Rows to skip (for page 2, 3, …).
    pub offset: i64,
}
