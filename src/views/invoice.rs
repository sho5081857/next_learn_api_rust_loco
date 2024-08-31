use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use chrono::naive::NaiveDate as Date;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromQueryResult)]
pub struct GetAllLatestInvoiceResponse {
    pub id: Uuid,
    pub name: String,
    pub image_url: Option<String>,
    pub email: String,
    pub amount: i64,
}


#[derive(Debug, Serialize, Deserialize, FromQueryResult)]
pub struct GetFilteredInvoiceResponse {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub name: String,
    pub email: String,
    pub image_url: Option<String>,
    pub amount: i64,
    pub date: Date,
    pub status: String,
}
