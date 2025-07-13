use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: String,
    pub name: String,
    pub lang_local: String,
    pub registered_at: DateTime<Utc>,
}
