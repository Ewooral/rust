//! SQL for the `reviews` table.

use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::domain::{ListParams, NewReview, Review, ReviewPatch};

/// The columns of a [`Review`], with the author's name joined in. A macro, so
/// `concat!` can extend it at compile time and the SQL stays `&'static str`.
macro_rules! select_reviews {
    () => {
        "SELECT r.id, r.user_id AS author_id, u.display_name AS author_name, \
         r.book_title, r.rating, r.body, r.created_at, r.updated_at \
         FROM reviews r JOIN users u ON u.id = r.user_id"
    };
}

/// Reads and writes reviews.
#[derive(Debug, Clone)]
pub struct ReviewRepository {
    pool: SqlitePool,
}

/// Adds the list filters to a query that already ends in `WHERE 1 = 1`.
/// Used by both the page query and the count, so they always agree.
fn push_filters(query: &mut QueryBuilder<Sqlite>, params: &ListParams) {
    if let Some(book) = params
        .book
        .as_deref()
        .map(str::trim)
        .filter(|b| !b.is_empty())
    {
        query
            .push(" AND r.book_title LIKE ")
            .push_bind(format!("%{book}%"));
    }
    if let Some(min) = params.min_rating {
        query.push(" AND r.rating >= ").push_bind(min);
    }
}

impl ReviewRepository {
    /// A repository on this pool.
    pub fn new(pool: SqlitePool) -> Self {
        ReviewRepository { pool }
    }

    /// Inserts a review and returns it, author name included.
    pub async fn insert(&self, author_id: i64, new: &NewReview) -> Result<Review, sqlx::Error> {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO reviews (user_id, book_title, rating, body) VALUES (?, ?, ?, ?) RETURNING id",
        )
        .bind(author_id)
        .bind(&new.book_title)
        .bind(new.rating)
        .bind(&new.body)
        .fetch_one(&self.pool)
        .await?;
        self.find(id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    /// The review with this id.
    pub async fn find(&self, id: i64) -> Result<Option<Review>, sqlx::Error> {
        sqlx::query_as(concat!(select_reviews!(), " WHERE r.id = ?"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    /// Who wrote this review (`None` if there is no such review).
    pub async fn author_of(&self, id: i64) -> Result<Option<i64>, sqlx::Error> {
        sqlx::query_scalar("SELECT user_id FROM reviews WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    /// One page of reviews, newest first, and how many match in total.
    pub async fn list(&self, params: &ListParams) -> Result<(Vec<Review>, u64), sqlx::Error> {
        let (_, per_page) = params.page_and_size();

        let mut count = QueryBuilder::<Sqlite>::new(
            "SELECT COUNT(*) FROM reviews r JOIN users u ON u.id = r.user_id WHERE 1 = 1",
        );
        push_filters(&mut count, params);
        let total: i64 = count.build_query_scalar().fetch_one(&self.pool).await?;

        let mut page = QueryBuilder::<Sqlite>::new(concat!(select_reviews!(), " WHERE 1 = 1"));
        push_filters(&mut page, params);
        page.push(" ORDER BY r.created_at DESC, r.id DESC LIMIT ")
            .push_bind(i64::from(per_page))
            .push(" OFFSET ")
            .push_bind(i64::try_from(params.offset()).unwrap_or(i64::MAX));
        let items = page
            .build_query_as::<Review>()
            .fetch_all(&self.pool)
            .await?;

        Ok((items, u64::try_from(total).unwrap_or(0)))
    }

    /// Applies a patch; `COALESCE(?, column)` keeps the old value when a
    /// field is absent (bound as NULL). `None` if there is no such review.
    pub async fn update(
        &self,
        id: i64,
        patch: &ReviewPatch,
    ) -> Result<Option<Review>, sqlx::Error> {
        let changed = sqlx::query(
            "UPDATE reviews SET book_title = COALESCE(?, book_title), \
             rating = COALESCE(?, rating), body = COALESCE(?, body), \
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?",
        )
        .bind(patch.book_title.as_deref())
        .bind(patch.rating)
        .bind(patch.body.as_deref())
        .bind(id)
        .execute(&self.pool)
        .await?;
        if changed.rows_affected() == 0 {
            return Ok(None);
        }
        self.find(id).await
    }

    /// Deletes a review; `false` if there was no such review.
    pub async fn delete(&self, id: i64) -> Result<bool, sqlx::Error> {
        let done = sqlx::query("DELETE FROM reviews WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(done.rows_affected() == 1)
    }
}
