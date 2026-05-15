use async_trait::async_trait;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::todo::{
    domain::{Todo, TodoDescription, TodoId, TodoTitle},
    error::TodoError,
    repository::TodoRepository,
};

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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Executor, PgPool, postgres::PgPoolOptions};
    use url::Url;

    struct TestDatabase {
        admin_pool: PgPool,
        db_pool: PgPool,
        database_name: String,
    }

    impl TestDatabase {
        async fn create() -> Self {
            dotenvy::dotenv().ok();

            let database_url = std::env::var("TEST_DATABASE_URL")
                .expect("TEST_DATABASE_URL environment variable must be set");

            let database_name = format!("todo_api_test_{}", Uuid::new_v4().simple());

            let admin_database_url = build_database_url(&database_url, "postgres");
            let test_database_url = build_database_url(&database_url, &database_name);

            let admin_pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(admin_database_url.as_str())
                .await
                .expect("failed to connect to admin database");

            let create_database_query = format!(r#"CREATE DATABASE "{}""#, database_name);

            admin_pool
                .execute(create_database_query.as_str())
                .await
                .expect("failed to create test database");

            let db_pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(test_database_url.as_str())
                .await
                .expect("failed to connect to test database");

            sqlx::migrate!()
                .run(&db_pool)
                .await
                .expect("failed to run migrations on test database");

            Self {
                admin_pool,
                db_pool,
                database_name,
            }
        }

        fn pool(&self) -> PgPool {
            self.db_pool.clone()
        }
    }

    impl Drop for TestDatabase {
        fn drop(&mut self) {
            let admin_pool = self.admin_pool.clone();
            let db_pool = self.db_pool.clone();
            let database_name = self.database_name.clone();

            tokio::spawn(async move {
                db_pool.close().await;

                let terminate_connections_query = format!(
                    r#"
                    SELECT pg_terminate_backend(pid)
                    FROM pg_stat_activity
                    WHERE datname = '{}'
                    "#,
                    database_name
                );

                let drop_database_query = format!(r#"DROP DATABASE IF EXISTS "{}""#, database_name);

                if let Err(error) = admin_pool
                    .execute(terminate_connections_query.as_str())
                    .await
                {
                    tracing::error!(?error, "failed to terminate test database connections");
                }

                if let Err(error) = admin_pool.execute(drop_database_query.as_str()).await {
                    tracing::error!(?error, "failed to drop test database");
                }

                admin_pool.close().await;
            });
        }
    }

    fn build_database_url(database_url: &str, database_name: &str) -> String {
        let mut url = Url::parse(database_url).expect("DATABASE_URL must be a valid URL");

        url.set_path(database_name);

        url.to_string()
    }

    fn make_todo(title: &str) -> Todo {
        let title = TodoTitle::new(title.to_string()).unwrap();
        let description = TodoDescription::new(Some("Descrição teste".to_string()));

        Todo::new(title, description)
    }

    #[tokio::test]
    async fn should_create_todo_in_postgres() {
        let test_database = TestDatabase::create().await;
        let repository = PostgresTodoRepository::new(test_database.pool());

        let todo = make_todo("Criar tarefa no Postgres");

        let created_todo = repository.create(todo).await.unwrap();

        assert_eq!(created_todo.title().as_str(), "Criar tarefa no Postgres");
        assert_eq!(
            created_todo.description().as_deref(),
            Some("Descrição teste")
        );
        assert!(!created_todo.completed());
    }

    #[tokio::test]
    async fn should_find_todo_by_id_in_postgres() {
        let test_database = TestDatabase::create().await;
        let repository = PostgresTodoRepository::new(test_database.pool());

        let todo = make_todo("Buscar tarefa no Postgres");
        let created_todo = repository.create(todo).await.unwrap();

        let found_todo = repository
            .find_by_id(created_todo.id())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(found_todo.id(), created_todo.id());
        assert_eq!(found_todo.title().as_str(), "Buscar tarefa no Postgres");
    }

    #[tokio::test]
    async fn should_list_todos_from_postgres() {
        let test_database = TestDatabase::create().await;
        let repository = PostgresTodoRepository::new(test_database.pool());

        repository
            .create(make_todo("Primeira tarefa"))
            .await
            .unwrap();

        repository
            .create(make_todo("Segunda tarefa"))
            .await
            .unwrap();

        let todos = repository.find_all().await.unwrap();

        assert_eq!(todos.len(), 2);
    }

    #[tokio::test]
    async fn should_update_todo_in_postgres() {
        let test_database = TestDatabase::create().await;
        let repository = PostgresTodoRepository::new(test_database.pool());

        let mut todo = repository.create(make_todo("Título antigo")).await.unwrap();

        let new_title = TodoTitle::new("Título novo".to_string()).unwrap();
        let new_description = TodoDescription::new(Some("Descrição nova".to_string()));

        todo.update(Some(new_title), Some(new_description));

        let updated_todo = repository.update(todo).await.unwrap();

        assert_eq!(updated_todo.title().as_str(), "Título novo");
        assert_eq!(
            updated_todo.description().as_deref(),
            Some("Descrição nova")
        );
    }

    #[tokio::test]
    async fn should_delete_todo_from_postgres() {
        let test_database = TestDatabase::create().await;
        let repository = PostgresTodoRepository::new(test_database.pool());

        let todo = repository
            .create(make_todo("Deletar tarefa"))
            .await
            .unwrap();

        repository.delete(todo.id()).await.unwrap();

        let result = repository.find_by_id(todo.id()).await.unwrap();

        assert!(result.is_none());
    }
}
