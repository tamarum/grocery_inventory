#[cfg(feature = "web")]
pub mod routes {
    use axum::{
        extract::{Path, State},
        http::StatusCode,
        response::{Html, IntoResponse},
        routing::get,
        Json, Router,
    };
    use chrono::NaiveDate;
    use serde::Deserialize;
    use std::sync::Arc;

    use crate::app::App;
    use crate::category::{suggest_category, suggest_expiration_date};
    use crate::db::SqliteRepository;
    use crate::item::{GroceryItem, ItemError};
    use crate::location::{Location, Shelf};
    use crate::receipt::scanner;
    use crate::shopping::DefaultShoppingListGenerator;

    type SharedApp = Arc<App<SqliteRepository, DefaultShoppingListGenerator>>;

    #[derive(Debug, Deserialize)]
    struct ItemRequest {
        name: String,
        quantity: u32,
        unit: String,
        category: Option<String>,
        expiration_date: Option<NaiveDate>,
        #[serde(default)]
        minimum_stock: u32,
        location_id: Option<i64>,
        shelf_id: Option<i64>,
    }

    impl ItemRequest {
        fn auto_fill(self) -> Self {
            let category = if self.category.is_some() {
                self.category
            } else {
                suggest_category(&self.name).map(String::from)
            };
            let expiration_date = if self.expiration_date.is_some() {
                self.expiration_date
            } else {
                suggest_expiration_date(&self.name)
            };
            Self {
                category,
                expiration_date,
                ..self
            }
        }

        fn into_item(self) -> GroceryItem {
            let req = self.auto_fill();
            GroceryItem {
                id: None,
                name: req.name,
                quantity: req.quantity,
                unit: req.unit,
                category: req.category,
                expiration_date: req.expiration_date,
                minimum_stock: req.minimum_stock,
                location_id: req.location_id,
                shelf_id: req.shelf_id,
            }
        }

        fn into_item_with_id(self, id: i64) -> GroceryItem {
            let req = self.auto_fill();
            GroceryItem {
                id: Some(id),
                name: req.name,
                quantity: req.quantity,
                unit: req.unit,
                category: req.category,
                expiration_date: req.expiration_date,
                minimum_stock: req.minimum_stock,
                location_id: req.location_id,
                shelf_id: req.shelf_id,
            }
        }
    }

    #[derive(Debug, Deserialize)]
    struct ShelfRequest {
        name: String,
    }

    #[derive(Debug, Deserialize)]
    struct LocationRequest {
        name: String,
        temperature_f: f64,
    }

    fn error_response(err: ItemError) -> (StatusCode, String) {
        match err {
            ItemError::NotFound(_)
            | ItemError::LocationNotFound(_)
            | ItemError::ShelfNotFound(_) => (StatusCode::NOT_FOUND, err.to_string()),
            ItemError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        }
    }

    pub fn create_router(app: SharedApp) -> Router {
        Router::new()
            .route("/", get(index_html))
            .route("/api/items", get(list_items).post(create_item))
            .route(
                "/api/items/:id",
                get(get_item).put(update_item).delete(remove_item),
            )
            .route("/api/locations", get(list_locations).post(create_location))
            .route(
                "/api/locations/:id",
                get(get_location)
                    .put(update_location)
                    .delete(remove_location),
            )
            .route(
                "/api/locations/:id/shelves",
                get(list_shelves).post(create_shelf),
            )
            .route("/api/shelves", get(list_all_shelves))
            .route("/api/shelves/:id", get(get_shelf).delete(remove_shelf))
            .route("/api/shopping", get(shopping_list))
            .route("/api/suggest-category", get(suggest_category_endpoint))
            .route(
                "/api/receipt/scan",
                axum::routing::post(scan_receipt_endpoint),
            )
            .route("/health", get(health))
            .with_state(app)
    }

    async fn health() -> &'static str {
        "ok"
    }

    #[derive(Deserialize)]
    struct SuggestCategoryQuery {
        name: String,
    }

    #[derive(serde::Serialize)]
    struct ItemSuggestion {
        category: Option<&'static str>,
        expiration_date: Option<NaiveDate>,
    }

    async fn suggest_category_endpoint(
        axum::extract::Query(query): axum::extract::Query<SuggestCategoryQuery>,
    ) -> Json<ItemSuggestion> {
        Json(ItemSuggestion {
            category: suggest_category(&query.name),
            expiration_date: suggest_expiration_date(&query.name),
        })
    }

    async fn scan_receipt_endpoint(
        State(app): State<SharedApp>,
        mut multipart: axum::extract::Multipart,
    ) -> impl IntoResponse {
        let api_key = match &app.config.anthropic.api_key {
            Some(key) => key.clone(),
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    "Anthropic API key not configured. Add [anthropic] api_key to config.toml",
                )
                    .into_response()
            }
        };

        let mut image_bytes = Vec::new();
        let mut media_type = String::new();

        while let Ok(Some(field)) = multipart.next_field().await {
            if field.name() == Some("receipt") {
                media_type = field.content_type().unwrap_or("image/jpeg").to_string();
                match field.bytes().await {
                    Ok(bytes) => image_bytes = bytes.to_vec(),
                    Err(e) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            format!("Failed to read upload: {e}"),
                        )
                            .into_response()
                    }
                }
                break;
            }
        }

        if image_bytes.is_empty() {
            return (StatusCode::BAD_REQUEST, "No receipt image uploaded").into_response();
        }

        if let Err(e) = scanner::validate_image(&image_bytes, &media_type) {
            return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
        }

        match scanner::scan_receipt(&api_key, &image_bytes, &media_type).await {
            Ok(items) => Json(items).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }

    async fn index_html() -> Html<&'static str> {
        Html(INDEX_HTML)
    }

    const INDEX_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Grocery Inventory</title>
