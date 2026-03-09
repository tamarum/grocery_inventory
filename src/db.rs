use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection};

use crate::item::{GroceryItem, ItemError, ItemRepository};
use crate::location::{Location, LocationRepository, Shelf, ShelfRepository};

pub struct SqliteRepository {
    conn: Mutex<Connection>,
}

impl SqliteRepository {
    pub fn new(path: &Path) -> Result<Self, ItemError> {
        let conn = Connection::open(path).map_err(|e| ItemError::Database(e.to_string()))?;
        let repo = Self {
            conn: Mutex::new(conn),
        };
        repo.initialize_schema()?;
        Ok(repo)
    }

    pub fn in_memory() -> Result<Self, ItemError> {
        let conn = Connection::open_in_memory().map_err(|e| ItemError::Database(e.to_string()))?;
        let repo = Self {
            conn: Mutex::new(conn),
        };
        repo.initialize_schema()?;
        Ok(repo)
    }

    fn conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, ItemError> {
        self.conn
            .lock()
            .map_err(|e| ItemError::Database(format!("lock poisoned: {e}")))
    }

    fn initialize_schema(&self) -> Result<(), ItemError> {
        let conn = self.conn()?;

        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| ItemError::Database(e.to_string()))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS locations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                temperature_f REAL NOT NULL
            );

            CREATE TABLE IF NOT EXISTS shelves (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                location_id INTEGER NOT NULL REFERENCES locations(id) ON DELETE CASCADE,
                name TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS grocery_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                quantity INTEGER NOT NULL DEFAULT 0,
                unit TEXT NOT NULL DEFAULT '',
                category TEXT,
                expiration_date TEXT,
                minimum_stock INTEGER NOT NULL DEFAULT 0,
                location_id INTEGER REFERENCES locations(id) ON DELETE SET NULL,
                shelf_id INTEGER REFERENCES shelves(id) ON DELETE SET NULL
            );",
        )
        .map_err(|e| ItemError::Database(e.to_string()))?;

        // Migration: add location_id column if it doesn't exist (for existing databases)
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(grocery_items)")
            .map_err(|e| ItemError::Database(e.to_string()))?
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| ItemError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ItemError::Database(e.to_string()))?;

        if !columns.iter().any(|c| c == "location_id") {
            conn.execute_batch(
                "ALTER TABLE grocery_items ADD COLUMN location_id INTEGER REFERENCES locations(id) ON DELETE SET NULL;",
            )
            .map_err(|e| ItemError::Database(e.to_string()))?;
        }

        if !columns.iter().any(|c| c == "shelf_id") {
            conn.execute_batch(
                "ALTER TABLE grocery_items ADD COLUMN shelf_id INTEGER REFERENCES shelves(id) ON DELETE SET NULL;",
            )
            .map_err(|e| ItemError::Database(e.to_string()))?;
        }

        Ok(())
    }

    fn row_to_item(row: &rusqlite::Row) -> rusqlite::Result<GroceryItem> {
        let expiration_str: Option<String> = row.get(5)?;
        let expiration_date =
            expiration_str.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());

        Ok(GroceryItem {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            quantity: row.get(2)?,
            unit: row.get(3)?,
            category: row.get(4)?,
            expiration_date,
            minimum_stock: row.get(6)?,
            location_id: row.get(7)?,
            shelf_id: row.get(8)?,
        })
    }

    fn row_to_shelf(row: &rusqlite::Row) -> rusqlite::Result<Shelf> {
        Ok(Shelf {
            id: Some(row.get(0)?),
            location_id: row.get(1)?,
            name: row.get(2)?,
        })
    }

    fn row_to_location(row: &rusqlite::Row) -> rusqlite::Result<Location> {
        Ok(Location {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            temperature_f: row.get(2)?,
        })
    }
}

