pub mod authors;
pub mod posts;
pub mod sites;
pub mod tags;

use crate::AppState;
use axum::{
    Router,
    routing::{get, post, delete},
};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/posts", post_routes())
        .nest("/api/tags", tag_routes())
        .nest("/api/sites", site_routes())
        .nest("/api/authors", author_routes())
        .with_state(state)
}

pub fn post_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(posts::get_posts).post(posts::create_post))
        .route("/{id}", 
            get(posts::get_one_post)
            .put(posts::update_post)
            .delete(posts::delete_post)
        )
}

pub fn tag_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(tags::fetch_tags))
        .route("/admin", get(tags::admin_fetch_tags))
        .route("/{uuid}", post(tags::increment_tag_selection))
}

pub fn site_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(sites::get_sites).post(sites::create_site))
        .route("/{id}", delete(sites::delete_site))
}

pub fn author_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(authors::get_authors).post(authors::create_author))
        .route("/{uuid}/socials", post(authors::add_social))
}
