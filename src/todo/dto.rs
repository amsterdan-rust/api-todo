use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use super::domain::Todo;

#[derive(Debug, Deserialize)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
}

#[derive(Debug)]
pub struct CreateTodoInput {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct UpdateTodoInput {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
}

#[derive(Debug, Serialize)]
pub struct TodoResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,

    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<CreateTodoRequest> for CreateTodoInput {
    fn from(request: CreateTodoRequest) -> Self {
        Self {
            title: request.title,
            description: request.description,
        }
    }
}

impl From<UpdateTodoRequest> for UpdateTodoInput {
    fn from(request: UpdateTodoRequest) -> Self {
        Self {
            title: request.title,
            description: request.description,
        }
    }
}

impl From<Todo> for TodoResponse {
    fn from(todo: Todo) -> Self {
        Self {
            id: todo.id().as_uuid(),
            title: todo.title().as_str().to_string(),
            description: todo.description().as_deref().map(str::to_string),
            completed: todo.completed(),
            created_at: todo.created_at(),
            updated_at: todo.updated_at(),
        }
    }
}
