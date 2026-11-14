//! Reviews: validation, authorisation and pagination around the repository.

use crate::auth::AuthUser;
use crate::domain::{ListParams, NewReview, Page, Review, ReviewPatch};
use crate::error::AppError;
use crate::repository::ReviewRepository;
use crate::service::policy::{Action, authorize};

/// The rules for reviews.
#[derive(Debug, Clone)]
pub struct ReviewService {
    reviews: ReviewRepository,
}

impl ReviewService {
    /// Builds the service on a repository.
    pub fn new(reviews: ReviewRepository) -> Self {
        ReviewService { reviews }
    }

    /// Anyone logged in may write a review; it's theirs.
    pub async fn create(&self, actor: &AuthUser, new: NewReview) -> Result<Review, AppError> {
        let new = new.validate().map_err(AppError::Validation)?;
        let review = self.reviews.insert(actor.id, &new).await?;
        tracing::info!(review_id = review.id, user_id = actor.id, "review created");
        Ok(review)
    }

    /// One review, for anyone.
    pub async fn get(&self, id: i64) -> Result<Review, AppError> {
        self.reviews
            .find(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("review {id}")))
    }

    /// A page of reviews, for anyone.
    pub async fn list(&self, params: &ListParams) -> Result<Page<Review>, AppError> {
        if let Some(min) = params.min_rating
            && !(1..=5).contains(&min)
        {
            return Err(AppError::BadRequest(
                "min_rating must be from 1 to 5".into(),
            ));
        }
        let (page, per_page) = params.page_and_size();
        let (items, total) = self.reviews.list(params).await?;
        Ok(Page::new(items, page, per_page, total))
    }

    /// Edits a review: 404 if it doesn't exist, 403 if it isn't yours
    /// (unless you're an admin), 422 if the patch breaks the rules.
    pub async fn update(
        &self,
        actor: &AuthUser,
        id: i64,
        patch: ReviewPatch,
    ) -> Result<Review, AppError> {
        let author = self.author(id).await?;
        authorize(actor, Action::Edit, author)?;
        let patch = patch.validate().map_err(AppError::Validation)?;
        self.reviews
            .update(id, &patch)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("review {id}")))
    }

    /// Deletes a review, with the same rules as `update`.
    pub async fn delete(&self, actor: &AuthUser, id: i64) -> Result<(), AppError> {
        let author = self.author(id).await?;
        authorize(actor, Action::Delete, author)?;
        if self.reviews.delete(id).await? {
            tracing::info!(review_id = id, user_id = actor.id, "review deleted");
            Ok(())
        } else {
            Err(AppError::NotFound(format!("review {id}")))
        }
    }

    async fn author(&self, id: i64) -> Result<i64, AppError> {
        self.reviews
            .author_of(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("review {id}")))
    }
}
