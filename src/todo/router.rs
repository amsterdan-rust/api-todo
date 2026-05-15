use std::sync::Arc;

use axum::{
    Router,
    routing::{get, patch},
};
use sqlx::PgPool;

use super::{
    handler::{
        complete_todo, create_todo, delete_todo, get_todo, list_todos, reopen_todo, update_todo,
    },
    repository::PostgresTodoRepository,
    service::TodoService,
};

pub fn routes(db_pool: PgPool) -> Router {
    let repository = Arc::new(PostgresTodoRepository::new(db_pool));
    let service = TodoService::new(repository);

    Router::new()
        .route("/", get(list_todos).post(create_todo))
        .route(
            "/{id}",
            get(get_todo).patch(update_todo).delete(delete_todo),
        )
        .route("/{id}/complete", patch(complete_todo))
        .route("/{id}/reopen", patch(reopen_todo))
        .with_state(service)
}
