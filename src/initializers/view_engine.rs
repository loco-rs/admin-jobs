use axum::{async_trait, Extension, Router as AxumRouter};
use loco_rs::{
    app::{AppContext, Initializer},
    controller::views::{engines, ViewEngine},
    Result,
};

#[allow(clippy::module_name_repetitions)]
pub struct ViewEngineInitializer;

#[async_trait]
impl Initializer for ViewEngineInitializer {
    fn name(&self) -> String {
        "view-engine".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        let tera_engine = engines::TeraView::build()?;
        Ok(router.layer(Extension(ViewEngine::from(tera_engine))))
    }
}