impl ItemRepository for SqliteRepository {
    fn add(&self, item: &GroceryItem) -> Result<i64, ItemError> {
        let conn = self.conn()?;
        let expiration = item
            .expiration_date
            .map(|d| d.format("%Y-%m-%d").to_string());
        conn.execute(
            "INSERT INTO grocery_items (name, quantity, unit, category, expiration_date, minimum_stock, location_id, shelf_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                item.name,
                item.quantity,
                item.unit,
                item.category,
                expiration,
                item.minimum_stock,
                item.location_id,
                item.shelf_id,
            ],
        )
        .map_err(|e| ItemError::Database(e.to_string()))?;
        Ok(conn.last_insert_rowid())
    }

    fn get(&self, id: i64) -> Result<GroceryItem, ItemError> {
        let conn = self.conn()?;
        conn.query_row(
            "SELECT id, name, quantity, unit, category, expiration_date, minimum_stock, location_id, shelf_id
             FROM grocery_items WHERE id = ?1",
            params![id],
            Self::row_to_item,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => ItemError::NotFound(id),
            other => ItemError::Database(other.to_string()),
        })
    }

    fn list(&self) -> Result<Vec<GroceryItem>, ItemError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, quantity, unit, category, expiration_date, minimum_stock, location_id, shelf_id
                 FROM grocery_items ORDER BY name",
            )
            .map_err(|e| ItemError::Database(e.to_string()))?;

        let items = stmt
            .query_map([], Self::row_to_item)
            .map_err(|e| ItemError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ItemError::Database(e.to_string()))?;

        Ok(items)
    }

    fn update(&self, item: &GroceryItem) -> Result<(), ItemError> {
        let id = item.id.ok_or(ItemError::Database(
            "cannot update item without id".to_string(),
        ))?;
        let conn = self.conn()?;
        let expiration = item
            .expiration_date
            .map(|d| d.format("%Y-%m-%d").to_string());
        let rows = conn
            .execute(
                "UPDATE grocery_items
                 SET name = ?1, quantity = ?2, unit = ?3, category = ?4,
                     expiration_date = ?5, minimum_stock = ?6, location_id = ?7, shelf_id = ?8
                 WHERE id = ?9",
                params![
                    item.name,
                    item.quantity,
                    item.unit,
                    item.category,
                    expiration,
                    item.minimum_stock,
                    item.location_id,
                    item.shelf_id,
                    id,
                ],
            )
            .map_err(|e| ItemError::Database(e.to_string()))?;

        if rows == 0 {
            return Err(ItemError::NotFound(id));
        }
        Ok(())
    }

    fn remove(&self, id: i64) -> Result<(), ItemError> {
        let conn = self.conn()?;
        let rows = conn
            .execute("DELETE FROM grocery_items WHERE id = ?1", params![id])
            .map_err(|e| ItemError::Database(e.to_string()))?;

        if rows == 0 {
            return Err(ItemError::NotFound(id));
        }
        Ok(())
    }

    fn find_low_stock(&self, threshold: u32) -> Result<Vec<GroceryItem>, ItemError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, quantity, unit, category, expiration_date, minimum_stock, location_id, shelf_id
                 FROM grocery_items
                 WHERE quantity <= ?1 OR quantity <= minimum_stock
                 ORDER BY name",
            )
            .map_err(|e| ItemError::Database(e.to_string()))?;

        let items = stmt
            .query_map(params![threshold], Self::row_to_item)
            .map_err(|e| ItemError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ItemError::Database(e.to_string()))?;

        Ok(items)
    }
}

