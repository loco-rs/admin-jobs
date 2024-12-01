use admin_panel::app::App;
use loco_rs::cli;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    cli::main::<App>().await
}
