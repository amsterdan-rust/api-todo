use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use sqlx::{Executor, PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;
use url::Url;
use uuid::Uuid;

use todo_api::app;

struct TestDatabase {
    admin_pool: PgPool,
    db_pool: PgPool,
    database_name: String,
}

impl TestDatabase {
    async fn create() -> Self {
        dotenvy::dotenv().ok();

        let test_database_base_url = std::env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL environment variable must be set");

        let database_name = format!("todo_api_http_test_{}", Uuid::new_v4().simple());

        let admin_database_url = build_database_url(&test_database_base_url, "postgres");
        let test_database_url = build_database_url(&test_database_base_url, &database_name);

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

            let _ = admin_pool
                .execute(terminate_connections_query.as_str())
                .await;
            let _ = admin_pool.execute(drop_database_query.as_str()).await;

            admin_pool.close().await;
        });
    }
}

fn build_database_url(database_url: &str, database_name: &str) -> String {
    let mut url = Url::parse(database_url).expect("TEST_DATABASE_URL must be a valid URL");

    url.set_path(database_name);

    url.to_string()
}

async fn response_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("failed to read response body");

    serde_json::from_slice(&body).expect("failed to parse response body as json")
}

fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn empty_request(method: &str, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn should_create_todo() {
    let test_database = TestDatabase::create().await;
    let app = app::create_app(test_database.pool());

    let response = app
        .oneshot(json_request(
            "POST",
            "/todos",
            json!({
                "title": "Lavar louça",
                "description": "Depois do almoço"
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response_json(response).await;

    assert_eq!(body["title"], "Lavar louça");
    assert_eq!(body["description"], "Depois do almoço");
    assert_eq!(body["completed"], false);
    assert!(body["id"].is_string());
    assert!(body["created_at"].is_string());
    assert!(body["updated_at"].is_string());
}

#[tokio::test]
async fn should_list_todos() {
    let test_database = TestDatabase::create().await;
    let app = app::create_app(test_database.pool());

    let app = app
        .oneshot(json_request(
            "POST",
            "/todos",
            json!({
                "title": "Primeira tarefa",
                "description": null
            }),
        ))
        .await
        .unwrap();

    assert_eq!(app.status(), StatusCode::CREATED);

    let app = app::create_app(test_database.pool());

    let response = app
        .oneshot(json_request(
            "POST",
            "/todos",
            json!({
                "title": "Segunda tarefa",
                "description": null
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let app = app::create_app(test_database.pool());

    let response = app.oneshot(empty_request("GET", "/todos")).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;

    assert!(body.is_array());
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn should_get_todo_by_id() {
    let test_database = TestDatabase::create().await;
    let app = app::create_app(test_database.pool());

    let response = app
        .oneshot(json_request(
            "POST",
            "/todos",
            json!({
                "title": "Buscar tarefa",
                "description": null
            }),
        ))
        .await
        .unwrap();

    let body = response_json(response).await;
    let id = body["id"].as_str().unwrap();

    let app = app::create_app(test_database.pool());

    let response = app
        .oneshot(empty_request("GET", &format!("/todos/{id}")))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;

    assert_eq!(body["id"], id);
    assert_eq!(body["title"], "Buscar tarefa");
}

#[tokio::test]
async fn should_update_todo() {
    let test_database = TestDatabase::create().await;
    let app = app::create_app(test_database.pool());

    let response = app
        .oneshot(json_request(
            "POST",
            "/todos",
            json!({
                "title": "Título antigo",
                "description": "Descrição antiga"
            }),
        ))
        .await
        .unwrap();

    let body = response_json(response).await;
    let id = body["id"].as_str().unwrap();

    let app = app::create_app(test_database.pool());

    let response = app
        .oneshot(json_request(
            "PATCH",
            &format!("/todos/{id}"),
            json!({
                "title": "Título novo",
                "description": "Descrição nova"
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;

    assert_eq!(body["title"], "Título novo");
    assert_eq!(body["description"], "Descrição nova");
}

#[tokio::test]
async fn should_complete_todo() {
    let test_database = TestDatabase::create().await;
    let app = app::create_app(test_database.pool());

    let response = app
        .oneshot(json_request(
            "POST",
            "/todos",
            json!({
                "title": "Concluir tarefa",
                "description": null
            }),
        ))
        .await
        .unwrap();

    let body = response_json(response).await;
    let id = body["id"].as_str().unwrap();

    let app = app::create_app(test_database.pool());

    let response = app
        .oneshot(empty_request("PATCH", &format!("/todos/{id}/complete")))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;

    assert_eq!(body["completed"], true);
}

#[tokio::test]
async fn should_return_bad_request_for_invalid_todo_id() {
    let test_database = TestDatabase::create().await;
    let app = app::create_app(test_database.pool());

    let response = app
        .oneshot(empty_request("GET", "/todos/abc"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response_json(response).await;

    assert_eq!(
        body["message"],
        "O parâmetro 'todo_id' deve ser um UUID válido"
    );
}
