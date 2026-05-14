use axum::{
    Router,
    routing::{get, patch},
};

use super::{
    handler::{
        complete_todo, create_todo, delete_todo, get_todo, list_todos, reopen_todo, update_todo,
    },
    service::TodoService,
};

pub fn routes() -> Router<TodoService> {
    Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .route(
            "/todos/{id}",
            get(get_todo).patch(update_todo).delete(delete_todo),
        )
        .route("/todos/{id}/complete", patch(complete_todo))
        .route("/todos/{id}/reopen", patch(reopen_todo))
}
