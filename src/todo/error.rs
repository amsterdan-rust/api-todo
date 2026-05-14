use thiserror::Error;

#[derive(Debug, Error)]
pub enum TodoError {
    #[error("todo not found")]
    NotFound,

    #[error("todo title cannot be empty")]
    InvalidTitle,

    #[error("todo title cannot be longer than 100 characters")]
    TitleTooLong,

    #[error("todo is already completed")]
    AlreadyCompleted,

    #[error("todo is already open")]
    AlreadyOpen,

    #[error("unexpected repository error")]
    Repository,
}
