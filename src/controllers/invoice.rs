#![allow(clippy::unused_async)]
use axum::{debug_handler, extract::Query};
use chrono::naive::NaiveDate as Date;
use loco_rs::prelude::*;
use migration::Expr;

use sea_orm::{
    Condition, JoinType, QueryOrder, QuerySelect, RelationTrait,
};
use serde::{Deserialize, Serialize};
use std::string::ToString;
use uuid::Uuid;

use crate::{
    models::_entities::{
        customers,
        invoices::{self, ActiveModel, Entity, Model},
    },
    views::invoice::{GetAllLatestInvoiceResponse, GetFilteredInvoiceResponse},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Queries {
    pub query: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvoiceRequest {
    pub customer_id: Option<Uuid>,
    pub amount: Option<i32>,
    pub status: Option<String>,
    pub date: Option<Date>,
}

impl InvoiceRequest {
    fn update(&self, item: &mut ActiveModel) {
        item.customer_id = Set(self.customer_id.unwrap());
        item.amount = Set(self.amount.unwrap());
        item.status = Set(self.status.clone().unwrap());
        if let Some(date) = self.date {
            item.date = Set(date);
        }
    }
}

async fn load_item(ctx: &AppContext, id: Uuid) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[debug_handler]
pub async fn get_all_latest(_auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let invoices = Entity::find()
        .find_with_related(customers::Entity)
        .order_by_desc(invoices::Column::Date)
        .into_json()
        .all(&ctx.db)
        .await?;

    let invoice_responses: Result<Vec<GetAllLatestInvoiceResponse>, Box<dyn std::error::Error>> =
        invoices
            .into_iter()
            .map(|(invoice, customer_opt)| {
                let customer = customer_opt.ok_or("Missing customer")?;
                Ok(GetAllLatestInvoiceResponse {
                    id: Uuid::parse_str(invoice["id"].as_str().ok_or("Missing id")?)?,
                    name: customer["name"].as_str().ok_or("Missing name")?.to_string(),
                    image_url: customer["image_url"].as_str().map(|s| s.to_string()),
                    email: customer["email"]
                        .as_str()
                        .ok_or("Missing email")?
                        .to_string(),
                    amount: invoice["amount"].as_i64().ok_or("Missing amount")?,
                })
            })
            .collect();

    match invoice_responses {
        Ok(invoices) => {
            println!("{:?}", invoices);
            format::json(invoices)
        }
        Err(e) => {
            eprintln!("Error mapping invoice responses: {}", e);
            format::json("Error processing request")
        }
    }
}

#[debug_handler]
pub async fn get_filtered(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(queries): Query<Queries>,
) -> Result<Response> {
    let query = queries.query.unwrap_or_default();
    let invoices = Entity::find()
        .find_with_related(customers::Entity)
        .filter(
            Condition::any()
                .add(customers::Column::Name.contains(query.to_lowercase()))
                .add(customers::Column::Email.contains(query.to_lowercase()))
                .add(Expr::cust(format!(
                    "CAST(invoices.amount AS TEXT) LIKE '%{}%'",
                    query
                )))
                .add(Expr::cust(format!(
                    "CAST(invoices.date AS TEXT) LIKE '%{}%'",
                    query
                ))),
        )
        .order_by_desc(invoices::Column::Date)
        .into_json()
        .all(&ctx.db)
        .await?;

    let invoice_responses: Result<Vec<GetFilteredInvoiceResponse>, Box<dyn std::error::Error>> =
        invoices
            .into_iter()
            .map(|(invoice, customer_opt)| {
                let customer = customer_opt.ok_or("Missing customer")?;
                Ok(GetFilteredInvoiceResponse {
                    id: Uuid::parse_str(invoice["id"].as_str().ok_or("Missing id")?)?,
                    customer_id: Uuid::parse_str(
                        invoice["customer_id"].as_str().ok_or("Missing id")?,
                    )?,
                    name: customer["name"].as_str().ok_or("Missing name")?.to_string(),
                    email: customer["email"]
                        .as_str()
                        .ok_or("Missing email")?
                        .to_string(),
                    image_url: customer["image_url"].as_str().map(|s| s.to_string()),
                    amount: invoice["amount"].as_i64().ok_or("Missing amount")?,
                    date: Date::parse_from_str(
                        invoice["date"].as_str().ok_or("Missing date")?,
                        "%Y-%m-%d",
                    )
                    .map_err(|_| "Invalid date format")?,
                    status: invoice["status"]
                        .as_str()
                        .ok_or("Missing status")?
                        .to_string(),
                })
            })
            .collect();

    match invoice_responses {
        Ok(invoices) => format::json(invoices),
        Err(e) => {
            eprintln!("Error mapping invoice responses: {}", e);
            format::json("Error processing request")
        }
    }
}

#[debug_handler]
pub async fn get_count(_auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    format::json(
        Entity::find()
            .select_only()
            .column_as(invoices::Column::Id.count(), "count")
            .into_tuple::<i64>()
            .one(&ctx.db)
            .await?,
    )
}

#[debug_handler]
pub async fn get_status_count(_auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let pending = Entity::find()
        .select_only()
        .column_as(invoices::Column::Id.count(), "count")
        .filter(invoices::Column::Status.eq("pending"))
        .into_tuple::<i64>()
        .one(&ctx.db)
        .await?;
    let paid = Entity::find()
        .select_only()
        .column_as(invoices::Column::Id.count(), "count")
        .filter(invoices::Column::Status.eq("paid"))
        .into_tuple::<i64>()
        .one(&ctx.db)
        .await?;
    format::json(serde_json::json!({"pending": pending, "paid": paid}))
}

#[debug_handler]
pub async fn get_pages(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(queries): Query<Queries>,
) -> Result<Response> {
    let query = queries.query.unwrap_or_default();
    let count = Entity::find()
        .join(JoinType::InnerJoin, invoices::Relation::Customer.def())
        .filter(
            Condition::any()
                .add(customers::Column::Name.contains(query.to_lowercase()))
                .add(customers::Column::Email.contains(query.to_lowercase()))
                .add(Expr::cust(format!(
                    "CAST(invoices.amount AS TEXT) LIKE '%{}%'",
                    query
                )))
                .add(Expr::cust(format!(
                    "CAST(invoices.date AS TEXT) LIKE '%{}%'",
                    query
                ))),
        )
        .select_only()
        .column_as(Expr::cust("COUNT(*)"), "count")
        .into_tuple::<i64>()
        .one(&ctx.db)
        .await?;

    format::json(count)
}

#[debug_handler]
pub async fn get_by_id(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(invoice_id): Path<Uuid>
) -> Result<Response> {
    let invoice = Entity::find()
        .filter(invoices::Column::Id.eq(invoice_id))
        .one(&ctx.db)
        .await?;
    format::json(invoice)
}

#[debug_handler]
pub async fn create(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(invoice_request): Json<InvoiceRequest>,
) -> Result<Response> {
    let mut item = ActiveModel {
        ..Default::default()
    };
    invoice_request.update(&mut item);
    let item = item.insert(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn update(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<Uuid>,
    Json(invoice_request): Json<InvoiceRequest>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    let mut item = item.into_active_model();
    invoice_request.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn remove(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<Uuid>,
) -> Result<()> {
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    Ok(())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("invoices")
        .add("/latest", get(get_all_latest))
        .add("/filtered", get(get_filtered))
        .add("/count", get(get_count))
        .add("/statusCount", get(get_status_count))
        .add("/pages", get(get_pages))
        .add("/:invoiceId", get(get_by_id))
        .add("", post(create))
        .add("/:invoiceId", put(update))
        .add("/:invoiceId", delete(remove))
}
