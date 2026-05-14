mod app;
mod shared;
mod todo;

#[tokio::main]
async fn main() {
    let app = app::create_app();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .expect("failed to bind TCP listener");

    axum::serve(listener, app)
        .await
        .expect("failed to start server");
}
