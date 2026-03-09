use std::path::PathBuf;
#[cfg(feature = "web")]
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use grocery_inventory::app::App;
use grocery_inventory::config::Config;
use grocery_inventory::db::SqliteRepository;
use grocery_inventory::item::GroceryItem;
use grocery_inventory::location::{Location, Shelf};
use grocery_inventory::shopping::DefaultShoppingListGenerator;

#[derive(Parser)]
#[command(name = "grocery", about = "Home grocery inventory manager")]
struct Cli {
    /// Path to config file
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new item to inventory
    Add {
        /// Item name
        name: String,
        /// Quantity
        #[arg(short, long, default_value_t = 1)]
        quantity: u32,
        /// Unit of measurement
        #[arg(short, long, default_value = "count")]
        unit: String,
        /// Category
        #[arg(long)]
        category: Option<String>,
        /// Minimum stock level
        #[arg(long, default_value_t = 0)]
        min_stock: u32,
        /// Storage location ID
        #[arg(long)]
        location: Option<i64>,
        /// Shelf ID (auto-sets location from shelf's parent)
        #[arg(long)]
        shelf: Option<i64>,
    },
    /// List all items in inventory
    List,
    /// Update an existing item
    Update {
        /// Item ID
        id: i64,
        /// New quantity
        #[arg(short, long)]
        quantity: Option<u32>,
        /// New name
        #[arg(short, long)]
        name: Option<String>,
        /// New category
        #[arg(long)]
        category: Option<String>,
        /// Storage location ID (use 0 to clear)
        #[arg(long)]
        location: Option<i64>,
        /// Shelf ID (auto-sets location from shelf's parent; use 0 to clear)
        #[arg(long)]
        shelf: Option<i64>,
    },
    /// Remove an item from inventory
    Remove {
        /// Item ID
        id: i64,
    },
    /// Generate a shopping list
    Shop,
    /// Manage storage locations
    Location {
        #[command(subcommand)]
        action: LocationCommands,
    },
    /// Manage shelves within locations
    Shelf {
        #[command(subcommand)]
        action: ShelfCommands,
    },
    /// Start the web interface (requires --features web)
    Web,
}

#[derive(Subcommand)]
enum LocationCommands {
    /// Add a new storage location
    Add {
        /// Location name (e.g., Fridge, Pantry, Freezer)
        name: String,
        /// Storage temperature in Fahrenheit
        #[arg(short, long)]
        temp: f64,
    },
    /// List all storage locations
    List,
    /// Remove a storage location
    Remove {
        /// Location ID
        id: i64,
    },
}

