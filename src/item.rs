use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ItemError {
    #[error("item not found: {0}")]
    NotFound(i64),
    #[error("location not found: {0}")]
    LocationNotFound(i64),
    #[error("shelf not found: {0}")]
    ShelfNotFound(i64),
    #[error("database error: {0}")]
    Database(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroceryItem {
    pub id: Option<i64>,
    pub name: String,
    pub quantity: u32,
    pub unit: String,
    pub category: Option<String>,
    pub expiration_date: Option<NaiveDate>,
    pub minimum_stock: u32,
    pub location_id: Option<i64>,
    pub shelf_id: Option<i64>,
}

impl GroceryItem {
    pub fn new(name: impl Into<String>, quantity: u32, unit: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            quantity,
            unit: unit.into(),
            category: None,
            expiration_date: None,
            minimum_stock: 0,
            location_id: None,
            shelf_id: None,
        }
    }

    pub fn is_low_stock(&self, threshold: u32) -> bool {
        self.quantity <= threshold || self.quantity <= self.minimum_stock
    }
}

pub trait ItemRepository: Send + Sync {
    fn add(&self, item: &GroceryItem) -> Result<i64, ItemError>;
    fn get(&self, id: i64) -> Result<GroceryItem, ItemError>;
    fn list(&self) -> Result<Vec<GroceryItem>, ItemError>;
    fn update(&self, item: &GroceryItem) -> Result<(), ItemError>;
    fn remove(&self, id: i64) -> Result<(), ItemError>;
    fn find_low_stock(&self, threshold: u32) -> Result<Vec<GroceryItem>, ItemError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_item_defaults() {
        let item = GroceryItem::new("Milk", 2, "gallons");
        assert!(item.id.is_none());
        assert_eq!(item.name, "Milk");
        assert_eq!(item.quantity, 2);
        assert_eq!(item.unit, "gallons");
        assert!(item.category.is_none());
        assert!(item.expiration_date.is_none());
        assert_eq!(item.minimum_stock, 0);
    }

    #[test]
    fn low_stock_detection() {
        let item = GroceryItem::new("Eggs", 1, "dozen");
        assert!(item.is_low_stock(2));
        assert!(!item.is_low_stock(0));
    }

    #[test]
    fn low_stock_uses_minimum_stock() {
        let mut item = GroceryItem::new("Butter", 3, "sticks");
        item.minimum_stock = 4;
        assert!(item.is_low_stock(0)); // below minimum_stock
    }
}
