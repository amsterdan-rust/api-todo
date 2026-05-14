use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use super::{
    domain::TodoId,
    dto::{CreateTodoInput, CreateTodoRequest, TodoResponse, UpdateTodoInput, UpdateTodoRequest},
    service::TodoService,
};
use crate::shared::error::AppError;

pub async fn create_todo(
    State(service): State<TodoService>,
    Json(request): Json<CreateTodoRequest>,
) -> Result<(StatusCode, Json<TodoResponse>), AppError> {
    let input: CreateTodoInput = request.into();

    let todo = service.create_todo(input).await?;

    Ok((StatusCode::CREATED, Json(todo.into())))
}

pub async fn list_todos(
    State(service): State<TodoService>,
) -> Result<Json<Vec<TodoResponse>>, AppError> {
    let todos = service.list_todos().await?;

    let response = todos.into_iter().map(TodoResponse::from).collect();

    Ok(Json(response))
}

pub async fn get_todo(
    State(service): State<TodoService>,
    Path(id): Path<Uuid>,
) -> Result<Json<TodoResponse>, AppError> {
    let id = TodoId::from_uuid(id);

    let todo = service.get_todo(id).await?;

    Ok(Json(todo.into()))
}

pub async fn update_todo(
    State(service): State<TodoService>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateTodoRequest>,
) -> Result<Json<TodoResponse>, AppError> {
    let id = TodoId::from_uuid(id);
    let input: UpdateTodoInput = request.into();

    let todo = service.update_todo(id, input).await?;

    Ok(Json(todo.into()))
}

pub async fn delete_todo(
    State(service): State<TodoService>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let id = TodoId::from_uuid(id);

    service.delete_todo(id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn complete_todo(
    State(service): State<TodoService>,
    Path(id): Path<Uuid>,
) -> Result<Json<TodoResponse>, AppError> {
    let id = TodoId::from_uuid(id);

    let todo = service.complete_todo(id).await?;

    Ok(Json(todo.into()))
}

pub async fn reopen_todo(
    State(service): State<TodoService>,
    Path(id): Path<Uuid>,
) -> Result<Json<TodoResponse>, AppError> {
    let id = TodoId::from_uuid(id);

    let todo = service.reopen_todo(id).await?;

    Ok(Json(todo.into()))
}
