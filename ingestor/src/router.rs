use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers;

pub fn build_routes() -> Router {
    Router::new()
        .route("/livez", get(handlers::probes::livez))
        .nest(
            "/ingest",
            Router::new()
                .route("/har", post(handlers::ingest::ingest_har))
                .route("/console", post(handlers::ingest::ingest_console)),
        )
}
