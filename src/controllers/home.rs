use crate::shared::db::DB;
use axum::{debug_handler, Extension};
use loco_rs::prelude::*;
use std::sync::Arc;

#[debug_handler]
async fn index(
    Extension(db): Extension<Arc<DB>>,
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    let status_stats = db.status_stats().await.map_err(|err| {
        tracing::error!(
            error = %err,
            "Could not getting job stats"
        );
        Error::string("action failed")
    })?;
    let jobs = db.get_jobs(None, None).await.map_err(|err| {
        tracing::error!(
            error = %err,
            "Could not get jobs list."
        );
        Error::string("action failed")
    })?;

    format::render().view(
        &v,
        "home/index.html",
        data!({"status_stats": status_stats, "jobs": jobs}),
    )
}

pub fn routes() -> Routes {
    Routes::new().add("/", get(index))
}