<style>
  *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
  body { font-family: system-ui, -apple-system, sans-serif; background: #f5f5f5; color: #333; padding: 1rem; max-width: 960px; margin: 0 auto; }
  h1 { margin-bottom: 1rem; }
  .tabs { display: flex; gap: .5rem; margin-bottom: 1rem; }
  .tabs button { padding: .5rem 1rem; border: 1px solid #ccc; background: #fff; border-radius: 4px 4px 0 0; cursor: pointer; }
  .tabs button.active { background: #4a7c59; color: #fff; border-color: #4a7c59; }
  .panel { display: none; }
  .panel.active { display: block; }
  table { width: 100%; border-collapse: collapse; background: #fff; border-radius: 4px; overflow: hidden; }
  th, td { padding: .5rem .75rem; text-align: left; border-bottom: 1px solid #eee; }
  th { background: #4a7c59; color: #fff; font-weight: 600; }
  tr:hover { background: #f0f7f2; }
  .btn { padding: .35rem .75rem; border: none; border-radius: 4px; cursor: pointer; font-size: .85rem; }
  .btn-edit { background: #e8a83e; color: #fff; }
  .btn-delete { background: #c0392b; color: #fff; }
  .btn-primary { background: #4a7c59; color: #fff; padding: .5rem 1.5rem; font-size: 1rem; }
  .btn:hover { opacity: .85; }
  form { background: #fff; padding: 1rem; border-radius: 4px; margin-bottom: 1rem; display: grid; grid-template-columns: 1fr 1fr; gap: .75rem; }
  form label { display: flex; flex-direction: column; font-size: .85rem; font-weight: 600; }
  form input, form select { padding: .4rem; border: 1px solid #ccc; border-radius: 4px; margin-top: .25rem; }
  .form-actions { grid-column: 1 / -1; display: flex; gap: .5rem; justify-content: flex-end; }
  .msg { padding: .5rem 1rem; border-radius: 4px; margin-bottom: .75rem; }
  .msg-ok { background: #d4edda; color: #155724; }
  .msg-err { background: #f8d7da; color: #721c24; }
  .low-stock { color: #c0392b; font-weight: 600; }
  .shop-entry { background: #fff; padding: .75rem 1rem; margin-bottom: .5rem; border-radius: 4px; border-left: 4px solid #4a7c59; }
  .empty { text-align: center; padding: 2rem; color: #888; }
  .batch-controls { display: flex; gap: .5rem; margin-bottom: .75rem; }
  .batch-table input, .batch-table select { width: 100%; padding: .35rem .4rem; border: 1px solid #ddd; border-radius: 3px; font-size: .85rem; }
  .batch-table input:focus, .batch-table select:focus { outline: none; border-color: #4a7c59; }
  .batch-table td { padding: .3rem .25rem; border-bottom: 1px solid #e0e0e0; border-right: 1px solid #f0f0f0; }
  .batch-table td:last-child { border-right: none; text-align: center; }
  .btn-remove { background: none; border: none; color: #c0392b; cursor: pointer; font-size: 1.1rem; padding: .2rem .5rem; }
  .btn-remove:hover { background: #fbeaea; border-radius: 4px; }
  .table-wrap { overflow-x: auto; -webkit-overflow-scrolling: touch; }

  @media (max-width: 640px) {
    body { padding: .5rem; }
    h1 { font-size: 1.3rem; }

    .tabs { flex-wrap: wrap; }
    .tabs button { flex: 1; min-width: 45%; padding: .65rem .5rem; font-size: .9rem; border-radius: 4px; }

    form { grid-template-columns: 1fr; }
    form input, form select { padding: .6rem; font-size: 1rem; }
    form label { font-size: .9rem; }

    .btn { padding: .55rem 1rem; font-size: .9rem; }
    .btn-primary { padding: .65rem 1.5rem; }
    .btn-edit, .btn-delete { padding: .5rem .85rem; }

    .card-table thead { display: none; }
    .card-table tbody tr { display: block; background: #fff; margin-bottom: .75rem; border-radius: 6px; padding: .75rem; box-shadow: 0 1px 3px rgba(0,0,0,.1); }
    .card-table tbody td { display: flex; justify-content: space-between; align-items: center; padding: .35rem 0; border-bottom: 1px solid #f5f5f5; }
    .card-table tbody td::before { content: attr(data-label); font-weight: 600; color: #666; font-size: .85rem; flex-shrink: 0; margin-right: .75rem; }
    .card-table tbody td:last-child { justify-content: flex-end; gap: .5rem; border-bottom: none; padding-top: .5rem; }
    .card-table tbody td:last-child::before { display: none; }

    .card-table.batch-table td { border-right: none; }
    .card-table.batch-table input, .card-table.batch-table select { flex: 1; min-width: 0; padding: .5rem; font-size: 1rem; }

    .shop-entry { font-size: .95rem; }
  }
</style>
</head>
<body>
<h1>Grocery Inventory</h1>
<div id="msg"></div>

<div class="tabs">
  <button class="active" data-tab="inventory" onclick="showTab('inventory')">Inventory</button>
  <button data-tab="shopping" onclick="showTab('shopping')">Shopping List</button>
  <button data-tab="batch" onclick="showTab('batch')">Batch Add</button>
  <button data-tab="locations" onclick="showTab('locations')">Locations</button>
</div>

<div id="inventory" class="panel active">
  <form id="item-form" onsubmit="return saveItem(event)">
    <input type="hidden" id="edit-id">
    <label>Name <input type="text" id="f-name" required></label>
    <label>Quantity <input type="number" id="f-qty" min="0" required></label>
    <label>Unit <input type="text" id="f-unit" required></label>
    <label>Category <input type="text" id="f-cat"></label>
    <label>Expiration Date <input type="date" id="f-exp"></label>
    <label>Min Stock <input type="number" id="f-min" min="0" value="0"></label>
    <label>Location <select id="f-loc" onchange="updateShelfDropdown('f-shelf', this.value)"><option value="">-- None --</option></select></label>
    <label>Shelf <select id="f-shelf"><option value="">-- None --</option></select></label>
    <div class="form-actions">
      <button type="button" class="btn" onclick="resetForm()">Cancel</button>
      <button type="submit" class="btn btn-primary" id="submit-btn">Add Item</button>
    </div>
  </form>
  <table class="card-table">
    <thead>
      <tr>
        <th>Name</th><th>Qty</th><th>Unit</th><th>Category</th><th>Expires</th><th>Min Stock</th><th>Location</th><th>Shelf</th><th>Actions</th>
      </tr>
    </thead>
    <tbody id="items-body"></tbody>
  </table>
</div>

<div id="shopping" class="panel">
  <div id="shop-list"></div>
</div>

<div id="batch" class="panel">
  <div style="margin-bottom:.75rem;">
    <label class="btn btn-primary" style="display:inline-flex;align-items:center;gap:.4rem;cursor:pointer;">
      Scan Receipt
      <input type="file" id="receipt-input" accept="image/*" capture="environment"
             onchange="scanReceipt(this)" style="display:none">
    </label>
    <span id="receipt-status" style="margin-left:.75rem;color:#888;"></span>
  </div>
  <div id="receipt-preview" style="display:none;margin-bottom:.75rem;">
    <img id="receipt-img" style="max-width:100%;max-height:200px;border-radius:4px;border:1px solid #ddd;">
  </div>
  <div class="batch-controls">
    <button class="btn btn-primary" onclick="addBatchRow()">+ Add Row</button>
    <button class="btn btn-primary" onclick="submitBatch()">Submit All</button>
  </div>
  <table class="batch-table card-table">
    <thead>
      <tr>
        <th>Name</th><th>Qty</th><th>Unit</th><th>Category</th><th>Expires</th><th>Min Stock</th><th>Location</th><th>Shelf</th><th></th>
      </tr>
    </thead>
    <tbody id="batch-body"></tbody>
  </table>
</div>

<div id="locations" class="panel">
  <form id="loc-form" onsubmit="return saveLocation(event)">
    <input type="hidden" id="loc-edit-id">
    <label>Name <input type="text" id="loc-name" required></label>
    <label>Temperature (F) <input type="number" id="loc-temp" step="0.1" required></label>
    <div class="form-actions">
      <button type="button" class="btn" onclick="resetLocForm()">Cancel</button>
      <button type="submit" class="btn btn-primary" id="loc-submit-btn">Add Location</button>
    </div>
  </form>
  <table class="card-table">
    <thead>
      <tr>
        <th>Name</th><th>Temp (F)</th><th>Shelves</th><th>Actions</th>
      </tr>
    </thead>
    <tbody id="loc-body"></tbody>
  </table>
  <h3 style="margin:1rem 0 .5rem">Add Shelf</h3>
  <form id="shelf-form" onsubmit="return saveShelf(event)">
    <label>Location <select id="shelf-loc" required><option value="">-- Select --</option></select></label>
    <label>Shelf Name <input type="text" id="shelf-name" required></label>
    <div class="form-actions">
      <button type="submit" class="btn btn-primary">Add Shelf</button>
    </div>
  </form>
</div>

<script>
const API = '/api';
let locationsMap = new Map();
let shelvesMap = new Map();
let shelvesByLocation = new Map();

function showTab(name) {
  document.querySelectorAll('.panel').forEach(p => p.classList.remove('active'));
  document.querySelectorAll('.tabs button').forEach(b => b.classList.remove('active'));
  document.getElementById(name).classList.add('active');
  event.target.classList.add('active');
  localStorage.setItem('activeTab', name);
  if (name === 'shopping') loadShopping();
  if (name === 'inventory') loadItems();
  if (name === 'batch') initBatch();
  if (name === 'locations') loadLocationsTable();
}

function flash(text, ok) {
  const el = document.getElementById('msg');
  el.className = 'msg ' + (ok ? 'msg-ok' : 'msg-err');
  el.textContent = text;
  setTimeout(() => { el.className = ''; el.textContent = ''; }, 3000);
}

async function loadLocations() {
  try {
    const [locRes, shelfRes] = await Promise.all([
      fetch(API + '/locations'),
      fetch(API + '/shelves')
    ]);
    const locs = await locRes.json();
    const shelves = await shelfRes.json();
    locationsMap = new Map(locs.map(l => [l.id, l]));
    shelvesMap = new Map(shelves.map(s => [s.id, s]));
    shelvesByLocation = new Map();
    shelves.forEach(s => {
      if (!shelvesByLocation.has(s.location_id)) shelvesByLocation.set(s.location_id, []);
      shelvesByLocation.get(s.location_id).push(s);
    });
    populateLocationDropdowns();
  } catch (e) { /* silent */ }
}

function populateLocationDropdowns() {
  const selects = document.querySelectorAll('#f-loc, .batch-table select[data-loc]');
  selects.forEach(sel => {
    const current = sel.value;
    const opts = '<option value="">-- None --</option>' +
      Array.from(locationsMap.values()).map(l =>
        `<option value="${l.id}">${esc(l.name)} (${l.temperature_f}\u00b0F)</option>`
      ).join('');
    sel.innerHTML = opts;
    sel.value = current;
  });
}

function locationName(id) {
  if (!id) return '';
  const loc = locationsMap.get(id);
  return loc ? loc.name : '';
}

function shelfName(id) {
  if (!id) return '';
  const s = shelvesMap.get(id);
  return s ? s.name : '';
}

function updateShelfDropdown(selId, locId) {
  const sel = document.getElementById(selId);
  if (!sel) return;
  const current = sel.value;
  let opts = '<option value="">-- None --</option>';
  if (locId) {
    const shelves = shelvesByLocation.get(parseInt(locId, 10)) || [];
    opts += shelves.map(s => `<option value="${s.id}">${esc(s.name)}</option>`).join('');
  }
  sel.innerHTML = opts;
  sel.value = current;
}

async function loadItems() {
  await loadLocations();
  try {
    const res = await fetch(API + '/items');
    const items = await res.json();
    const tbody = document.getElementById('items-body');
    if (items.length === 0) {
      tbody.innerHTML = '<tr><td colspan="9" class="empty">No items yet. Add one above!</td></tr>';
      return;
    }
    tbody.innerHTML = items.map(i => `<tr>
      <td data-label="Name">${esc(i.name)}</td>
      <td data-label="Qty" class="${i.quantity <= i.minimum_stock ? 'low-stock' : ''}">${i.quantity}</td>
      <td data-label="Unit">${esc(i.unit)}</td>
      <td data-label="Category">${esc(i.category || '')}</td>
      <td data-label="Expires">${i.expiration_date || ''}</td>
      <td data-label="Min Stock">${i.minimum_stock}</td>
      <td data-label="Location">${esc(locationName(i.location_id))}</td>
      <td data-label="Shelf">${esc(shelfName(i.shelf_id))}</td>
      <td>
        <button class="btn btn-edit" onclick='editItem(${JSON.stringify(i).replace(/'/g,"&#39;")})'>Edit</button>
        <button class="btn btn-delete" onclick="deleteItem(${i.id})">Delete</button>
      </td>
    </tr>`).join('');
  } catch (e) { flash('Failed to load items', false); }
}

function esc(s) {
  const d = document.createElement('div');
  d.textContent = s;
  return d.innerHTML;
}

function editItem(item) {
  document.getElementById('edit-id').value = item.id;
  document.getElementById('f-name').value = item.name;
  document.getElementById('f-qty').value = item.quantity;
  document.getElementById('f-unit').value = item.unit;
  document.getElementById('f-cat').value = item.category || '';
  document.getElementById('f-exp').value = item.expiration_date || '';
  document.getElementById('f-min').value = item.minimum_stock;
  document.getElementById('f-loc').value = item.location_id || '';
  updateShelfDropdown('f-shelf', item.location_id || '');
  document.getElementById('f-shelf').value = item.shelf_id || '';
  document.getElementById('submit-btn').textContent = 'Update Item';
}

// Auto-suggest category when name changes
let _catSuggestTimer;
document.getElementById('f-name').addEventListener('input', function() {
  clearTimeout(_catSuggestTimer);
  const catField = document.getElementById('f-cat');
  if (catField.dataset.userEdited === 'true') return;
  const name = this.value.trim();
  if (!name) { catField.value = ''; return; }
  _catSuggestTimer = setTimeout(async () => {
    try {
      const res = await fetch('/api/suggest-category?name=' + encodeURIComponent(name));
      const data = await res.json();
      if (data.category && !catField.dataset.userEdited) {
        catField.value = data.category;
      }
      const expField = document.getElementById('f-exp');
      if (data.expiration_date && !expField.dataset.userEdited) {
        expField.value = data.expiration_date;
      }
    } catch(e) {}
  }, 300);
});
document.getElementById('f-cat').addEventListener('input', function() {
  this.dataset.userEdited = this.value ? 'true' : '';
});
document.getElementById('f-exp').addEventListener('input', function() {
  this.dataset.userEdited = this.value ? 'true' : '';
});

function resetForm() {
  document.getElementById('f-cat').dataset.userEdited = '';
  document.getElementById('f-exp').dataset.userEdited = '';
  document.getElementById('item-form').reset();
  document.getElementById('edit-id').value = '';
  document.getElementById('f-min').value = '0';
  document.getElementById('f-loc').value = '';
  document.getElementById('f-shelf').innerHTML = '<option value="">-- None --</option>';
  document.getElementById('submit-btn').textContent = 'Add Item';
}

async function saveItem(e) {
  e.preventDefault();
  const id = document.getElementById('edit-id').value;
  const locVal = document.getElementById('f-loc').value;
  const shelfVal = document.getElementById('f-shelf').value;
  const body = {
    name: document.getElementById('f-name').value,
    quantity: parseInt(document.getElementById('f-qty').value, 10),
    unit: document.getElementById('f-unit').value,
    category: document.getElementById('f-cat').value || null,
    expiration_date: document.getElementById('f-exp').value || null,
    minimum_stock: parseInt(document.getElementById('f-min').value, 10) || 0,
    location_id: locVal ? parseInt(locVal, 10) : null,
    shelf_id: shelfVal ? parseInt(shelfVal, 10) : null,
  };
  try {
    const url = id ? `${API}/items/${id}` : `${API}/items`;
    const res = await fetch(url, {
      method: id ? 'PUT' : 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify(body),
    });
    if (!res.ok) throw new Error(await res.text());
    flash(id ? 'Item updated' : 'Item added', true);
    resetForm();
    loadItems();
  } catch (e) { flash('Error: ' + e.message, false); }
}

async function deleteItem(id) {
  if (!confirm('Delete this item?')) return;
  try {
    const res = await fetch(`${API}/items/${id}`, { method: 'DELETE' });
    if (!res.ok) throw new Error(await res.text());
    flash('Item deleted', true);
    loadItems();
  } catch (e) { flash('Error: ' + e.message, false); }
}

async function loadShopping() {
  try {
    const res = await fetch(API + '/shopping');
    const data = await res.json();
    const el = document.getElementById('shop-list');
    if (data.entries.length === 0) {
      el.innerHTML = '<div class="empty">You\'re fully stocked!</div>';
      return;
    }
    el.innerHTML = '<h3 style="margin-bottom:.75rem">You need ' + data.entries.length + ' item(s):</h3>' +
      data.entries.map(e => `<div class="shop-entry">
        <strong>${esc(e.name)}</strong> &mdash; need ${e.suggested_quantity} ${esc(e.unit)}
        (have ${e.current_quantity})
        ${e.category ? '<span style="color:#888"> [' + esc(e.category) + ']</span>' : ''}
      </div>`).join('');
  } catch (e) { flash('Failed to load shopping list', false); }
}

async function scanReceipt(input) {
  const file = input.files[0];
  if (!file) return;

  const reader = new FileReader();
  reader.onload = e => {
    document.getElementById('receipt-img').src = e.target.result;
    document.getElementById('receipt-preview').style.display = 'block';
  };
  reader.readAsDataURL(file);

  const status = document.getElementById('receipt-status');
  status.textContent = 'Scanning receipt...';
  status.style.color = '#888';

  const formData = new FormData();
  formData.append('receipt', file);

  try {
    const res = await fetch('/api/receipt/scan', { method: 'POST', body: formData });
    if (!res.ok) throw new Error(await res.text());
    const items = await res.json();

    if (items.length === 0) {
      status.textContent = 'No items found on receipt.';
      status.style.color = '#c0392b';
      input.value = '';
      return;
    }

    document.getElementById('batch-body').innerHTML = '';
    await loadLocations();
    items.forEach(item => {
      addBatchRow();
      const row = document.getElementById('batch-body').lastElementChild;
      const inputs = row.querySelectorAll('input');
      inputs[0].value = item.name || '';
      inputs[1].value = item.quantity || 1;
      inputs[2].value = item.unit || 'count';
      inputs[3].value = item.category || '';
      if (item.expiration_date) inputs[4].value = item.expiration_date;
    });

    status.textContent = items.length + ' item(s) found. Review and submit below.';
    status.style.color = '#155724';
  } catch (e) {
    status.textContent = 'Scan failed: ' + e.message;
    status.style.color = '#c0392b';
  }
  input.value = '';
}

function initBatch() {
  loadLocations();
  const tbody = document.getElementById('batch-body');
  if (tbody.children.length === 0) {
    for (let i = 0; i < 5; i++) addBatchRow();
  }
}

function addBatchRow() {
  const tbody = document.getElementById('batch-body');
  const tr = document.createElement('tr');
  const rowIdx = tbody.children.length;
  const locOpts = '<option value="">-- None --</option>' +
    Array.from(locationsMap.values()).map(l =>
      `<option value="${l.id}">${esc(l.name)}</option>`
    ).join('');
  tr.innerHTML = `
    <td data-label="Name"><input type="text" placeholder="Name"></td>
    <td data-label="Qty"><input type="number" min="0" placeholder="0"></td>
    <td data-label="Unit"><input type="text" placeholder="Unit"></td>
    <td data-label="Category"><input type="text" placeholder="Category"></td>
    <td data-label="Expires"><input type="date"></td>
    <td data-label="Min Stock"><input type="number" min="0" placeholder="0"></td>
    <td data-label="Location"><select data-loc onchange="updateBatchShelf(this)">${locOpts}</select></td>
    <td data-label="Shelf"><select data-shelf><option value="">-- None --</option></select></td>
    <td><button class="btn-remove" onclick="removeBatchRow(this)" title="Remove row">&times;</button></td>`;
  tbody.appendChild(tr);
}

function updateBatchShelf(locSel) {
  const row = locSel.closest('tr');
  const shelfSel = row.querySelector('select[data-shelf]');
  const locId = locSel.value;
  let opts = '<option value="">-- None --</option>';
  if (locId) {
    const shelves = shelvesByLocation.get(parseInt(locId, 10)) || [];
    opts += shelves.map(s => `<option value="${s.id}">${esc(s.name)}</option>`).join('');
  }
  shelfSel.innerHTML = opts;
}

function removeBatchRow(btn) {
  btn.closest('tr').remove();
}

async function submitBatch() {
  const rows = document.querySelectorAll('#batch-body tr');
  let ok = 0, fail = 0;
  for (const row of rows) {
    const inputs = row.querySelectorAll('input');
    const locSel = row.querySelector('select[data-loc]');
    const shelfSel = row.querySelector('select[data-shelf]');
    const name = inputs[0].value.trim();
    if (!name) continue;
    const unit = inputs[2].value.trim();
    if (!unit) { fail++; continue; }
    const locVal = locSel ? locSel.value : '';
    const shelfVal = shelfSel ? shelfSel.value : '';
    const body = {
      name,
      quantity: parseInt(inputs[1].value, 10) || 0,
      unit,
      category: inputs[3].value.trim() || null,
      expiration_date: inputs[4].value || null,
      minimum_stock: parseInt(inputs[5].value, 10) || 0,
      location_id: locVal ? parseInt(locVal, 10) : null,
      shelf_id: shelfVal ? parseInt(shelfVal, 10) : null,
    };
    try {
      const res = await fetch(API + '/items', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify(body),
      });
      if (!res.ok) throw new Error();
      ok++;
    } catch (e) { fail++; }
  }
  if (ok === 0 && fail === 0) { flash('No rows to submit', false); return; }
  flash(ok + ' item(s) added' + (fail ? ', ' + fail + ' failed' : ''), ok > 0);
  if (ok > 0) {
    document.getElementById('batch-body').innerHTML = '';
    for (let i = 0; i < 5; i++) addBatchRow();
    loadItems();
  }
}

// --- Locations tab ---

async function loadLocationsTable() {
  await loadLocations();
  const tbody = document.getElementById('loc-body');
  const locs = Array.from(locationsMap.values());
  // populate shelf form location dropdown
  const shelfLocSel = document.getElementById('shelf-loc');
  if (shelfLocSel) {
    shelfLocSel.innerHTML = '<option value="">-- Select --</option>' +
      locs.map(l => `<option value="${l.id}">${esc(l.name)}</option>`).join('');
  }
  if (locs.length === 0) {
    tbody.innerHTML = '<tr><td colspan="4" class="empty">No locations yet. Add one above!</td></tr>';
    return;
  }
  tbody.innerHTML = locs.map(l => {
    const locShelves = shelvesByLocation.get(l.id) || [];
    const shelfList = locShelves.length === 0 ? '<em>none</em>' :
      locShelves.map(s => `<span style="display:inline-block;background:#eee;padding:.15rem .4rem;border-radius:3px;margin:.1rem">${esc(s.name)} <button class="btn-remove" onclick="deleteShelf(${s.id})" title="Remove shelf" style="font-size:.8rem">&times;</button></span>`).join(' ');
    return `<tr>
      <td data-label="Name">${esc(l.name)}</td>
      <td data-label="Temp">${l.temperature_f}\u00b0F</td>
      <td data-label="Shelves">${shelfList}</td>
      <td>
        <button class="btn btn-edit" onclick='editLocation(${JSON.stringify(l).replace(/'/g,"&#39;")})'>Edit</button>
        <button class="btn btn-delete" onclick="deleteLocation(${l.id})">Delete</button>
      </td>
    </tr>`;
  }).join('');
}

function editLocation(loc) {
  document.getElementById('loc-edit-id').value = loc.id;
  document.getElementById('loc-name').value = loc.name;
  document.getElementById('loc-temp').value = loc.temperature_f;
  document.getElementById('loc-submit-btn').textContent = 'Update Location';
}

function resetLocForm() {
  document.getElementById('loc-form').reset();
  document.getElementById('loc-edit-id').value = '';
  document.getElementById('loc-submit-btn').textContent = 'Add Location';
}

async function saveLocation(e) {
  e.preventDefault();
  const id = document.getElementById('loc-edit-id').value;
  const body = {
    name: document.getElementById('loc-name').value,
    temperature_f: parseFloat(document.getElementById('loc-temp').value),
  };
  try {
    const url = id ? `${API}/locations/${id}` : `${API}/locations`;
    const res = await fetch(url, {
      method: id ? 'PUT' : 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify(body),
    });
    if (!res.ok) throw new Error(await res.text());
    flash(id ? 'Location updated' : 'Location added', true);
    resetLocForm();
    loadLocationsTable();
  } catch (e) { flash('Error: ' + e.message, false); }
}

async function saveShelf(e) {
  e.preventDefault();
  const locId = document.getElementById('shelf-loc').value;
  const name = document.getElementById('shelf-name').value;
  if (!locId) { flash('Select a location', false); return; }
  try {
    const res = await fetch(`${API}/locations/${locId}/shelves`, {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({ name }),
    });
    if (!res.ok) throw new Error(await res.text());
    flash('Shelf added', true);
    document.getElementById('shelf-name').value = '';
    loadLocationsTable();
  } catch (e) { flash('Error: ' + e.message, false); }
}

async function deleteShelf(id) {
  if (!confirm('Delete this shelf? Items on it will have their shelf cleared.')) return;
  try {
    const res = await fetch(`${API}/shelves/${id}`, { method: 'DELETE' });
    if (!res.ok) throw new Error(await res.text());
    flash('Shelf deleted', true);
    loadLocationsTable();
  } catch (e) { flash('Error: ' + e.message, false); }
}

async function deleteLocation(id) {
  if (!confirm('Delete this location? Items assigned to it will have their location cleared.')) return;
  try {
    const res = await fetch(`${API}/locations/${id}`, { method: 'DELETE' });
    if (!res.ok) throw new Error(await res.text());
    flash('Location deleted', true);
    loadLocationsTable();
  } catch (e) { flash('Error: ' + e.message, false); }
}

{
  const saved = localStorage.getItem('activeTab');
  const btn = saved && document.querySelector(`.tabs button[data-tab="${saved}"]`);
  if (btn) btn.click();
  else loadItems();
}
</script>
</body>
</html>"##;

    async fn list_items(State(app): State<SharedApp>) -> impl IntoResponse {
        match app.list_items() {
            Ok(items) => Json(items).into_response(),
            Err(e) => error_response(e).into_response(),
        }
    }

    async fn create_item(
        State(app): State<SharedApp>,
        Json(req): Json<ItemRequest>,
    ) -> impl IntoResponse {
        let mut item = req.into_item();
        let result = if let Some(shelf_id) = item.shelf_id {
            app.add_item_to_shelf(&mut item, shelf_id)
        } else {
            app.add_item(&item)
        };
        match result {
            Ok(id) => {
                let created = app.get_item(id).unwrap();
                (StatusCode::CREATED, Json(created)).into_response()
            }
            Err(e) => error_response(e).into_response(),
        }
    }

    async fn get_item(State(app): State<SharedApp>, Path(id): Path<i64>) -> impl IntoResponse {
        match app.get_item(id) {
            Ok(item) => Json(item).into_response(),
            Err(e) => error_response(e).into_response(),
        }
    }

    async fn update_item(
        State(app): State<SharedApp>,
        Path(id): Path<i64>,
        Json(req): Json<ItemRequest>,
    ) -> impl IntoResponse {
        let mut item = req.into_item_with_id(id);
        let result = if let Some(shelf_id) = item.shelf_id {
            app.assign_shelf_to_item(&mut item, shelf_id)
        } else {
            app.update_item(&item)
        };
        match result {
            Ok(()) => Json(app.get_item(id).unwrap()).into_response(),
            Err(e) => error_response(e).into_response(),
        }
    }

    async fn remove_item(State(app): State<SharedApp>, Path(id): Path<i64>) -> impl IntoResponse {
        match app.remove_item(id) {
            Ok(()) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => error_response(e).into_response(),
        }
    }

    async fn shopping_list(State(app): State<SharedApp>) -> impl IntoResponse {
        match app.generate_shopping_list() {
            Ok(list) => Json(list).into_response(),
            Err(e) => error_response(e).into_response(),
        }
    }

    // --- Location endpoints ---

    async fn list_locations(State(app): State<SharedApp>) -> impl IntoResponse {
        match app.list_locations() {
            Ok(locations) => Json(locations).into_response(),
            Err(e) => error_response(e).into_response(),
        }
    }

    async fn create_location(
        State(app): State<SharedApp>,
        Json(req): Json<LocationRequest>,
    ) -> impl IntoResponse {
        let location = Location::new(req.name, req.temperature_f);
        match app.add_location(&location) {
            Ok(id) => {
                let created = app.get_location(id).unwrap();
                (StatusCode::CREATED, Json(created)).into_response()
            }
            Err(e) => error_response(e).into_response(),
        }
    }

    async fn get_location(State(app): State<SharedApp>, Path(id): Path<i64>) -> impl IntoResponse {
        match app.get_location(id) {
            Ok(location) => Json(location).into_response(),
            Err(e) => error_response(e).into_response(),
        }
    }

    async fn update_location(
        State(app): State<SharedApp>,
        Path(id): Path<i64>,
        Json(req): Json<LocationRequest>,
    ) -> impl IntoResponse {
        let location = Location {
            id: Some(id),
            name: req.name,
            temperature_f: req.temperature_f,
        };
        match app.update_location(&location) {
            Ok(()) => Json(app.get_location(id).unwrap()).into_response(),
            Err(e) => error_response(e).into_response(),
        }
    }

    async fn remove_location(
        State(app): State<SharedApp>,
        Path(id): Path<i64>,
    ) -> impl IntoResponse {
        match app.remove_location(id) {
            Ok(()) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => error_response(e).into_response(),
        }
    }

    // --- Shelf endpoints ---

    async fn list_shelves(
        State(app): State<SharedApp>,
        Path(location_id): Path<i64>,
    ) -> impl IntoResponse {
        match app.list_shelves(location_id) {
            Ok(shelves) => Json(shelves).into_response(),
            Err(e) => error_response(e).into_response(),
        }
    }

    async fn list_all_shelves(State(app): State<SharedApp>) -> impl IntoResponse {
        match app.list_all_shelves() {
            Ok(shelves) => Json(shelves).into_response(),
            Err(e) => error_response(e).into_response(),
        }
    }

    async fn create_shelf(
        State(app): State<SharedApp>,
        Path(location_id): Path<i64>,
        Json(req): Json<ShelfRequest>,
    ) -> impl IntoResponse {
        let shelf = Shelf::new(location_id, req.name);
        match app.add_shelf(&shelf) {
            Ok(id) => {
                let created = app.get_shelf(id).unwrap();
                (StatusCode::CREATED, Json(created)).into_response()
            }
            Err(e) => error_response(e).into_response(),
        }
    }

    async fn get_shelf(State(app): State<SharedApp>, Path(id): Path<i64>) -> impl IntoResponse {
        match app.get_shelf(id) {
            Ok(shelf) => Json(shelf).into_response(),
            Err(e) => error_response(e).into_response(),
        }
    }

    async fn remove_shelf(State(app): State<SharedApp>, Path(id): Path<i64>) -> impl IntoResponse {
        match app.remove_shelf(id) {
            Ok(()) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => error_response(e).into_response(),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::config::{Config, DatabaseConfig};
        use axum::body::Body;
        use axum::http::Request;
        use http_body_util::BodyExt;
        use std::path::PathBuf;
        use tower::ServiceExt;

        fn test_app() -> SharedApp {
            let repo = SqliteRepository::in_memory().unwrap();
            let shopping = DefaultShoppingListGenerator;
            let config = Config {
                database: DatabaseConfig {
                    path: PathBuf::from(":memory:"),
                },
                web: Default::default(),
                shopping: Default::default(),
                anthropic: Default::default(),
            };
            Arc::new(App::new(repo, shopping, config))
        }

        fn test_router() -> Router {
            create_router(test_app())
        }

        async fn body_to_string(body: Body) -> String {
            let bytes = body.collect().await.unwrap().to_bytes();
            String::from_utf8(bytes.to_vec()).unwrap()
        }

        fn item_json() -> serde_json::Value {
            serde_json::json!({
                "name": "Milk",
                "quantity": 2,
                "unit": "gallons",
                "category": "Dairy",
                "minimum_stock": 1
            })
        }

        #[tokio::test]
        async fn index_returns_html() {
            let router = test_router();
            let req = Request::get("/").body(Body::empty()).unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body = body_to_string(resp.into_body()).await;
            assert!(body.contains("Grocery Inventory"));
        }

        #[tokio::test]
        async fn health_check() {
            let router = test_router();
            let req = Request::get("/health").body(Body::empty()).unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body = body_to_string(resp.into_body()).await;
            assert_eq!(body, "ok");
        }

        #[tokio::test]
        async fn create_item_returns_201() {
            let router = test_router();
            let req = Request::post("/api/items")
                .header("content-type", "application/json")
                .body(Body::from(item_json().to_string()))
                .unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::CREATED);

            let body = body_to_string(resp.into_body()).await;
            let item: serde_json::Value = serde_json::from_str(&body).unwrap();
            assert_eq!(item["name"], "Milk");
            assert_eq!(item["quantity"], 2);
            assert!(item["id"].as_i64().is_some());
        }

        #[tokio::test]
        async fn get_item_found() {
            let app = test_app();
            let id = app
                .add_item(&GroceryItem::new("Eggs", 12, "count"))
                .unwrap();

            let router = create_router(app);
            let req = Request::get(format!("/api/items/{id}"))
                .body(Body::empty())
                .unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);

            let body = body_to_string(resp.into_body()).await;
            let item: serde_json::Value = serde_json::from_str(&body).unwrap();
            assert_eq!(item["name"], "Eggs");
        }

        #[tokio::test]
        async fn get_item_not_found_returns_404() {
            let router = test_router();
            let req = Request::get("/api/items/999").body(Body::empty()).unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        }

        #[tokio::test]
        async fn update_item_success() {
            let app = test_app();
            let id = app.add_item(&GroceryItem::new("Bread", 1, "loaf")).unwrap();

            let router = create_router(app);
            let update = serde_json::json!({
                "name": "Bread",
                "quantity": 3,
                "unit": "loaves",
                "minimum_stock": 1
            });
            let req = Request::put(format!("/api/items/{id}"))
                .header("content-type", "application/json")
                .body(Body::from(update.to_string()))
                .unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);

            let body = body_to_string(resp.into_body()).await;
            let item: serde_json::Value = serde_json::from_str(&body).unwrap();
            assert_eq!(item["quantity"], 3);
            assert_eq!(item["unit"], "loaves");
        }

        #[tokio::test]
        async fn update_nonexistent_returns_404() {
            let router = test_router();
            let update = serde_json::json!({
                "name": "Ghost",
                "quantity": 1,
                "unit": "box"
            });
            let req = Request::put("/api/items/999")
                .header("content-type", "application/json")
                .body(Body::from(update.to_string()))
                .unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        }

        #[tokio::test]
        async fn delete_item_returns_204() {
            let app = test_app();
            let id = app
                .add_item(&GroceryItem::new("Butter", 2, "sticks"))
                .unwrap();

            let router = create_router(app);
            let req = Request::delete(format!("/api/items/{id}"))
                .body(Body::empty())
                .unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        }

        #[tokio::test]
        async fn delete_nonexistent_returns_404() {
            let router = test_router();
            let req = Request::delete("/api/items/999")
                .body(Body::empty())
                .unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        }

        #[tokio::test]
        async fn list_items_returns_all() {
            let app = test_app();
            app.add_item(&GroceryItem::new("A", 1, "x")).unwrap();
            app.add_item(&GroceryItem::new("B", 2, "y")).unwrap();

            let router = create_router(app);
            let req = Request::get("/api/items").body(Body::empty()).unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);

            let body = body_to_string(resp.into_body()).await;
            let items: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
            assert_eq!(items.len(), 2);
        }

        #[tokio::test]
        async fn create_with_invalid_json_returns_error() {
            let router = test_router();
            let req = Request::post("/api/items")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name": "Milk"}"#))
                .unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
        }

        #[tokio::test]
        async fn shopping_list_endpoint() {
            let app = test_app();
            app.add_item(&GroceryItem::new("Rice", 10, "lbs")).unwrap();
            app.add_item(&GroceryItem::new("Salt", 1, "box")).unwrap();

            let router = create_router(app);
            let req = Request::get("/api/shopping").body(Body::empty()).unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);

            let body = body_to_string(resp.into_body()).await;
            let list: serde_json::Value = serde_json::from_str(&body).unwrap();
            let entries = list["entries"].as_array().unwrap();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0]["name"], "Salt");
        }

        #[tokio::test]
        async fn create_location_returns_201() {
            let router = test_router();
            let body = serde_json::json!({
                "name": "Fridge",
                "temperature_f": 37.0
            });
            let req = Request::post("/api/locations")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::CREATED);

            let body = body_to_string(resp.into_body()).await;
            let loc: serde_json::Value = serde_json::from_str(&body).unwrap();
            assert_eq!(loc["name"], "Fridge");
            assert!(loc["id"].as_i64().is_some());
        }

        #[tokio::test]
        async fn list_locations_endpoint() {
            let app = test_app();
            app.add_location(&Location::new("Fridge", 37.0)).unwrap();
            app.add_location(&Location::new("Pantry", 68.0)).unwrap();

            let router = create_router(app);
            let req = Request::get("/api/locations").body(Body::empty()).unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);

            let body = body_to_string(resp.into_body()).await;
            let locs: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
            assert_eq!(locs.len(), 2);
        }

        #[tokio::test]
        async fn delete_location_returns_204() {
            let app = test_app();
            let id = app.add_location(&Location::new("Freezer", 0.0)).unwrap();

            let router = create_router(app);
            let req = Request::delete(format!("/api/locations/{id}"))
                .body(Body::empty())
                .unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        }

        #[tokio::test]
        async fn get_nonexistent_location_returns_404() {
            let router = test_router();
            let req = Request::get("/api/locations/999")
                .body(Body::empty())
                .unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        }

        #[tokio::test]
        async fn create_item_with_location() {
            let app = test_app();
            let loc_id = app.add_location(&Location::new("Fridge", 37.0)).unwrap();

            let router = create_router(app);
            let body = serde_json::json!({
                "name": "Milk",
                "quantity": 2,
                "unit": "gallons",
                "location_id": loc_id
            });
            let req = Request::post("/api/items")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::CREATED);

            let body = body_to_string(resp.into_body()).await;
            let item: serde_json::Value = serde_json::from_str(&body).unwrap();
            assert_eq!(item["location_id"], loc_id);
        }

        #[tokio::test]
        async fn create_shelf_returns_201() {
            let app = test_app();
            let loc_id = app.add_location(&Location::new("Fridge", 37.0)).unwrap();

            let router = create_router(app);
            let body = serde_json::json!({ "name": "Top Shelf" });
            let req = Request::post(format!("/api/locations/{loc_id}/shelves"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::CREATED);

            let body = body_to_string(resp.into_body()).await;
            let shelf: serde_json::Value = serde_json::from_str(&body).unwrap();
            assert_eq!(shelf["name"], "Top Shelf");
            assert_eq!(shelf["location_id"], loc_id);
        }

        #[tokio::test]
        async fn list_shelves_for_location() {
            let app = test_app();
            let loc_id = app.add_location(&Location::new("Fridge", 37.0)).unwrap();
            app.add_shelf(&Shelf::new(loc_id, "Top")).unwrap();
            app.add_shelf(&Shelf::new(loc_id, "Bottom")).unwrap();

            let router = create_router(app);
            let req = Request::get(format!("/api/locations/{loc_id}/shelves"))
                .body(Body::empty())
                .unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);

            let body = body_to_string(resp.into_body()).await;
            let shelves: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
            assert_eq!(shelves.len(), 2);
        }

        #[tokio::test]
        async fn delete_shelf_returns_204() {
            let app = test_app();
            let loc_id = app.add_location(&Location::new("Fridge", 37.0)).unwrap();
            let shelf_id = app.add_shelf(&Shelf::new(loc_id, "Top")).unwrap();

            let router = create_router(app);
            let req = Request::delete(format!("/api/shelves/{shelf_id}"))
                .body(Body::empty())
                .unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        }

        #[tokio::test]
        async fn create_item_with_shelf_sets_location() {
            let app = test_app();
            let loc_id = app.add_location(&Location::new("Fridge", 37.0)).unwrap();
            let shelf_id = app.add_shelf(&Shelf::new(loc_id, "Top")).unwrap();

            let router = create_router(app);
            let body = serde_json::json!({
                "name": "Milk",
                "quantity": 2,
                "unit": "gallons",
                "shelf_id": shelf_id
            });
            let req = Request::post("/api/items")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::CREATED);

            let body = body_to_string(resp.into_body()).await;
            let item: serde_json::Value = serde_json::from_str(&body).unwrap();
            assert_eq!(item["shelf_id"], shelf_id);
            assert_eq!(item["location_id"], loc_id);
        }

        #[tokio::test]
        async fn get_nonexistent_shelf_returns_404() {
            let router = test_router();
            let req = Request::get("/api/shelves/999")
                .body(Body::empty())
                .unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        }
    }
}
