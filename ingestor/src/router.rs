use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers;

pub fn build_routes() -> Router {
    Router::new()
        .route("/livez", get(handlers::probes::livez))
        .route("/logs", post(handlers::ingest::ingest_logs))
        .nest(
            "/ingest",
            Router::new()
                .route("/har", post(handlers::ingest::ingest_har))
                .route("/console", post(handlers::ingest::ingest_logs)),
        )
}
