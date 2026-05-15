use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::todo::{
    domain::{Todo, TodoId},
    error::TodoError,
    repository::TodoRepository,
};

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
