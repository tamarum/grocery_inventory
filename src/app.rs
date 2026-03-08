use crate::config::Config;
use crate::item::{GroceryItem, ItemError, ItemRepository};
use crate::shopping::{ShoppingList, ShoppingListGenerator};

pub struct App<R: ItemRepository, S: ShoppingListGenerator> {
    pub repo: R,
    pub shopping: S,
    pub config: Config,
}

impl<R: ItemRepository, S: ShoppingListGenerator> App<R, S> {
    pub fn new(repo: R, shopping: S, config: Config) -> Self {
        Self {
            repo,
            shopping,
            config,
        }
    }

    pub fn add_item(&self, item: &GroceryItem) -> Result<i64, ItemError> {
        self.repo.add(item)
    }

    pub fn get_item(&self, id: i64) -> Result<GroceryItem, ItemError> {
        self.repo.get(id)
    }

    pub fn list_items(&self) -> Result<Vec<GroceryItem>, ItemError> {
        self.repo.list()
    }

    pub fn update_item(&self, item: &GroceryItem) -> Result<(), ItemError> {
        self.repo.update(item)
    }

    pub fn remove_item(&self, id: i64) -> Result<(), ItemError> {
        self.repo.remove(id)
    }

    pub fn generate_shopping_list(&self) -> Result<ShoppingList, ItemError> {
        self.shopping.generate(
            &self.repo,
            self.config.shopping.low_stock_threshold,
            self.config.shopping.include_out_of_stock,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::SqliteRepository;
    use crate::shopping::DefaultShoppingListGenerator;
    use std::path::PathBuf;

    fn test_app() -> App<SqliteRepository, DefaultShoppingListGenerator> {
        let repo = SqliteRepository::in_memory().unwrap();
        let shopping = DefaultShoppingListGenerator;
        let config = Config {
            database: crate::config::DatabaseConfig {
                path: PathBuf::from(":memory:"),
            },
            web: Default::default(),
            shopping: Default::default(),
        };
        App::new(repo, shopping, config)
    }

    #[test]
    fn crud_workflow() {
        let app = test_app();

        let id = app
            .add_item(&GroceryItem::new("Milk", 2, "gallons"))
            .unwrap();
        let item = app.get_item(id).unwrap();
        assert_eq!(item.name, "Milk");

        let items = app.list_items().unwrap();
        assert_eq!(items.len(), 1);

        let mut updated = item;
        updated.quantity = 1;
        app.update_item(&updated).unwrap();
        assert_eq!(app.get_item(id).unwrap().quantity, 1);

        app.remove_item(id).unwrap();
        assert!(app.list_items().unwrap().is_empty());
    }

    #[test]
    fn shopping_list_integration() {
        let app = test_app();
        app.add_item(&GroceryItem::new("Rice", 10, "lbs")).unwrap();
        app.add_item(&GroceryItem::new("Salt", 1, "box")).unwrap();

        let list = app.generate_shopping_list().unwrap();
        assert_eq!(list.len(), 1);
    }
}
