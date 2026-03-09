// Auto-categorization of grocery items based on name matching.
// Uses a built-in lookup table of common grocery items mapped to categories.
// Matching is case-insensitive and checks if any keyword appears in the item name.
const CATEGORY_RULES: &[(&[&str], &str)] = &[
    // Dairy
    (
        &[
            "milk",
            "cream",
            "yogurt",
            "yoghurt",
            "butter",
            "cheese",
            "sour cream",
            "cottage cheese",
            "whipping cream",
            "half and half",
            "half & half",
            "creamer",
            "kefir",
        ],
        "Dairy",
    ),
    // Produce - Fruits
    (
        &[
            "apple",
            "banana",
            "orange",
            "grape",
            "strawberry",
            "blueberry",
            "raspberry",
            "blackberry",
            "cherry",
            "peach",
            "pear",
            "plum",
            "mango",
            "pineapple",
            "watermelon",
            "cantaloupe",
            "honeydew",
            "kiwi",
            "lemon",
            "lime",
            "grapefruit",
            "avocado",
            "pomegranate",
            "fig",
            "date",
            "coconut",
            "papaya",
            "tangerine",
            "clementine",
            "nectarine",
            "apricot",
            "cranberry",
        ],
        "Produce - Fruits",
    ),
    // Produce - Vegetables
    (
        &[
            "lettuce",
            "spinach",
            "kale",
            "arugula",
            "cabbage",
            "broccoli",
            "cauliflower",
            "carrot",
            "celery",
            "cucumber",
            "tomato",
            "pepper",
            "onion",
            "garlic",
            "potato",
            "sweet potato",
            "yam",
            "corn",
            "peas",
            "green bean",
            "asparagus",
            "zucchini",
            "squash",
            "eggplant",
            "mushroom",
            "radish",
            "beet",
            "turnip",
            "parsnip",
            "artichoke",
            "brussels sprout",
            "bok choy",
            "scallion",
            "leek",
            "shallot",
            "ginger",
            "jalapeño",
            "jalapeno",
            "cilantro",
            "parsley",
            "basil",
            "dill",
            "mint",
            "rosemary",
            "thyme",
            "oregano",
            "chive",
        ],
        "Produce - Vegetables",
    ),
    // Meat & Poultry
    (
        &[
            "chicken",
            "beef",
            "pork",
            "turkey",
            "lamb",
            "steak",
            "ground meat",
            "ground beef",
            "ground turkey",
            "ground pork",
            "bacon",
            "sausage",
            "ham",
            "salami",
            "pepperoni",
            "hot dog",
            "brisket",
            "ribs",
            "roast",
            "veal",
            "duck",
            "goose",
            "venison",
            "bison",
        ],
        "Meat & Poultry",
    ),
    // Seafood
    (
        &[
            "salmon",
            "tuna",
            "shrimp",
            "crab",
            "lobster",
            "cod",
            "tilapia",
            "halibut",
            "trout",
            "catfish",
            "sardine",
            "anchovy",
            "clam",
            "mussel",
            "oyster",
            "scallop",
            "squid",
            "calamari",
            "octopus",
            "fish",
            "mahi",
            "swordfish",
            "bass",
            "snapper",
        ],
        "Seafood",
    ),
    // Eggs
    (&["egg"], "Eggs"),
    // Bread & Bakery
    (
        &[
            "bread",
            "bagel",
            "muffin",
            "croissant",
            "tortilla",
            "pita",
            "naan",
            "roll",
            "bun",
            "biscuit",
            "english muffin",
            "flatbread",
            "ciabatta",
            "sourdough",
            "cornbread",
            "cake",
            "pie",
            "pastry",
            "donut",
            "doughnut",
            "danish",
            "scone",
        ],
        "Bread & Bakery",
    ),
    // Grains & Pasta
    (
        &[
            "rice",
            "pasta",
            "noodle",
            "spaghetti",
            "penne",
            "macaroni",
            "fettuccine",
            "linguine",
            "ramen",
            "udon",
            "couscous",
            "quinoa",
            "barley",
            "oat",
            "oatmeal",
            "cereal",
            "granola",
            "flour",
            "cornmeal",
            "grits",
            "polenta",
            "bulgur",
            "farro",
        ],
        "Grains & Pasta",
    ),
    // Canned & Jarred
    (
        &[
            "canned",
            "can of",
            "soup",
            "broth",
            "stock",
            "tomato sauce",
            "pasta sauce",
            "marinara",
            "salsa",
            "pickle",
            "olive",
            "jam",
            "jelly",
            "preserves",
            "peanut butter",
            "almond butter",
            "nutella",
            "applesauce",
        ],
        "Canned & Jarred",
    ),
    // Frozen
    (
        &[
            "frozen",
            "ice cream",
            "popsicle",
            "frozen pizza",
            "frozen dinner",
            "frozen vegetable",
            "frozen fruit",
            "gelato",
            "sorbet",
            "sherbet",
        ],
        "Frozen",
    ),
    // Snacks
    (
        &[
            "chip",
            "cracker",
            "pretzel",
            "popcorn",
            "nut",
            "almond",
            "cashew",
            "peanut",
            "walnut",
            "pecan",
            "pistachio",
            "trail mix",
            "granola bar",
            "protein bar",
            "cookie",
            "candy",
            "chocolate",
            "gummy",
            "jerky",
            "dried fruit",
            "raisin",
        ],
        "Snacks",
    ),
    // Beverages
    (
        &[
            "water",
            "juice",
            "soda",
            "pop",
            "cola",
            "coffee",
            "tea",
            "kombucha",
            "lemonade",
            "sports drink",
            "energy drink",
            "sparkling",
            "seltzer",
            "wine",
            "beer",
            "liquor",
            "vodka",
            "whiskey",
            "rum",
            "gin",
            "tequila",
            "champagne",
            "cider",
        ],
        "Beverages",
    ),
    // Condiments & Sauces
    (
        &[
            "ketchup",
            "mustard",
            "mayonnaise",
            "mayo",
            "soy sauce",
            "hot sauce",
            "bbq sauce",
            "barbecue sauce",
            "vinegar",
            "dressing",
            "ranch",
            "sriracha",
            "worcestershire",
            "teriyaki",
            "hoisin",
            "fish sauce",
            "tahini",
            "hummus",
            "guacamole",
        ],
        "Condiments & Sauces",
    ),
    // Spices & Seasonings
    (
        &[
            "salt",
            "pepper",
            "cinnamon",
            "cumin",
            "paprika",
            "turmeric",
            "chili powder",
            "garlic powder",
            "onion powder",
            "cayenne",
            "nutmeg",
            "clove",
            "allspice",
            "curry",
            "seasoning",
            "spice",
            "vanilla extract",
            "baking powder",
            "baking soda",
            "yeast",
        ],
        "Spices & Seasonings",
    ),
    // Oils & Vinegars
    (
        &[
            "olive oil",
            "vegetable oil",
            "canola oil",
            "coconut oil",
            "sesame oil",
            "cooking spray",
            "shortening",
            "lard",
        ],
        "Oils & Cooking Fats",
    ),
    // Deli
    (
        &[
            "deli",
            "lunch meat",
            "cold cut",
            "prosciutto",
            "pastrami",
            "roast beef",
            "smoked",
        ],
        "Deli",
    ),
    // Baby & Infant
    (&["baby food", "formula", "diaper", "baby"], "Baby & Infant"),
    // Pet
    (
        &[
            "dog food",
            "cat food",
            "pet food",
            "cat litter",
            "dog treat",
            "cat treat",
            "pet",
        ],
        "Pet Supplies",
    ),
    // Household
    (
        &[
            "paper towel",
            "toilet paper",
            "tissue",
            "napkin",
            "trash bag",
            "garbage bag",
            "aluminum foil",
            "plastic wrap",
            "parchment",
            "wax paper",
            "sponge",
            "dish soap",
            "laundry detergent",
            "bleach",
            "cleaner",
            "disinfectant",
        ],
        "Household",
    ),
    // Personal Care
    (
        &[
            "shampoo",
            "conditioner",
            "soap",
            "body wash",
            "toothpaste",
            "toothbrush",
            "deodorant",
            "lotion",
            "sunscreen",
            "razor",
            "floss",
        ],
        "Personal Care",
    ),
    // Sugar & Sweeteners
    (
        &[
            "sugar",
            "honey",
            "maple syrup",
            "agave",
            "stevia",
            "molasses",
            "corn syrup",
        ],
        "Sweeteners",
    ),
];

