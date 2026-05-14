use thiserror::Error;

#[derive(Debug, Error)]
pub enum TodoError {
    #[error("tarefa não encontrada")]
    NotFound,

    #[error("o título da tarefa não pode estar vazio")]
    InvalidTitle,

    #[error("o título da tarefa não pode ter mais de 100 caracteres")]
    TitleTooLong,

    #[error("a tarefa já está concluída")]
    AlreadyCompleted,

    #[error("a tarefa já está aberta")]
    AlreadyOpen,

    #[error("erro inesperado no repositório")]
    Repository,
}
