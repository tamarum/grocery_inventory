#[cfg(feature = "web")]
pub mod routes {
    use axum::{
        extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router,
    };
    use std::sync::Arc;

    use crate::app::App;
    use crate::db::SqliteRepository;
    use crate::shopping::DefaultShoppingListGenerator;

    type SharedApp = Arc<App<SqliteRepository, DefaultShoppingListGenerator>>;

    pub fn create_router(app: SharedApp) -> Router {
        Router::new()
            .route("/api/items", get(list_items))
            .route("/api/shopping", get(shopping_list))
            .route("/health", get(health))
            .with_state(app)
    }

    async fn health() -> &'static str {
        "ok"
    }

    async fn list_items(State(app): State<SharedApp>) -> impl IntoResponse {
        match app.list_items() {
            Ok(items) => Json(items).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }

    async fn shopping_list(State(app): State<SharedApp>) -> impl IntoResponse {
        match app.generate_shopping_list() {
            Ok(list) => Json(list).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
}
