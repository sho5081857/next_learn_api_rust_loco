#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use axum::debug_handler;

use crate::models::_entities::revenues::Entity;

#[debug_handler]
pub async fn get_all(_auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    format::json(Entity::find().all(&ctx.db).await?)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("revenues")
        .add("", get(get_all))
}