impl LocationRepository for SqliteRepository {
    fn add_location(&self, location: &Location) -> Result<i64, ItemError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO locations (name, temperature_f) VALUES (?1, ?2)",
            params![location.name, location.temperature_f],
        )
        .map_err(|e| ItemError::Database(e.to_string()))?;
        Ok(conn.last_insert_rowid())
    }

    fn get_location(&self, id: i64) -> Result<Location, ItemError> {
        let conn = self.conn()?;
        conn.query_row(
            "SELECT id, name, temperature_f FROM locations WHERE id = ?1",
            params![id],
            Self::row_to_location,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => ItemError::LocationNotFound(id),
            other => ItemError::Database(other.to_string()),
        })
    }

    fn list_locations(&self) -> Result<Vec<Location>, ItemError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, name, temperature_f FROM locations ORDER BY name")
            .map_err(|e| ItemError::Database(e.to_string()))?;

        let locations = stmt
            .query_map([], Self::row_to_location)
            .map_err(|e| ItemError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ItemError::Database(e.to_string()))?;

        Ok(locations)
    }

    fn update_location(&self, location: &Location) -> Result<(), ItemError> {
        let id = location.id.ok_or(ItemError::Database(
            "cannot update location without id".to_string(),
        ))?;
        let conn = self.conn()?;
        let rows = conn
            .execute(
                "UPDATE locations SET name = ?1, temperature_f = ?2 WHERE id = ?3",
                params![location.name, location.temperature_f, id],
            )
            .map_err(|e| ItemError::Database(e.to_string()))?;

        if rows == 0 {
            return Err(ItemError::LocationNotFound(id));
        }
        Ok(())
    }

    fn remove_location(&self, id: i64) -> Result<(), ItemError> {
        let conn = self.conn()?;
        let rows = conn
            .execute("DELETE FROM locations WHERE id = ?1", params![id])
            .map_err(|e| ItemError::Database(e.to_string()))?;

        if rows == 0 {
            return Err(ItemError::LocationNotFound(id));
        }
        Ok(())
    }
}

