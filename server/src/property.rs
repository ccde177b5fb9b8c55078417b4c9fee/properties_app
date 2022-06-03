use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
pub struct Property {
    #[serde(default)]
    pub id: i32,
    pub name: String,
    pub location: i32,
    pub area: i32,
    pub property_type: i32,
    pub wc: i32,
    pub floor: i32,
    pub tothesea: i32,
    pub furniture: bool,
    pub appliances: bool,
    pub price: i32,
    #[serde(skip_deserializing)]
    pub posting_date: Option<DateTime<Utc>>,
    #[serde(skip_deserializing)]
    pub gallery_location: Option<String>,
}
