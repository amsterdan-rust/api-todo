use time::OffsetDateTime;
use uuid::Uuid;

use super::error::TodoError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TodoId(Uuid);

impl TodoId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for TodoId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoTitle(String);

impl TodoTitle {
    pub fn new(value: String) -> Result<Self, TodoError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(TodoError::InvalidTitle);
        }

        if value.len() > 100 {
            return Err(TodoError::TitleTooLong);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoDescription(Option<String>);

impl TodoDescription {
    pub fn new(value: Option<String>) -> Self {
        let value = value
            .map(|description| description.trim().to_string())
            .filter(|description| !description.is_empty());

        Self(value)
    }

    pub fn as_deref(&self) -> Option<&str> {
        self.0.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    id: TodoId,
    title: TodoTitle,
    description: TodoDescription,
    completed: bool,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl Todo {
    pub fn new(title: TodoTitle, description: TodoDescription) -> Self {
        let now = OffsetDateTime::now_utc();

        Self {
            id: TodoId::new(),
            title,
            description,
            completed: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn restore(
        id: TodoId,
        title: TodoTitle,
        description: TodoDescription,
        completed: bool,
        created_at: OffsetDateTime,
        updated_at: OffsetDateTime,
    ) -> Self {
        Self {
            id,
            title,
            description,
            completed,
            created_at,
            updated_at,
        }
    }

    pub fn update(&mut self, title: Option<TodoTitle>, description: Option<TodoDescription>) {
        if let Some(title) = title {
            self.title = title;
        }

        if let Some(description) = description {
            self.description = description;
        }

        self.touch();
    }

    pub fn complete(&mut self) -> Result<(), TodoError> {
        if self.completed {
            return Err(TodoError::AlreadyCompleted);
        }

        self.completed = true;
        self.touch();

        Ok(())
    }

    pub fn reopen(&mut self) -> Result<(), TodoError> {
        if !self.completed {
            return Err(TodoError::AlreadyOpen);
        }

        self.completed = false;
        self.touch();

        Ok(())
    }

    fn touch(&mut self) {
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn id(&self) -> TodoId {
        self.id
    }

    pub fn title(&self) -> &TodoTitle {
        &self.title
    }

    pub fn description(&self) -> &TodoDescription {
        &self.description
    }

    pub fn completed(&self) -> bool {
        self.completed
    }

    pub fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }

    pub fn updated_at(&self) -> OffsetDateTime {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_valid_todo_title() {
        let title = TodoTitle::new("Estudar Axum".to_string()).unwrap();

        assert_eq!(title.as_str(), "Estudar Axum");
    }

    #[test]
    fn should_trim_todo_title() {
        let title = TodoTitle::new("   Estudar Axum   ".to_string()).unwrap();

        assert_eq!(title.as_str(), "Estudar Axum");
    }

    #[test]
    fn should_reject_empty_todo_title() {
        let result = TodoTitle::new("   ".to_string());

        assert!(matches!(result, Err(TodoError::InvalidTitle)));
    }

    #[test]
    fn should_reject_todo_title_longer_than_100_characters() {
        let title = "a".repeat(101);

        let result = TodoTitle::new(title);

        assert!(matches!(result, Err(TodoError::TitleTooLong)));
    }

    #[test]
    fn should_create_description_with_text() {
        let description = TodoDescription::new(Some("Comprar leite".to_string()));

        assert_eq!(description.as_deref(), Some("Comprar leite"));
    }

    #[test]
    fn should_trim_description() {
        let description = TodoDescription::new(Some("   Comprar leite   ".to_string()));

        assert_eq!(description.as_deref(), Some("Comprar leite"));
    }

    #[test]
    fn should_convert_empty_description_to_none() {
        let description = TodoDescription::new(Some("   ".to_string()));

        assert_eq!(description.as_deref(), None);
    }

    #[test]
    fn should_create_new_todo_as_open() {
        let title = TodoTitle::new("Estudar Rust".to_string()).unwrap();
        let description = TodoDescription::new(None);

        let todo = Todo::new(title, description);

        assert!(!todo.completed());
        assert_eq!(todo.title().as_str(), "Estudar Rust");
        assert_eq!(todo.description().as_deref(), None);
    }

    #[test]
    fn should_complete_todo() {
        let title = TodoTitle::new("Estudar Rust".to_string()).unwrap();
        let description = TodoDescription::new(None);
        let mut todo = Todo::new(title, description);

        let result = todo.complete();

        assert!(result.is_ok());
        assert!(todo.completed());
    }

    #[test]
    fn should_not_complete_already_completed_todo() {
        let title = TodoTitle::new("Estudar Rust".to_string()).unwrap();
        let description = TodoDescription::new(None);
        let mut todo = Todo::new(title, description);

        todo.complete().unwrap();

        let result = todo.complete();

        assert!(matches!(result, Err(TodoError::AlreadyCompleted)));
    }

    #[test]
    fn should_reopen_completed_todo() {
        let title = TodoTitle::new("Estudar Rust".to_string()).unwrap();
        let description = TodoDescription::new(None);
        let mut todo = Todo::new(title, description);

        todo.complete().unwrap();

        let result = todo.reopen();

        assert!(result.is_ok());
        assert!(!todo.completed());
    }

    #[test]
    fn should_not_reopen_open_todo() {
        let title = TodoTitle::new("Estudar Rust".to_string()).unwrap();
        let description = TodoDescription::new(None);
        let mut todo = Todo::new(title, description);

        let result = todo.reopen();

        assert!(matches!(result, Err(TodoError::AlreadyOpen)));
    }

    #[test]
    fn should_update_todo_title_and_description() {
        let title = TodoTitle::new("Título antigo".to_string()).unwrap();
        let description = TodoDescription::new(Some("Descrição antiga".to_string()));
        let mut todo = Todo::new(title, description);

        let new_title = TodoTitle::new("Título novo".to_string()).unwrap();
        let new_description = TodoDescription::new(Some("Descrição nova".to_string()));

        todo.update(Some(new_title), Some(new_description));

        assert_eq!(todo.title().as_str(), "Título novo");
        assert_eq!(todo.description().as_deref(), Some("Descrição nova"));
    }
}
