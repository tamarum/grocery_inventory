use crate::config::Config;
use crate::item::{GroceryItem, ItemError, ItemRepository};
use crate::location::{Location, LocationRepository, Shelf, ShelfRepository};
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

impl<R: ItemRepository + LocationRepository, S: ShoppingListGenerator> App<R, S> {
    pub fn add_location(&self, location: &Location) -> Result<i64, ItemError> {
        self.repo.add_location(location)
    }

    pub fn get_location(&self, id: i64) -> Result<Location, ItemError> {
        self.repo.get_location(id)
    }

    pub fn list_locations(&self) -> Result<Vec<Location>, ItemError> {
        self.repo.list_locations()
    }

    pub fn update_location(&self, location: &Location) -> Result<(), ItemError> {
        self.repo.update_location(location)
    }

    pub fn remove_location(&self, id: i64) -> Result<(), ItemError> {
        self.repo.remove_location(id)
    }
}

impl<R: ItemRepository + LocationRepository + ShelfRepository, S: ShoppingListGenerator> App<R, S> {
    pub fn add_shelf(&self, shelf: &Shelf) -> Result<i64, ItemError> {
        // Validate that the location exists
        self.repo.get_location(shelf.location_id)?;
        self.repo.add_shelf(shelf)
    }

    pub fn get_shelf(&self, id: i64) -> Result<Shelf, ItemError> {
        self.repo.get_shelf(id)
    }

    pub fn list_shelves(&self, location_id: i64) -> Result<Vec<Shelf>, ItemError> {
        self.repo.list_shelves(location_id)
    }

    pub fn list_all_shelves(&self) -> Result<Vec<Shelf>, ItemError> {
        self.repo.list_all_shelves()
    }

    pub fn remove_shelf(&self, id: i64) -> Result<(), ItemError> {
        self.repo.remove_shelf(id)
    }

    /// Add an item assigned to a shelf, auto-setting location_id from the shelf's parent.
    pub fn add_item_to_shelf(
        &self,
        item: &mut GroceryItem,
        shelf_id: i64,
    ) -> Result<i64, ItemError> {
        let shelf = self.repo.get_shelf(shelf_id)?;
        item.shelf_id = Some(shelf_id);
        item.location_id = Some(shelf.location_id);
        self.repo.add(item)
    }

    /// Assign a shelf to an existing item, auto-setting location_id from the shelf's parent.
    pub fn assign_shelf_to_item(
        &self,
        item: &mut GroceryItem,
        shelf_id: i64,
    ) -> Result<(), ItemError> {
        let shelf = self.repo.get_shelf(shelf_id)?;
        item.shelf_id = Some(shelf_id);
        item.location_id = Some(shelf.location_id);
        self.repo.update(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::SqliteRepository;
    use crate::location::{Location, Shelf};
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
            anthropic: Default::default(),
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

    #[test]
    fn location_crud_workflow() {
        let app = test_app();
        let id = app.add_location(&Location::new("Fridge", 37.0)).unwrap();
        let loc = app.get_location(id).unwrap();
        assert_eq!(loc.name, "Fridge");

        let locs = app.list_locations().unwrap();
        assert_eq!(locs.len(), 1);

        app.remove_location(id).unwrap();
        assert!(app.list_locations().unwrap().is_empty());
    }

    #[test]
    fn shelf_crud_workflow() {
        let app = test_app();
        let loc_id = app.add_location(&Location::new("Fridge", 37.0)).unwrap();
        let shelf_id = app.add_shelf(&Shelf::new(loc_id, "Top Shelf")).unwrap();
        let shelf = app.get_shelf(shelf_id).unwrap();
        assert_eq!(shelf.name, "Top Shelf");
        assert_eq!(shelf.location_id, loc_id);

        let shelves = app.list_shelves(loc_id).unwrap();
        assert_eq!(shelves.len(), 1);

        app.remove_shelf(shelf_id).unwrap();
        assert!(app.list_shelves(loc_id).unwrap().is_empty());
    }

    #[test]
    fn add_item_to_shelf_sets_location() {
        let app = test_app();
        let loc_id = app.add_location(&Location::new("Fridge", 37.0)).unwrap();
        let shelf_id = app.add_shelf(&Shelf::new(loc_id, "Top")).unwrap();

        let mut item = GroceryItem::new("Milk", 2, "gallons");
        let item_id = app.add_item_to_shelf(&mut item, shelf_id).unwrap();

        let fetched = app.get_item(item_id).unwrap();
        assert_eq!(fetched.shelf_id, Some(shelf_id));
        assert_eq!(fetched.location_id, Some(loc_id));
    }

    #[test]
    fn assign_shelf_to_item_updates_location() {
        let app = test_app();
        let loc_id = app.add_location(&Location::new("Fridge", 37.0)).unwrap();
        let shelf_id = app.add_shelf(&Shelf::new(loc_id, "Bottom")).unwrap();

        let item_id = app
            .add_item(&GroceryItem::new("Eggs", 12, "count"))
            .unwrap();
        let mut item = app.get_item(item_id).unwrap();
        app.assign_shelf_to_item(&mut item, shelf_id).unwrap();

        let fetched = app.get_item(item_id).unwrap();
        assert_eq!(fetched.shelf_id, Some(shelf_id));
        assert_eq!(fetched.location_id, Some(loc_id));
    }
}