// Shelf life rules: (keywords, days until expiration).
// More specific matches come first to take priority.
const SHELF_LIFE_RULES: &[(&[&str], u32)] = &[
    // Very short shelf life (1-5 days)
    (&["fresh fish", "fresh salmon", "fresh tuna", "sashimi"], 2),
    (
        &[
            "ground beef",
            "ground turkey",
            "ground pork",
            "ground meat",
            "ground chicken",
        ],
        2,
    ),
    (
        &["deli", "lunch meat", "cold cut", "prosciutto", "pastrami"],
        5,
    ),
    // Short shelf life (5-14 days)
    (
        &[
            "milk",
            "cream",
            "half and half",
            "half & half",
            "creamer",
            "kefir",
        ],
        10,
    ),
    (&["yogurt", "yoghurt", "sour cream", "cottage cheese"], 14),
    (
        &[
            "bread",
            "bagel",
            "muffin",
            "croissant",
            "bun",
            "roll",
            "english muffin",
            "naan",
            "pita",
            "tortilla",
        ],
        7,
    ),
    (
        &["chicken", "turkey", "pork", "beef", "steak", "lamb", "veal"],
        5,
    ),
    (
        &[
            "salmon", "tuna", "shrimp", "cod", "tilapia", "halibut", "trout", "catfish", "fish",
            "crab", "lobster", "scallop", "clam", "mussel", "oyster",
        ],
        3,
    ),
    (&["bacon", "sausage", "hot dog"], 7),
    (&["egg"], 28),
    (&["lettuce", "spinach", "kale", "arugula", "baby greens"], 7),
    (
        &[
            "berr",
            "strawberry",
            "blueberry",
            "raspberry",
            "blackberry",
            "cranberry",
        ],
        7,
    ),
    (&["avocado"], 5),
    (&["banana"], 7),
    (&["tomato"], 7),
    (
        &["cilantro", "parsley", "basil", "dill", "mint", "chive"],
        7,
    ),
    // Medium shelf life (2-6 weeks)
    (&["butter", "margarine"], 30),
    (
        &[
            "cheese",
            "cheddar",
            "mozzarella",
            "parmesan",
            "swiss",
            "gouda",
            "brie",
            "feta",
        ],
        28,
    ),
    (&["apple", "pear"], 28),
    (
        &[
            "orange",
            "grapefruit",
            "tangerine",
            "clementine",
            "lemon",
            "lime",
        ],
        21,
    ),
    (
        &["carrot", "celery", "beet", "turnip", "parsnip", "radish"],
        21,
    ),
    (&["broccoli", "cauliflower", "brussels sprout"], 10),
    (
        &["pepper", "cucumber", "zucchini", "squash", "eggplant"],
        10,
    ),
    (&["onion", "shallot", "leek", "scallion"], 30),
    (&["potato", "sweet potato", "yam"], 30),
    (&["garlic", "ginger"], 30),
    (&["mushroom"], 10),
    (&["corn"], 5),
    (
        &[
            "grape",
            "cherry",
            "peach",
            "plum",
            "nectarine",
            "apricot",
            "kiwi",
            "mango",
        ],
        7,
    ),
    (
        &[
            "watermelon",
            "cantaloupe",
            "honeydew",
            "pineapple",
            "papaya",
        ],
        7,
    ),
    (&["ham"], 7),
    (&["hummus", "guacamole"], 7),
    (&["salsa"], 14),
    // Long shelf life (months)
    (&["juice"], 10),
    (&["jam", "jelly", "preserves"], 180),
    (&["ketchup", "mustard", "mayonnaise", "mayo"], 180),
    (
        &[
            "soy sauce",
            "hot sauce",
            "bbq sauce",
            "barbecue sauce",
            "sriracha",
            "worcestershire",
            "teriyaki",
            "hoisin",
            "fish sauce",
        ],
        365,
    ),
    (&["dressing", "ranch"], 60),
    (&["pickle", "olive"], 365),
    (
        &["peanut butter", "almond butter", "nutella", "tahini"],
        180,
    ),
    (&["maple syrup"], 365),
    (&["honey"], 730),
    // Very long shelf life (1-2+ years)
    (
        &[
            "canned",
            "can of",
            "soup",
            "broth",
            "stock",
            "tomato sauce",
            "pasta sauce",
            "marinara",
        ],
        730,
    ),
    (
        &[
            "rice",
            "pasta",
            "noodle",
            "spaghetti",
            "penne",
            "macaroni",
            "fettuccine",
            "linguine",
            "ramen",
            "udon",
            "couscous",
            "quinoa",
            "barley",
            "farro",
            "bulgur",
        ],
        730,
    ),
    (&["flour", "cornmeal", "grits", "polenta"], 365),
    (&["oat", "oatmeal", "cereal", "granola"], 365),
    (&["sugar", "salt"], 1825),
    (
        &[
            "spice",
            "cinnamon",
            "cumin",
            "paprika",
            "turmeric",
            "chili powder",
            "garlic powder",
            "onion powder",
            "cayenne",
            "nutmeg",
            "clove",
            "allspice",
            "curry",
            "seasoning",
            "pepper",
        ],
        730,
    ),
    (&["baking powder", "baking soda", "yeast"], 365),
    (&["vanilla extract"], 1825),
    (
        &[
            "olive oil",
            "vegetable oil",
            "canola oil",
            "coconut oil",
            "sesame oil",
        ],
        365,
    ),
    (&["vinegar"], 1825),
    (&["chip", "cracker", "pretzel", "popcorn"], 90),
    (
        &[
            "nut",
            "almond",
            "cashew",
            "peanut",
            "walnut",
            "pecan",
            "pistachio",
        ],
        180,
    ),
    (&["chocolate", "candy"], 365),
    (&["jerky"], 365),
    (&["dried fruit", "raisin", "date", "fig"], 180),
    (&["coffee"], 180),
    (&["tea"], 730),
    (
        &[
            "frozen",
            "ice cream",
            "gelato",
            "sorbet",
            "sherbet",
            "popsicle",
        ],
        180,
    ),
    // Non-food items: no expiration
];

