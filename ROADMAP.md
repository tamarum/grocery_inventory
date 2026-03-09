# Grocery Inventory Roadmap

Future features and enhancements, organized by theme.

---

## Item Intelligence

### Expiration Date Tracking
Alert when items are about to expire. Display warnings in both CLI and web UI.

### Auto-Fill Expiration Dates
Automatically estimate expiration dates based on the item type (e.g., milk = 7 days, canned goods = 2 years). Use a built-in knowledge base with user-overridable defaults.

### Auto-Fill Categories
Automatically assign categories to items based on their name using a lookup table or ML classifier. Reduce manual data entry when adding items.

### Barcode / UPC Scanning
Add items by scanning a barcode. Look up product details (name, category, typical expiration) from a UPC database.

### Receipt Scanning
Take a picture of a grocery store receipt and batch-import all items. Use OCR to extract item names, quantities, and prices, then map them to inventory entries.

---

## Organization & Visualization

### Categories and Tags
Organize items beyond location and shelf. Support multiple tags per item for flexible filtering and grouping.

### Visual Location Management
Interactive drag-and-drop interface for managing locations and shelves. Visually arrange items within locations — see at a glance what's where and reorganize by dragging.

---

## Shopping & Recipes

### Recipe Integration
Auto-generate shopping lists from recipes. Select a recipe, and the system checks current inventory and adds only the missing ingredients to the shopping list.

### Recipe Suggestions from Inventory
Generate suggested recipes based on what's currently in the inventory, prioritizing items expiring soonest. Reduce food waste by helping users cook what they have.

---

## Analytics & History

### Usage History and Analytics
Track consumption patterns over time. Show insights like frequently purchased items, average consumption rate, and spending trends.

---

## Notifications

### Low Stock and Expiration Alerts
Push notifications or email alerts when items are running low or approaching their expiration date.

---

## Data Management

### Import / Export
CSV and JSON support for bulk item management. Export inventory for backup or sharing, import from spreadsheets.

### Multi-User Support
Household members with separate views and permissions. Shared inventory with individual shopping lists.

---

## Mobile

### Mobile App
Dedicated native mobile app beyond the current responsive web UI. Offline support, push notifications, and camera integration for barcode/receipt scanning.
