#![allow(clippy::unused_async)]
use loco_rs::{controller::middleware, prelude::*};
use axum::debug_handler;

use crate::models::_entities::{revenues::Entity, users};

#[debug_handler]
pub async fn get_all(_auth: middleware::auth::ApiToken<users::Model>, State(ctx): State<AppContext>) -> Result<Response> {
    format::json(Entity::find().all(&ctx.db).await?)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("revenue")
        .add("", get(get_all))
}
