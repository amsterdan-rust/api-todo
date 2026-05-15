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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::todo::infra::in_memory_repository::InMemoryTodoRepository;

    fn make_service() -> TodoService {
        let repository = Arc::new(InMemoryTodoRepository::new());

        TodoService::new(repository)
    }

    #[tokio::test]
    async fn should_create_todo() {
        let service = make_service();

        let input = CreateTodoInput {
            title: "Estudar Axum".to_string(),
            description: Some("Criar uma API todo list".to_string()),
        };

        let todo = service.create_todo(input).await.unwrap();

        assert_eq!(todo.title().as_str(), "Estudar Axum");
        assert_eq!(
            todo.description().as_deref(),
            Some("Criar uma API todo list")
        );
        assert!(!todo.completed());
    }

    #[tokio::test]
    async fn should_reject_todo_with_empty_title() {
        let service = make_service();

        let input = CreateTodoInput {
            title: "   ".to_string(),
            description: None,
        };

        let result = service.create_todo(input).await;

        assert!(matches!(result, Err(TodoError::InvalidTitle)));
    }

    #[tokio::test]
    async fn should_list_todos() {
        let service = make_service();

        let first_input = CreateTodoInput {
            title: "Primeira tarefa".to_string(),
            description: None,
        };

        let second_input = CreateTodoInput {
            title: "Segunda tarefa".to_string(),
            description: None,
        };

        service.create_todo(first_input).await.unwrap();
        service.create_todo(second_input).await.unwrap();

        let todos = service.list_todos().await.unwrap();

        assert_eq!(todos.len(), 2);
    }

    #[tokio::test]
    async fn should_get_todo_by_id() {
        let service = make_service();

        let input = CreateTodoInput {
            title: "Buscar tarefa".to_string(),
            description: None,
        };

        let created_todo = service.create_todo(input).await.unwrap();

        let found_todo = service.get_todo(created_todo.id()).await.unwrap();

        assert_eq!(found_todo.id(), created_todo.id());
        assert_eq!(found_todo.title().as_str(), "Buscar tarefa");
    }

    #[tokio::test]
    async fn should_return_not_found_when_todo_does_not_exist() {
        let service = make_service();

        let result = service.get_todo(TodoId::new()).await;

        assert!(matches!(result, Err(TodoError::NotFound)));
    }

    #[tokio::test]
    async fn should_update_todo() {
        let service = make_service();

        let input = CreateTodoInput {
            title: "Título antigo".to_string(),
            description: Some("Descrição antiga".to_string()),
        };

        let todo = service.create_todo(input).await.unwrap();

        let update_input = UpdateTodoInput {
            title: Some("Título novo".to_string()),
            description: Some(Some("Descrição nova".to_string())),
        };

        let updated_todo = service.update_todo(todo.id(), update_input).await.unwrap();

        assert_eq!(updated_todo.title().as_str(), "Título novo");
        assert_eq!(
            updated_todo.description().as_deref(),
            Some("Descrição nova")
        );
    }

    #[tokio::test]
    async fn should_update_only_todo_title() {
        let service = make_service();

        let input = CreateTodoInput {
            title: "Título antigo".to_string(),
            description: Some("Descrição original".to_string()),
        };

        let todo = service.create_todo(input).await.unwrap();

        let update_input = UpdateTodoInput {
            title: Some("Título novo".to_string()),
            description: None,
        };

        let updated_todo = service.update_todo(todo.id(), update_input).await.unwrap();

        assert_eq!(updated_todo.title().as_str(), "Título novo");
        assert_eq!(
            updated_todo.description().as_deref(),
            Some("Descrição original")
        );
    }

    #[tokio::test]
    async fn should_clear_todo_description() {
        let service = make_service();

        let input = CreateTodoInput {
            title: "Tarefa com descrição".to_string(),
            description: Some("Descrição original".to_string()),
        };

        let todo = service.create_todo(input).await.unwrap();

        let update_input = UpdateTodoInput {
            title: None,
            description: Some(None),
        };

        let updated_todo = service.update_todo(todo.id(), update_input).await.unwrap();

        assert_eq!(updated_todo.description().as_deref(), None);
    }

    #[tokio::test]
    async fn should_complete_todo() {
        let service = make_service();

        let input = CreateTodoInput {
            title: "Completar tarefa".to_string(),
            description: None,
        };

        let todo = service.create_todo(input).await.unwrap();

        let completed_todo = service.complete_todo(todo.id()).await.unwrap();

        assert!(completed_todo.completed());
    }

    #[tokio::test]
    async fn should_not_complete_already_completed_todo() {
        let service = make_service();

        let input = CreateTodoInput {
            title: "Completar tarefa".to_string(),
            description: None,
        };

        let todo = service.create_todo(input).await.unwrap();

        service.complete_todo(todo.id()).await.unwrap();

        let result = service.complete_todo(todo.id()).await;

        assert!(matches!(result, Err(TodoError::AlreadyCompleted)));
    }

    #[tokio::test]
    async fn should_reopen_todo() {
        let service = make_service();

        let input = CreateTodoInput {
            title: "Reabrir tarefa".to_string(),
            description: None,
        };

        let todo = service.create_todo(input).await.unwrap();

        service.complete_todo(todo.id()).await.unwrap();

        let reopened_todo = service.reopen_todo(todo.id()).await.unwrap();

        assert!(!reopened_todo.completed());
    }

    #[tokio::test]
    async fn should_not_reopen_open_todo() {
        let service = make_service();

        let input = CreateTodoInput {
            title: "Tarefa aberta".to_string(),
            description: None,
        };

        let todo = service.create_todo(input).await.unwrap();

        let result = service.reopen_todo(todo.id()).await;

        assert!(matches!(result, Err(TodoError::AlreadyOpen)));
    }

    #[tokio::test]
    async fn should_delete_todo() {
        let service = make_service();

        let input = CreateTodoInput {
            title: "Deletar tarefa".to_string(),
            description: None,
        };

        let todo = service.create_todo(input).await.unwrap();

        service.delete_todo(todo.id()).await.unwrap();

        let result = service.get_todo(todo.id()).await;

        assert!(matches!(result, Err(TodoError::NotFound)));
    }
}