/// Suggest a typical shelf life in days for an item based on its name.
///
/// Returns `None` if no match is found.
pub fn suggest_shelf_life_days(item_name: &str) -> Option<u32> {
    let lower = item_name.to_lowercase();
    for (keywords, days) in SHELF_LIFE_RULES {
        for keyword in *keywords {
            if lower.contains(keyword) {
                return Some(*days);
            }
        }
    }
    None
}

/// Suggest an expiration date for an item, calculated from today + typical shelf life.
pub fn suggest_expiration_date(item_name: &str) -> Option<chrono::NaiveDate> {
    suggest_shelf_life_days(item_name)
        .map(|days| chrono::Local::now().date_naive() + chrono::Duration::days(days as i64))
}

/// Suggest a category for an item based on its name.
///
/// Returns `None` if no match is found. Matching is case-insensitive
/// and checks if any keyword is contained within the item name.
pub fn suggest_category(item_name: &str) -> Option<&'static str> {
    let lower = item_name.to_lowercase();
    for (keywords, category) in CATEGORY_RULES {
        for keyword in *keywords {
            if lower.contains(keyword) {
                return Some(category);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categorize_dairy() {
        assert_eq!(suggest_category("Whole Milk"), Some("Dairy"));
        assert_eq!(suggest_category("Greek Yogurt"), Some("Dairy"));
        assert_eq!(suggest_category("Cheddar Cheese"), Some("Dairy"));
    }

    #[test]
    fn categorize_produce() {
        assert_eq!(suggest_category("Bananas"), Some("Produce - Fruits"));
        assert_eq!(
            suggest_category("Baby Spinach"),
            Some("Produce - Vegetables")
        );
        assert_eq!(suggest_category("Red Onion"), Some("Produce - Vegetables"));
    }

    #[test]
    fn categorize_meat() {
        assert_eq!(suggest_category("Chicken Breast"), Some("Meat & Poultry"));
        assert_eq!(suggest_category("Ground Beef"), Some("Meat & Poultry"));
    }

    #[test]
    fn categorize_case_insensitive() {
        assert_eq!(suggest_category("SALMON FILLET"), Some("Seafood"));
        assert_eq!(suggest_category("oat milk"), Some("Dairy"));
    }

    #[test]
    fn no_match_returns_none() {
        assert_eq!(suggest_category("Xylophone"), None);
        assert_eq!(suggest_category(""), None);
    }

    #[test]
    fn categorize_household() {
        assert_eq!(suggest_category("Paper Towels"), Some("Household"));
        assert_eq!(suggest_category("Dish Soap"), Some("Household"));
    }

    #[test]
    fn categorize_beverages() {
        assert_eq!(suggest_category("Coffee Beans"), Some("Beverages"));
        assert_eq!(suggest_category("Green Tea"), Some("Beverages"));
    }

    #[test]
    fn categorize_grains() {
        assert_eq!(suggest_category("Brown Rice"), Some("Grains & Pasta"));
        assert_eq!(suggest_category("Spaghetti"), Some("Grains & Pasta"));
    }

    #[test]
    fn shelf_life_perishables() {
        assert_eq!(suggest_shelf_life_days("Whole Milk"), Some(10));
        assert_eq!(suggest_shelf_life_days("Chicken Breast"), Some(5));
        assert_eq!(suggest_shelf_life_days("Ground Beef"), Some(2));
        assert_eq!(suggest_shelf_life_days("Salmon"), Some(3));
    }

    #[test]
    fn shelf_life_long_lasting() {
        assert_eq!(suggest_shelf_life_days("Brown Rice"), Some(730));
        assert_eq!(suggest_shelf_life_days("Canned Beans"), Some(730));
        assert_eq!(suggest_shelf_life_days("Sugar"), Some(1825));
    }

    #[test]
    fn shelf_life_no_match() {
        assert_eq!(suggest_shelf_life_days("Xylophone"), None);
    }

    #[test]
    fn expiration_date_suggestion() {
        let today = chrono::Local::now().date_naive();
        let exp = suggest_expiration_date("Milk").unwrap();
        assert_eq!(exp, today + chrono::Duration::days(10));
    }
}
