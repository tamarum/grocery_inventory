use crate::item::{ItemError, ItemRepository};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ShoppingEntry {
    pub name: String,
    pub current_quantity: u32,
    pub suggested_quantity: u32,
    pub unit: String,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShoppingList {
    pub entries: Vec<ShoppingEntry>,
}

impl ShoppingList {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl std::fmt::Display for ShoppingList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.entries.is_empty() {
            return writeln!(f, "Shopping list is empty - you're fully stocked!");
        }
        writeln!(f, "Shopping List ({} items):", self.entries.len())?;
        writeln!(f, "{:-<40}", "")?;
        for entry in &self.entries {
            writeln!(
                f,
                "  [ ] {} - need {} {} (have {})",
                entry.name, entry.suggested_quantity, entry.unit, entry.current_quantity
            )?;
        }
        Ok(())
    }
}

pub trait ShoppingListGenerator: Send + Sync {
    fn generate(
        &self,
        repo: &dyn ItemRepository,
        threshold: u32,
        include_out_of_stock: bool,
    ) -> Result<ShoppingList, ItemError>;
}

pub struct DefaultShoppingListGenerator;

impl ShoppingListGenerator for DefaultShoppingListGenerator {
    fn generate(
        &self,
        repo: &dyn ItemRepository,
        threshold: u32,
        include_out_of_stock: bool,
    ) -> Result<ShoppingList, ItemError> {
        let low_stock_items = repo.find_low_stock(threshold)?;

        let entries = low_stock_items
            .into_iter()
            .filter(|item| include_out_of_stock || item.quantity > 0)
            .map(|item| {
                let target = std::cmp::max(item.minimum_stock, threshold + 1);
                let suggested = target.saturating_sub(item.quantity);
                ShoppingEntry {
                    name: item.name,
                    current_quantity: item.quantity,
                    suggested_quantity: suggested,
                    unit: item.unit,
                    category: item.category,
                }
            })
            .collect();

        Ok(ShoppingList { entries })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::SqliteRepository;
    use crate::item::GroceryItem;

    #[test]
    fn generate_shopping_list() {
        let repo = SqliteRepository::in_memory().unwrap();
        repo.add(&GroceryItem::new("Rice", 10, "lbs")).unwrap();
        repo.add(&GroceryItem::new("Salt", 1, "box")).unwrap();

        let generator = DefaultShoppingListGenerator;
        let list = generator.generate(&repo, 2, true).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list.entries[0].name, "Salt");
    }

    #[test]
    fn empty_when_fully_stocked() {
        let repo = SqliteRepository::in_memory().unwrap();
        repo.add(&GroceryItem::new("Rice", 10, "lbs")).unwrap();

        let generator = DefaultShoppingListGenerator;
        let list = generator.generate(&repo, 2, true).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn exclude_out_of_stock() {
        let repo = SqliteRepository::in_memory().unwrap();
        repo.add(&GroceryItem::new("Gone", 0, "box")).unwrap();

        let generator = DefaultShoppingListGenerator;
        let list = generator.generate(&repo, 2, false).unwrap();
        assert!(list.is_empty());

        let list = generator.generate(&repo, 2, true).unwrap();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn display_format() {
        let list = ShoppingList {
            entries: vec![ShoppingEntry {
                name: "Milk".to_string(),
                current_quantity: 0,
                suggested_quantity: 3,
                unit: "gallons".to_string(),
                category: None,
            }],
        };
        let output = format!("{list}");
        assert!(output.contains("Milk"));
        assert!(output.contains("need 3 gallons"));
    }
}
