use serde::{Deserialize, Serialize};

use crate::item::ItemError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: Option<i64>,
    pub name: String,
    pub temperature_f: f64,
}

impl Location {
    pub fn new(name: impl Into<String>, temperature_f: f64) -> Self {
        Self {
            id: None,
            name: name.into(),
            temperature_f,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shelf {
    pub id: Option<i64>,
    pub location_id: i64,
    pub name: String,
}

impl Shelf {
    pub fn new(location_id: i64, name: impl Into<String>) -> Self {
        Self {
            id: None,
            location_id,
            name: name.into(),
        }
    }
}

pub trait LocationRepository: Send + Sync {
    fn add_location(&self, location: &Location) -> Result<i64, ItemError>;
    fn get_location(&self, id: i64) -> Result<Location, ItemError>;
    fn list_locations(&self) -> Result<Vec<Location>, ItemError>;
    fn update_location(&self, location: &Location) -> Result<(), ItemError>;
    fn remove_location(&self, id: i64) -> Result<(), ItemError>;
}

pub trait ShelfRepository: Send + Sync {
    fn add_shelf(&self, shelf: &Shelf) -> Result<i64, ItemError>;
    fn get_shelf(&self, id: i64) -> Result<Shelf, ItemError>;
    fn list_shelves(&self, location_id: i64) -> Result<Vec<Shelf>, ItemError>;
    fn list_all_shelves(&self) -> Result<Vec<Shelf>, ItemError>;
    fn remove_shelf(&self, id: i64) -> Result<(), ItemError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_shelf_defaults() {
        let shelf = Shelf::new(1, "Top Shelf");
        assert!(shelf.id.is_none());
        assert_eq!(shelf.location_id, 1);
        assert_eq!(shelf.name, "Top Shelf");
    }

    #[test]
    fn new_location_defaults() {
        let loc = Location::new("Fridge", 37.0);
        assert!(loc.id.is_none());
        assert_eq!(loc.name, "Fridge");
        assert!((loc.temperature_f - 37.0).abs() < f64::EPSILON);
    }
}
