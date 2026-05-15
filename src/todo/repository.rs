use async_trait::async_trait;

use super::{
    domain::{Todo, TodoId},
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
