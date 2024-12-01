use crate::utils;
use admin_panel::app::App;
use loco_rs::testing;
use serial_test::serial;
use std::path::PathBuf;

#[tokio::test]
#[serial]
async fn get_homepage() {
    testing::request::<App, _, _>(|request, _ctx| async move {
        utils::prepare_sqlite_db(&PathBuf::from("."), "test.sqlite").await;
        let res = request.get("/").await;

        assert_eq!(res.status_code(), 200);
    })
    .await;
}
