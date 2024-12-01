use crate::utils;
use admin_panel::app::App;
use loco_rs::testing;
use serial_test::serial;
use std::path::PathBuf;

#[rstest::rstest]
#[tokio::test]
#[serial]
async fn search_jobs(
    #[values(None, Some("UserAccountActivation".to_string()))] search: Option<String>,
    #[values(None, Some("completed".to_string()))] status: Option<String>,
) {
    testing::request::<App, _, _>(|request, _ctx| async move {
        utils::prepare_sqlite_db(&PathBuf::from("."), "test.sqlite").await;

        let res = request
            .get("/jobs/search")
            .add_query_param("search", search)
            .add_query_param("status", status)
            .await;

        assert_eq!(res.status_code(), 200);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn delete_job() {
    testing::request::<App, _, _>(|request, _ctx| async move {
        let connector = utils::prepare_sqlite_db(&PathBuf::from("."), "test.sqlite").await;
        let job_id = "01JDM0X8EVAM823JZBGKYNBA99";
        assert!(connector.get_job(job_id).await.expect("get job").is_some());
        let res = request.delete(&format!("/jobs/{job_id}")).await;

        assert_eq!(res.status_code(), 200);
        assert!(connector.get_job(job_id).await.expect("get job").is_none());
    })
    .await;
}

#[tokio::test]
#[serial]
async fn job_retry() {
    testing::request::<App, _, _>(|request, _ctx| async move {
        let connector = utils::prepare_sqlite_db(&PathBuf::from("."), "test.sqlite").await;
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
        let res = request.post(&format!("/jobs/retry/{job_id}")).await;

        assert_eq!(res.status_code(), 200);
        assert_eq!(
            connector
                .get_job(job_id)
                .await
                .expect("get job")
                .expect("job")
                .status,
            "queued"
        );
    })
    .await;
}
