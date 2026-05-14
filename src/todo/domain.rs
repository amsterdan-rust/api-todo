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
