mod app;
mod shared;
mod todo;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "todo_api=debug,tower_http=debug".into()),
        )
        .init();

    tracing::info!("starting todo api");

    let app = app::create_app();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .expect("failed to bind TCP listener");

    tracing::info!("server listening on http://0.0.0.0:8000");

    axum::serve(listener, app)
        .await
        .expect("failed to start server");
}
