# REST API Reference

The web server runs on the host/port defined in `config.toml` (default: `http://127.0.0.1:3000`). Requires building with `--features web`.

## Items

### List all items

```
GET /api/items
```

Returns: `200` with JSON array of items.

### Create an item

```
POST /api/items
Content-Type: application/json

{
  "name": "Milk",
  "quantity": 2,
  "unit": "gallons",
  "category": "Dairy",
  "expiration_date": "2026-03-15",
  "minimum_stock": 1,
  "location_id": 1,
  "shelf_id": 2
}
```

Required fields: `name`, `quantity`, `unit`. All others are optional.

When `shelf_id` is provided, `location_id` is auto-set from the shelf's parent location.

Returns: `201` with the created item.

### Get an item

```
GET /api/items/:id
```

Returns: `200` with item JSON, or `404`.

### Update an item

```
PUT /api/items/:id
Content-Type: application/json

{ "name": "Milk", "quantity": 3, "unit": "gallons" }
```

All fields from create are accepted. When `shelf_id` is provided, `location_id` is auto-set.

Returns: `200` with updated item, or `404`.

### Delete an item

```
DELETE /api/items/:id
```

Returns: `204`, or `404`.

## Locations

### List all locations

```
GET /api/locations
```

### Create a location

```
POST /api/locations
Content-Type: application/json

{ "name": "Fridge", "temperature_f": 37.0 }
```

Returns: `201`.

### Get a location

```
GET /api/locations/:id
```

### Update a location

```
PUT /api/locations/:id
Content-Type: application/json

{ "name": "Fridge", "temperature_f": 35.0 }
```

### Delete a location

```
DELETE /api/locations/:id
```

Cascade-deletes shelves. Sets `location_id` to null on associated items.

## Shelves

### List shelves for a location

```
GET /api/locations/:id/shelves
```

### List all shelves

```
GET /api/shelves
```

### Create a shelf

```
POST /api/locations/:id/shelves
Content-Type: application/json

{ "name": "Top Shelf" }
```

Returns: `201`.

### Get a shelf

```
GET /api/shelves/:id
```

### Delete a shelf

```
DELETE /api/shelves/:id
```

Sets `shelf_id` to null on associated items. Does not affect `location_id`.

## Shopping List

```
GET /api/shopping
```

Returns: `200` with JSON:

```json
{
  "entries": [
    {
      "name": "Salt",
      "current_quantity": 1,
      "suggested_quantity": 2,
      "unit": "box",
      "category": "Spices",
      "expiring": false
    }
  ]
}
```

Items expiring within 3 days are automatically included with `"expiring": true`, even if they are not low stock.

## Item Suggestions

```
GET /api/suggest-category?name=Chicken+Breast
```

Returns: `200` with JSON:

```json
{
  "category": "Meat & Poultry",
  "expiration_date": "2026-03-13"
}
```

Suggests a category and estimated expiration date based on the item name. Used by the web UI for real-time auto-fill. Returns `null` for fields that cannot be determined.

## Receipt Scanning

```
POST /api/receipt/scan
Content-Type: multipart/form-data

receipt: <image file>
```

Uploads a receipt image and extracts grocery items using the Claude Vision API. Requires `[anthropic] api_key` in config.

Supported image types: JPEG, PNG, WebP, GIF (max 5MB).

Returns: `200` with JSON array:

```json
[
  {
    "name": "Whole Milk",
    "quantity": 2,
    "unit": "gallons",
    "category": "Dairy",
    "expiration_date": "2026-03-18"
  }
]
```

Items are automatically enriched with category and expiration date suggestions. Fractional weights are converted (e.g., 0.776 kg becomes 776 g).

Returns `400` if no API key is configured or the image is invalid.

## Health Check

```
GET /health
```

Returns: `200 ok`
