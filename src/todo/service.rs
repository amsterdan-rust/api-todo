use std::sync::Arc;

use super::{
    domain::{Todo, TodoDescription, TodoId, TodoTitle},
    dto::{CreateTodoInput, UpdateTodoInput},
    error::TodoError,
    repository::TodoRepository,
};

#[derive(Clone)]
pub struct TodoService {
    repository: Arc<dyn TodoRepository>,
}

impl TodoService {
    pub fn new(repository: Arc<dyn TodoRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_todo(&self, input: CreateTodoInput) -> Result<Todo, TodoError> {
        let title = TodoTitle::new(input.title)?;
        let description = TodoDescription::new(input.description);

        let todo = Todo::new(title, description);

        self.repository.create(todo).await
    }

    pub async fn list_todos(&self) -> Result<Vec<Todo>, TodoError> {
        self.repository.find_all().await
    }

    pub async fn get_todo(&self, id: TodoId) -> Result<Todo, TodoError> {
        let todo = self.repository.find_by_id(id).await?;

        todo.ok_or(TodoError::NotFound)
    }

    pub async fn update_todo(&self, id: TodoId, input: UpdateTodoInput) -> Result<Todo, TodoError> {
        let mut todo = self.get_todo(id).await?;

        let title = match input.title {
            Some(title) => Some(TodoTitle::new(title)?),
            None => None,
        };

        let description = input.description.map(TodoDescription::new);

        todo.update(title, description);

        self.repository.update(todo).await
    }

    pub async fn delete_todo(&self, id: TodoId) -> Result<(), TodoError> {
        self.repository.delete(id).await
    }

    pub async fn complete_todo(&self, id: TodoId) -> Result<Todo, TodoError> {
        let mut todo = self.get_todo(id).await?;

        todo.complete()?;

        self.repository.update(todo).await
    }

    pub async fn reopen_todo(&self, id: TodoId) -> Result<Todo, TodoError> {
        let mut todo = self.get_todo(id).await?;

        todo.reopen()?;

        self.repository.update(todo).await
    }
}
