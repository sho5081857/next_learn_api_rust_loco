use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromQueryResult)]
pub struct CustomerResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub image_url: Option<String>,
    pub total_invoices: i64,
    pub total_pending: Option<i64>,
    pub total_paid: Option<i64>,
}

