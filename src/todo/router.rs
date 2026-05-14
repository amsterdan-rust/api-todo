use std::sync::Arc;

use axum::{
    Router,
    routing::{get, patch},
};

use super::{
    handler::{
        complete_todo, create_todo, delete_todo, get_todo, list_todos, reopen_todo, update_todo,
    },
    repository::InMemoryTodoRepository,
    service::TodoService,
};

pub fn routes() -> Router {
    let repository = Arc::new(InMemoryTodoRepository::new());
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
