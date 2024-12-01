use crate::{
    controllers, initializers,
    shared::{config::DBConfig, db},
};
use async_trait::async_trait;
use axum::{Extension, Router as AxumRouter};
use loco_rs::{
    app::{AppContext, Hooks, Initializer},
    bgworker::Queue,
    boot::{create_app, BootResult, StartMode},
    controller::AppRoutes,
    environment::Environment,
    task::Tasks,
    Result,
};
use std::sync::Arc;

pub struct App;
#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(mode: StartMode, environment: &Environment) -> Result<BootResult> {
        create_app::<Self>(mode, environment).await
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![Box::new(
            initializers::view_engine::ViewEngineInitializer,
        )])
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .add_route(controllers::home::routes())
            .add_route(controllers::job::routes())
    }

    async fn after_routes(router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
        let db_config = DBConfig::from_context(ctx)?;

        let db = db::init(db_config).await.map_err(|err| {
            tracing::error!(
                err = %err,
                "connection error"
            );
            loco_rs::Error::string("connection error")
        })?;
        Ok(router.layer(Extension(Arc::new(db))))
    }

    async fn connect_workers(_ctx: &AppContext, _queue: &Queue) -> Result<()> {
        Ok(())
    }
    fn register_tasks(_tasks: &mut Tasks) {}
}
