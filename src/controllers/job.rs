use crate::shared::db::{JobStatus, DB};
use axum::{debug_handler, extract::Query, Extension};
use loco_rs::prelude::*;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct SearchQueryParams {
    pub status: Option<String>,
    pub search: Option<String>,
}

#[debug_handler]
async fn search(
    Query(params): Query<SearchQueryParams>,
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
    let jobs = db
        .get_jobs(params.search.clone(), params.status.clone())
        .await
        .map_err(|err| {
            tracing::error!(
                error = %err,
                "Could not get jobs list."
            );
            Error::string("action failed")
        })?;

    format::render().view(
        &v,
        "jobs/_list.html",
        data!({"status_stats": status_stats, "jobs": jobs,  "filters": {"search": params.search, "status": params.status}}),
    )
}

#[debug_handler]
async fn remove(Path(job_id): Path<String>, Extension(db): Extension<Arc<DB>>) -> Result<Response> {
    db.delete_job(&job_id).await.map_err(|err| {
        tracing::error!(
            error = %err,
            job_id,
            "Failed to delete job"
        );
        Error::string(&format!("Unable to delete job id '{job_id}'",))
    })?;
    format::RenderBuilder::default().json(serde_json::json!({}))
}

#[debug_handler]
async fn retry(
    Path(job_id): Path<String>,
    Extension(db): Extension<Arc<DB>>,
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    let job = db
        .update_status(&job_id, &JobStatus::Queued)
        .await
        .map_err(|err| {
            tracing::error!(
                error = %err,
                job_id,
                "Failed to update job status to 'Queued'."
            );
            Error::string("action failed")
        })?;

    format::render().view(&v, "jobs/_job.html", data!({"job": job}))
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/jobs/search", get(search))
        .add("/jobs/:id", delete(remove))
        .add("/jobs/retry/:id", post(retry))
}
