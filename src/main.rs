mod api;
mod model;
mod schema;

use actix_web::{middleware::Logger, web::Data, App, HttpServer};

use api::file::{create_file, delete_file, get_file, get_files, update_file};

use crate::config::app::AppState;

mod config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data: AppState = AppState::new().await;
    let url = app_data.config.app.url.clone();
    let port = app_data.config.app.port.clone().parse::<u16>().unwrap();

    println!("Starting memora!");
    println!("Listening on http://{}:{}", url, port);

    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    HttpServer::new(move || {
        let logger = Logger::default();

        App::new()
            .app_data(Data::new(AppState {
                config: app_data.config.clone(),
                database: app_data.database.clone(),
            }))
            .wrap(logger)
            .service(get_file)
            .service(get_files)
            .service(create_file)
            .service(update_file)
            .service(delete_file)
    })
    .bind((url, port))?
    .run()
    .await
}
