use std::sync::Arc;

mod app;
mod config;

fn main() {
    let app = Arc::new(app::App::new());
    println!("Hello, world!");
}
