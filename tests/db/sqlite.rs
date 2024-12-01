use crate::utils;
use admin_panel::shared::db;
use insta::assert_debug_snapshot;

#[tokio::test]
async fn can_get_status_stats() {
    let tree_fs = tree_fs::TreeBuilder::default()
        .drop(true)
        .create()
        .expect("create temp folder");
    let connector = utils::prepare_sqlite_db(&tree_fs.root, "sample.sqlite").await;

    assert_debug_snapshot!(connector.status_stats().await);
}

#[rstest::rstest]
#[tokio::test]
async fn can_get_jobs(
    #[values(None, Some("UserAccountActivation".to_string()))] search: Option<String>,
    #[values(None, Some("completed".to_string()))] status: Option<String>,
) {
    let tree_fs = tree_fs::TreeBuilder::default()
        .drop(true)
        .create()
        .expect("create temp folder");
    let connector = utils::prepare_sqlite_db(&tree_fs.root, "sample.sqlite").await;

    assert_debug_snapshot!(
        format!("search[{search:?}]-status[{status:?}]"),
        connector.get_jobs(search, status).await
    );
}

#[tokio::test]
async fn can_get_job() {
    let tree_fs = tree_fs::TreeBuilder::default()
        .drop(true)
        .create()
        .expect("create temp folder");
    let connector = utils::prepare_sqlite_db(&tree_fs.root, "sample.sqlite").await;

    assert_debug_snapshot!(connector.get_job("01JDM0X8EVAM823JZBGKYNBA99").await);
}

#[tokio::test]
async fn can_delete_job() {
    let tree_fs = tree_fs::TreeBuilder::default()
        .drop(true)
        .create()
        .expect("create temp folder");
    let connector = utils::prepare_sqlite_db(&tree_fs.root, "sample.sqlite").await;
    let job_id = "01JDM0X8EVAM823JZBGKYNBA99";
    assert!(connector.get_job(job_id).await.expect("get job").is_some());
    assert!(connector.delete_job(job_id).await.is_ok());
    assert!(connector.get_job(job_id).await.expect("get job").is_none());
}

#[tokio::test]
async fn can_update_status() {
    let tree_fs = tree_fs::TreeBuilder::default()
        .drop(true)
        .create()
        .expect("create temp folder");
    let connector = utils::prepare_sqlite_db(&tree_fs.root, "sample.sqlite").await;
    let job_id = "01JDM0X8EVAM823JZBGKYNBA99";
    assert_eq!(
        connector
            .get_job(job_id)
            .await
            .expect("get job")
            .expect("job")
            .status,
        "queued"
    );

    assert!(connector
        .update_status(job_id, &db::JobStatus::Failed)
        .await
        .is_ok());

    assert_eq!(
        connector
            .get_job(job_id)
            .await
            .expect("get job")
            .expect("job")
            .status,
        "failed"
    );
}
