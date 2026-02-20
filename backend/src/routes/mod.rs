pub mod posts;
pub mod tags;

use crate::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/posts", post_routes())
        .nest("/api/tags", tag_routes())
        .with_state(state)
}

pub fn post_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(posts::get_posts).post(posts::create_post))
        .route("/{id}", get(posts::get_one_post))
}

pub fn tag_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(tags::fetch_tags))
        .route("/{uuid}", post(tags::increment_tag_selection))
}
