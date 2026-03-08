use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection};

use crate::item::{GroceryItem, ItemError, ItemRepository};

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
        self.conn()
            .map_err(|e| ItemError::Database(e.to_string()))?
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS grocery_items (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    quantity INTEGER NOT NULL DEFAULT 0,
                    unit TEXT NOT NULL DEFAULT '',
                    category TEXT,
                    expiration_date TEXT,
                    minimum_stock INTEGER NOT NULL DEFAULT 0
                );",
            )
            .map_err(|e| ItemError::Database(e.to_string()))?;
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
            "INSERT INTO grocery_items (name, quantity, unit, category, expiration_date, minimum_stock)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                item.name,
                item.quantity,
                item.unit,
                item.category,
                expiration,
                item.minimum_stock,
            ],
        )
        .map_err(|e| ItemError::Database(e.to_string()))?;
        Ok(conn.last_insert_rowid())
    }

    fn get(&self, id: i64) -> Result<GroceryItem, ItemError> {
        let conn = self.conn()?;
        conn.query_row(
            "SELECT id, name, quantity, unit, category, expiration_date, minimum_stock
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
                "SELECT id, name, quantity, unit, category, expiration_date, minimum_stock
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
                     expiration_date = ?5, minimum_stock = ?6
                 WHERE id = ?7",
                params![
                    item.name,
                    item.quantity,
                    item.unit,
                    item.category,
                    expiration,
                    item.minimum_stock,
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
                "SELECT id, name, quantity, unit, category, expiration_date, minimum_stock
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
}
