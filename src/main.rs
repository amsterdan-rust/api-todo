mod app;
mod shared;
mod todo;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "todo_api=debug,tower_http=info".into()),
        )
        .init();

    tracing::info!("starting todo api");

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

    let db_pool = shared::db::create_pool(&database_url).await;

    let app = app::create_app(db_pool);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());

    let address = format!("0.0.0.0:{port}");

    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("failed to bind TCP listener");

    tracing::info!("server listening on http://{address}");

    axum::serve(listener, app)
        .await
        .expect("failed to start server");
}