impl ShelfRepository for SqliteRepository {
    fn add_shelf(&self, shelf: &Shelf) -> Result<i64, ItemError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO shelves (location_id, name) VALUES (?1, ?2)",
            params![shelf.location_id, shelf.name],
        )
        .map_err(|e| ItemError::Database(e.to_string()))?;
        Ok(conn.last_insert_rowid())
    }

    fn get_shelf(&self, id: i64) -> Result<Shelf, ItemError> {
        let conn = self.conn()?;
        conn.query_row(
            "SELECT id, location_id, name FROM shelves WHERE id = ?1",
            params![id],
            Self::row_to_shelf,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => ItemError::ShelfNotFound(id),
            other => ItemError::Database(other.to_string()),
        })
    }

    fn list_shelves(&self, location_id: i64) -> Result<Vec<Shelf>, ItemError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, location_id, name FROM shelves WHERE location_id = ?1 ORDER BY name",
            )
            .map_err(|e| ItemError::Database(e.to_string()))?;

        let shelves = stmt
            .query_map(params![location_id], Self::row_to_shelf)
            .map_err(|e| ItemError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ItemError::Database(e.to_string()))?;

        Ok(shelves)
    }

    fn list_all_shelves(&self) -> Result<Vec<Shelf>, ItemError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, location_id, name FROM shelves ORDER BY location_id, name")
            .map_err(|e| ItemError::Database(e.to_string()))?;

        let shelves = stmt
            .query_map([], Self::row_to_shelf)
            .map_err(|e| ItemError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ItemError::Database(e.to_string()))?;

        Ok(shelves)
    }

    fn remove_shelf(&self, id: i64) -> Result<(), ItemError> {
        let conn = self.conn()?;
        let rows = conn
            .execute("DELETE FROM shelves WHERE id = ?1", params![id])
            .map_err(|e| ItemError::Database(e.to_string()))?;

        if rows == 0 {
            return Err(ItemError::ShelfNotFound(id));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_repo() -> SqliteRepository {
        SqliteRepository::in_memory().unwrap()
    }

    #[test]
    fn add_and_get() {
        let repo = test_repo();
        let item = GroceryItem::new("Milk", 2, "gallons");
        let id = repo.add(&item).unwrap();
        let fetched = repo.get(id).unwrap();
        assert_eq!(fetched.name, "Milk");
        assert_eq!(fetched.quantity, 2);
        assert!(fetched.location_id.is_none());
    }

    #[test]
    fn list_items() {
        let repo = test_repo();
        repo.add(&GroceryItem::new("Apples", 5, "count")).unwrap();
        repo.add(&GroceryItem::new("Bread", 1, "loaf")).unwrap();
        let items = repo.list().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "Apples"); // ordered by name
    }

    #[test]
    fn update_item() {
        let repo = test_repo();
        let id = repo.add(&GroceryItem::new("Eggs", 12, "count")).unwrap();
        let mut item = repo.get(id).unwrap();
        item.quantity = 6;
        repo.update(&item).unwrap();
        let updated = repo.get(id).unwrap();
        assert_eq!(updated.quantity, 6);
    }

    #[test]
    fn remove_item() {
        let repo = test_repo();
        let id = repo.add(&GroceryItem::new("Butter", 1, "stick")).unwrap();
        repo.remove(id).unwrap();
        assert!(matches!(repo.get(id), Err(ItemError::NotFound(_))));
    }

    #[test]
    fn find_low_stock() {
        let repo = test_repo();
        repo.add(&GroceryItem::new("Rice", 10, "lbs")).unwrap();
        repo.add(&GroceryItem::new("Salt", 1, "box")).unwrap();
        let low = repo.find_low_stock(2).unwrap();
        assert_eq!(low.len(), 1);
        assert_eq!(low[0].name, "Salt");
    }

    #[test]
    fn get_nonexistent() {
        let repo = test_repo();
        assert!(matches!(repo.get(999), Err(ItemError::NotFound(999))));
    }

    #[test]
    fn add_and_get_location() {
        let repo = test_repo();
        let loc = Location::new("Fridge", 37.0);
        let id = repo.add_location(&loc).unwrap();
        let fetched = repo.get_location(id).unwrap();
        assert_eq!(fetched.name, "Fridge");
        assert!((fetched.temperature_f - 37.0).abs() < f64::EPSILON);
    }

    #[test]
    fn list_locations() {
        let repo = test_repo();
        repo.add_location(&Location::new("Fridge", 37.0)).unwrap();
        repo.add_location(&Location::new("Pantry", 68.0)).unwrap();
        let locs = repo.list_locations().unwrap();
        assert_eq!(locs.len(), 2);
        assert_eq!(locs[0].name, "Fridge"); // ordered by name
    }

    #[test]
    fn update_location() {
        let repo = test_repo();
        let id = repo.add_location(&Location::new("Fridge", 37.0)).unwrap();
        let mut loc = repo.get_location(id).unwrap();
        loc.temperature_f = 35.0;
        repo.update_location(&loc).unwrap();
        let updated = repo.get_location(id).unwrap();
        assert!((updated.temperature_f - 35.0).abs() < f64::EPSILON);
    }

    #[test]
    fn remove_location() {
        let repo = test_repo();
        let id = repo.add_location(&Location::new("Freezer", 0.0)).unwrap();
        repo.remove_location(id).unwrap();
        assert!(matches!(
            repo.get_location(id),
            Err(ItemError::LocationNotFound(_))
        ));
    }

    #[test]
    fn get_nonexistent_location() {
        let repo = test_repo();
        assert!(matches!(
            repo.get_location(999),
            Err(ItemError::LocationNotFound(999))
        ));
    }

    #[test]
    fn item_with_location() {
        let repo = test_repo();
        let loc_id = repo.add_location(&Location::new("Fridge", 37.0)).unwrap();
        let mut item = GroceryItem::new("Milk", 2, "gallons");
        item.location_id = Some(loc_id);
        let item_id = repo.add(&item).unwrap();
        let fetched = repo.get(item_id).unwrap();
        assert_eq!(fetched.location_id, Some(loc_id));
    }

    #[test]
    fn add_and_get_shelf() {
        let repo = test_repo();
        let loc_id = repo.add_location(&Location::new("Fridge", 37.0)).unwrap();
        let shelf = Shelf::new(loc_id, "Top Shelf");
        let shelf_id = repo.add_shelf(&shelf).unwrap();
        let fetched = repo.get_shelf(shelf_id).unwrap();
        assert_eq!(fetched.name, "Top Shelf");
        assert_eq!(fetched.location_id, loc_id);
    }

    #[test]
    fn list_shelves_for_location() {
        let repo = test_repo();
        let loc_id = repo.add_location(&Location::new("Fridge", 37.0)).unwrap();
        let loc_id2 = repo.add_location(&Location::new("Pantry", 68.0)).unwrap();
        repo.add_shelf(&Shelf::new(loc_id, "Top")).unwrap();
        repo.add_shelf(&Shelf::new(loc_id, "Bottom")).unwrap();
        repo.add_shelf(&Shelf::new(loc_id2, "Shelf A")).unwrap();
        let shelves = repo.list_shelves(loc_id).unwrap();
        assert_eq!(shelves.len(), 2);
        let all = repo.list_all_shelves().unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn remove_shelf() {
        let repo = test_repo();
        let loc_id = repo.add_location(&Location::new("Fridge", 37.0)).unwrap();
        let shelf_id = repo.add_shelf(&Shelf::new(loc_id, "Top")).unwrap();
        repo.remove_shelf(shelf_id).unwrap();
        assert!(matches!(
            repo.get_shelf(shelf_id),
            Err(ItemError::ShelfNotFound(_))
        ));
    }

    #[test]
    fn get_nonexistent_shelf() {
        let repo = test_repo();
        assert!(matches!(
            repo.get_shelf(999),
            Err(ItemError::ShelfNotFound(999))
        ));
    }

    #[test]
    fn item_with_shelf() {
        let repo = test_repo();
        let loc_id = repo.add_location(&Location::new("Fridge", 37.0)).unwrap();
        let shelf_id = repo.add_shelf(&Shelf::new(loc_id, "Top")).unwrap();
        let mut item = GroceryItem::new("Milk", 2, "gallons");
        item.location_id = Some(loc_id);
        item.shelf_id = Some(shelf_id);
        let item_id = repo.add(&item).unwrap();
        let fetched = repo.get(item_id).unwrap();
        assert_eq!(fetched.shelf_id, Some(shelf_id));
        assert_eq!(fetched.location_id, Some(loc_id));
    }

    #[test]
    fn delete_shelf_clears_item_shelf_reference() {
        let repo = test_repo();
        let loc_id = repo.add_location(&Location::new("Fridge", 37.0)).unwrap();
        let shelf_id = repo.add_shelf(&Shelf::new(loc_id, "Top")).unwrap();
        let mut item = GroceryItem::new("Milk", 2, "gallons");
        item.location_id = Some(loc_id);
        item.shelf_id = Some(shelf_id);
        let item_id = repo.add(&item).unwrap();
        repo.remove_shelf(shelf_id).unwrap();
        let fetched = repo.get(item_id).unwrap();
        assert!(fetched.shelf_id.is_none());
        assert_eq!(fetched.location_id, Some(loc_id)); // location preserved
    }

    #[test]
    fn delete_location_cascades_shelves() {
        let repo = test_repo();
        let loc_id = repo.add_location(&Location::new("Fridge", 37.0)).unwrap();
        let shelf_id = repo.add_shelf(&Shelf::new(loc_id, "Top")).unwrap();
        repo.remove_location(loc_id).unwrap();
        assert!(matches!(
            repo.get_shelf(shelf_id),
            Err(ItemError::ShelfNotFound(_))
        ));
    }

    #[test]
    fn delete_location_clears_item_reference() {
        let repo = test_repo();
        let loc_id = repo.add_location(&Location::new("Fridge", 37.0)).unwrap();
        let mut item = GroceryItem::new("Milk", 2, "gallons");
        item.location_id = Some(loc_id);
        let item_id = repo.add(&item).unwrap();
        repo.remove_location(loc_id).unwrap();
        let fetched = repo.get(item_id).unwrap();
        assert!(fetched.location_id.is_none());
    }
}
