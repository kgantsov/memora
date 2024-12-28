mod api;
mod client;
mod error;
mod model;
mod node;
mod schema;

use std::{env, fs};

use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};

use api::file::{create_file, delete_file, get_file, get_files, update_file};

use crate::client::Client;
use crate::config::app::AppState;

mod config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data: AppState = AppState::new().await;
    let url = app_data.config.app.url.clone();
    let port = app_data.config.app.port.clone().parse::<u16>().unwrap();

    log::info!("creating temporary upload directory");

    fs::create_dir_all("./tmp").unwrap();

    log::info!("configuring S3 client");
    let aws_region = RegionProviderChain::default_provider().or_else("us-east-1");
    let endpoint = env::var("AWS_ENDPOINT_URL").unwrap();
    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        .endpoint_url(endpoint)
        .region(aws_region)
        .load()
        .await;

    // create singleton S3 client
    let client = Client::new(&aws_config);

    log::info!("using AWS region: {}", aws_config.region().unwrap());

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
            .app_data(Data::new(client.clone()))
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
