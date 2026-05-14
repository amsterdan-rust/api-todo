mod app;
mod shared;
mod todo;

#[tokio::main]
async fn main() {
    let app = app::create_app();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