#[derive(Subcommand)]
enum ShelfCommands {
    /// Add a shelf to a location
    Add {
        /// Location ID
        location_id: i64,
        /// Shelf name
        #[arg(short, long)]
        name: String,
    },
    /// List shelves for a location
    List {
        /// Location ID
        location_id: i64,
    },
    /// Remove a shelf
    Remove {
        /// Shelf ID
        id: i64,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    let config = Config::from_file(&cli.config)
        .context("failed to load config (copy config.example.toml to config.toml)")?;

    let repo = SqliteRepository::new(&config.database.path).context("failed to open database")?;
    let shopping = DefaultShoppingListGenerator;
    let app = App::new(repo, shopping, config.clone());

    match cli.command {
        Commands::Add {
            name,
            quantity,
            unit,
            category,
            min_stock,
            location,
            shelf,
        } => {
            let mut item = GroceryItem::new(name, quantity, unit);
            item.category = category;
            item.minimum_stock = min_stock;
            item.location_id = location;
            let id = if let Some(sid) = shelf {
                app.add_item_to_shelf(&mut item, sid)?
            } else {
                app.add_item(&item)?
            };
            println!("Added '{}' with id {id}", item.name);
        }
        Commands::List => {
            let items = app.list_items()?;
            let locations = app.list_locations()?;
            let shelves = app.list_all_shelves()?;
            if items.is_empty() {
                println!("Inventory is empty.");
            } else {
                println!(
                    "{:<5} {:<20} {:<8} {:<10} {:<15} {:<15} {:<15}",
                    "ID", "Name", "Qty", "Unit", "Category", "Location", "Shelf"
                );
                println!("{:-<90}", "");
                for item in items {
                    let loc_name = item
                        .location_id
                        .and_then(|lid| locations.iter().find(|l| l.id == Some(lid)))
                        .map(|l| l.name.as_str())
                        .unwrap_or("-");
                    let shelf_name = item
                        .shelf_id
                        .and_then(|sid| shelves.iter().find(|s| s.id == Some(sid)))
                        .map(|s| s.name.as_str())
                        .unwrap_or("-");
                    println!(
                        "{:<5} {:<20} {:<8} {:<10} {:<15} {:<15} {:<15}",
                        item.id.unwrap_or(0),
                        item.name,
                        item.quantity,
                        item.unit,
                        item.category.as_deref().unwrap_or("-"),
                        loc_name,
                        shelf_name,
                    );
                }
            }
        }
        Commands::Update {
            id,
            quantity,
            name,
            category,
            location,
            shelf,
        } => {
            let mut item = app.get_item(id)?;
            if let Some(q) = quantity {
                item.quantity = q;
            }
            if let Some(n) = name {
                item.name = n;
            }
            if let Some(c) = category {
                item.category = Some(c);
            }
            if let Some(lid) = location {
                item.location_id = if lid == 0 { None } else { Some(lid) };
            }
            if let Some(sid) = shelf {
                if sid == 0 {
                    item.shelf_id = None;
                } else {
                    app.assign_shelf_to_item(&mut item, sid)?;
                    println!("Updated item {id}");
                    return Ok(());
                }
            }
            app.update_item(&item)?;
            println!("Updated item {id}");
        }
        Commands::Remove { id } => {
            app.remove_item(id)?;
            println!("Removed item {id}");
        }
        Commands::Shop => {
            let list = app.generate_shopping_list()?;
            print!("{list}");
        }
        Commands::Location { action } => match action {
            LocationCommands::Add { name, temp } => {
                let loc = Location::new(&name, temp);
                let id = app.add_location(&loc)?;
                println!("Added location '{name}' ({}°F) with id {id}", temp);
            }
            LocationCommands::List => {
                let locations = app.list_locations()?;
                if locations.is_empty() {
                    println!("No locations defined.");
                } else {
                    println!("{:<5} {:<20} {:<10}", "ID", "Name", "Temp (°F)");
                    println!("{:-<37}", "");
                    for loc in locations {
                        println!(
                            "{:<5} {:<20} {:<10.1}",
                            loc.id.unwrap_or(0),
                            loc.name,
                            loc.temperature_f,
                        );
                    }
                }
            }
            LocationCommands::Remove { id } => {
                app.remove_location(id)?;
                println!("Removed location {id}");
            }
        },
        Commands::Shelf { action } => match action {
            ShelfCommands::Add { location_id, name } => {
                let shelf = Shelf::new(location_id, &name);
                let id = app.add_shelf(&shelf)?;
                println!("Added shelf '{name}' to location {location_id} with id {id}");
            }
            ShelfCommands::List { location_id } => {
                let shelves = app.list_shelves(location_id)?;
                if shelves.is_empty() {
                    println!("No shelves for location {location_id}.");
                } else {
                    println!("{:<5} {:<20} {:<10}", "ID", "Name", "Location");
                    println!("{:-<37}", "");
                    for shelf in shelves {
                        println!(
                            "{:<5} {:<20} {:<10}",
                            shelf.id.unwrap_or(0),
                            shelf.name,
                            shelf.location_id,
                        );
                    }
                }
            }
            ShelfCommands::Remove { id } => {
                app.remove_shelf(id)?;
                println!("Removed shelf {id}");
            }
        },
        Commands::Web => {
            #[cfg(feature = "web")]
            {
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(async {
                    let app = Arc::new(app);
                    let router = grocery_inventory::web::routes::create_router(app.clone());
                    let addr = format!("{}:{}", app.config.web.host, app.config.web.port);
                    println!("Starting web server at http://{addr}");
                    let listener = tokio::net::TcpListener::bind(&addr).await?;
                    axum::serve(listener, router).await?;
                    Ok::<_, anyhow::Error>(())
                })?;
            }
            #[cfg(not(feature = "web"))]
            {
                anyhow::bail!("Web feature not enabled. Rebuild with: cargo build --features web");
            }
        }
    }

    Ok(())
}
