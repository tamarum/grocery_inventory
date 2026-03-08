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
    },
    /// Remove an item from inventory
    Remove {
        /// Item ID
        id: i64,
    },
    /// Generate a shopping list
    Shop,
    /// Start the web interface (requires --features web)
    Web,
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
        } => {
            let mut item = GroceryItem::new(name, quantity, unit);
            item.category = category;
            item.minimum_stock = min_stock;
            let id = app.add_item(&item)?;
            println!("Added '{}' with id {id}", item.name);
        }
        Commands::List => {
            let items = app.list_items()?;
            if items.is_empty() {
                println!("Inventory is empty.");
            } else {
                println!(
                    "{:<5} {:<20} {:<8} {:<10} {:<15}",
                    "ID", "Name", "Qty", "Unit", "Category"
                );
                println!("{:-<60}", "");
                for item in items {
                    println!(
                        "{:<5} {:<20} {:<8} {:<10} {:<15}",
                        item.id.unwrap_or(0),
                        item.name,
                        item.quantity,
                        item.unit,
                        item.category.as_deref().unwrap_or("-"),
                    );
                }
            }
        }
        Commands::Update {
            id,
            quantity,
            name,
            category,
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
