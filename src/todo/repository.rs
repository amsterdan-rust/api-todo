use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use sqlx::PgPool;
use time::OffsetDateTime;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{
    domain::{Todo, TodoDescription, TodoId, TodoTitle},
    error::TodoError,
};

#[async_trait]
pub trait TodoRepository: Send + Sync {
    async fn create(&self, todo: Todo) -> Result<Todo, TodoError>;

    async fn find_all(&self) -> Result<Vec<Todo>, TodoError>;

    async fn find_by_id(&self, id: TodoId) -> Result<Option<Todo>, TodoError>;

    async fn update(&self, todo: Todo) -> Result<Todo, TodoError>;

    async fn delete(&self, id: TodoId) -> Result<(), TodoError>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryTodoRepository {
    todos: Arc<RwLock<HashMap<TodoId, Todo>>>,
}

impl InMemoryTodoRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl TodoRepository for InMemoryTodoRepository {
    async fn create(&self, todo: Todo) -> Result<Todo, TodoError> {
        let mut todos = self.todos.write().await;

        todos.insert(todo.id(), todo.clone());

        Ok(todo)
    }

    async fn find_all(&self) -> Result<Vec<Todo>, TodoError> {
        let todos = self.todos.read().await;

        Ok(todos.values().cloned().collect())
    }

    async fn find_by_id(&self, id: TodoId) -> Result<Option<Todo>, TodoError> {
        let todos = self.todos.read().await;

        Ok(todos.get(&id).cloned())
    }

    async fn update(&self, todo: Todo) -> Result<Todo, TodoError> {
        let mut todos = self.todos.write().await;

        if !todos.contains_key(&todo.id()) {
            return Err(TodoError::NotFound);
        }

        todos.insert(todo.id(), todo.clone());

        Ok(todo)
    }

    async fn delete(&self, id: TodoId) -> Result<(), TodoError> {
        let mut todos = self.todos.write().await;

        if todos.remove(&id).is_none() {
            return Err(TodoError::NotFound);
        }

        Ok(())
    }
}

// ------------------- POSTGRES -------------------

#[derive(Debug, sqlx::FromRow)]
struct TodoRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    completed: bool,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl TryFrom<TodoRow> for Todo {
    type Error = TodoError;

    fn try_from(row: TodoRow) -> Result<Self, Self::Error> {
        let id = TodoId::from_uuid(row.id);
        let title = TodoTitle::new(row.title)?;
        let description = TodoDescription::new(row.description);

        Ok(Todo::restore(
            id,
            title,
            description,
            row.completed,
            row.created_at,
            row.updated_at,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct PostgresTodoRepository {
    db_pool: PgPool,
}

impl PostgresTodoRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl TodoRepository for PostgresTodoRepository {
    async fn create(&self, todo: Todo) -> Result<Todo, TodoError> {
        sqlx::query(
            r#"
            INSERT INTO todos (id, title, description, completed, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(todo.id().as_uuid())
        .bind(todo.title().as_str())
        .bind(todo.description().as_deref())
        .bind(todo.completed())
        .bind(todo.created_at())
        .bind(todo.updated_at())
        .execute(&self.db_pool)
        .await
        .map_err(|error| {
            tracing::error!(?error, "failed to create todo");
            TodoError::Repository
        })?;

        Ok(todo)
    }

    async fn find_all(&self) -> Result<Vec<Todo>, TodoError> {
        let rows = sqlx::query_as::<_, TodoRow>(
            r#"
            SELECT id, title, description, completed, created_at, updated_at
            FROM todos
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|error| {
            tracing::error!(?error, "failed to list todos");
            TodoError::Repository
        })?;

        rows.into_iter().map(Todo::try_from).collect()
    }

    async fn find_by_id(&self, id: TodoId) -> Result<Option<Todo>, TodoError> {
        let row = sqlx::query_as::<_, TodoRow>(
            r#"
            SELECT id, title, description, completed, created_at, updated_at
            FROM todos
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|error| {
            tracing::error!(?error, todo_id = %id.as_uuid(), "failed to find todo by id");
            TodoError::Repository
        })?;

        row.map(Todo::try_from).transpose()
    }

    async fn update(&self, todo: Todo) -> Result<Todo, TodoError> {
        let result = sqlx::query(
            r#"
            UPDATE todos
            SET title = $2,
                description = $3,
                completed = $4,
                updated_at = $5
            WHERE id = $1
            "#,
        )
        .bind(todo.id().as_uuid())
        .bind(todo.title().as_str())
        .bind(todo.description().as_deref())
        .bind(todo.completed())
        .bind(todo.updated_at())
        .execute(&self.db_pool)
        .await
        .map_err(|error| {
            tracing::error!(
                ?error,
                todo_id = %todo.id().as_uuid(),
                "failed to update todo"
            );

            TodoError::Repository
        })?;

        if result.rows_affected() == 0 {
            return Err(TodoError::NotFound);
        }

        Ok(todo)
    }

    async fn delete(&self, id: TodoId) -> Result<(), TodoError> {
        let result = sqlx::query(
            r#"
            DELETE FROM todos
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .execute(&self.db_pool)
        .await
        .map_err(|error| {
            tracing::error!(?error, todo_id = %id.as_uuid(), "failed to delete todo");
            TodoError::Repository
        })?;

        if result.rows_affected() == 0 {
            return Err(TodoError::NotFound);
        }

        Ok(())
    }
}
