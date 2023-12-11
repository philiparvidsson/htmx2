use axum::Router;
use htmx2::RouterExt as _;
use tower_http::services::ServeDir;

mod auth;
mod page;
mod pages;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest_service("/pub", ServeDir::new("pub"))
        .with_htmx_routes();

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
