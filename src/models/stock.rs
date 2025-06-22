use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;
use std::fmt::Display;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Stock {
    pub id: uuid::Uuid,
    pub supplier: String,
    pub name: String,
    pub stock: f64,
    pub updated: DateTime<Utc>,
}
impl Stock {
    pub fn safe_print(&self) -> String {
        format!("📛 Наименование: {}\n📦 Остаток: {:.2}\n🕒 Обновлено: {}\n", self.name, self.stock, self.updated.format("%d.%m.%Y %H:%M"))
    }
}

impl Display for Stock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "🏭 Поставщик: {}", self.supplier)?;
        writeln!(f, "📛 Наименование: {}", self.name)?;
        writeln!(f, "📦 Остаток: {:.2}", self.stock)?;
        write!(f, "🕒 Обновлено: {}", self.updated.format("%d.%m.%Y %H:%M"))
    }
}