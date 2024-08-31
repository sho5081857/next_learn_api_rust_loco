#![allow(clippy::unused_async)]
use axum::{debug_handler, extract::Query};
use loco_rs::prelude::*;
use migration::Expr;
use sea_orm::{Condition, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::_entities::{
        customers::{self, Entity},
        invoices,
    },
    views::customer::CustomerResponse,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Queries {
    pub query: Option<String>,
}

#[debug_handler]
pub async fn get_all(_auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    format::json(Entity::find().all(&ctx.db).await?)
}

#[debug_handler]
pub async fn get_filtered(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(queries): Query<Queries>,
) -> Result<Response> {
    let query = queries.query.unwrap_or_default();

    let customers = Entity::find()
        .select_only()
        .columns([
            customers::Column::Id,
            customers::Column::Name,
            customers::Column::Email,
            customers::Column::ImageUrl,
        ])
        .column_as(invoices::Column::Id.count(), "total_invoices")
        .column_as(
            Expr::cust(
                "SUM(CASE WHEN invoices.status = 'pending' THEN invoices.amount ELSE 0 END)",
            ),
            "total_pending",
        )
        .column_as(
            Expr::cust("SUM(CASE WHEN invoices.status = 'paid' THEN invoices.amount ELSE 0 END)"),
            "total_paid",
        )
        .find_with_related(invoices::Entity)
        .filter(
            Condition::any()
                .add(customers::Column::Name.contains(query.to_lowercase()))
                .add(customers::Column::Email.contains(query.to_lowercase())),
        )
        .group_by(customers::Column::Id)
        .group_by(customers::Column::Name)
        .group_by(customers::Column::Email)
        .group_by(customers::Column::ImageUrl)
        .group_by(invoices::Column::Id)
        .order_by_asc(customers::Column::Name)
        .into_json()
        .all(&ctx.db)
        .await?;

    let customer_responses: Result<Vec<CustomerResponse>, Box<dyn std::error::Error>> = customers
        .into_iter()
        .map(|(customer, _)| {
            Ok(CustomerResponse {
                id: Uuid::parse_str(customer["id"].as_str().ok_or("Missing id")?)?,
                name: customer["name"].as_str().ok_or("Missing name")?.to_string(),
                email: customer["email"]
                    .as_str()
                    .ok_or("Missing email")?
                    .to_string(),
                image_url: customer["image_url"].as_str().map(|s| s.to_string()),
                total_invoices: customer["total_invoices"]
                    .as_i64()
                    .ok_or("Missing total_invoices")?,
                total_pending: customer["total_pending"].as_i64(),
                total_paid: customer["total_paid"].as_i64(),
            })
        })
        .collect();

    match customer_responses {
        Ok(customers) => format::json(customers),
        Err(e) => {
            eprintln!("Error mapping customer responses: {}", e);
            format::json("Error processing request")
        }
    }
}

#[debug_handler]
pub async fn get_count(_auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    format::json(
        Entity::find()
            .select_only()
            .column_as(customers::Column::Id.count(), "count")
            .into_tuple::<i64>()
            .one(&ctx.db)
            .await?,
    )
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("customers")
        .add("", get(get_all))
        .add("/filtered", get(get_filtered))
        .add("/count", get(get_count))
}
