use std::sync::Arc;

use actix_web::{App, HttpServer, web};
use log::Level;

use crate::db::DataAccess;

mod api_v1;
mod db;
mod util;

#[derive(Clone)]
struct AppData {
    db: Arc<DataAccess>,
}

fn config(cfg: &mut web::ServiceConfig) {
    api_v1::configure(cfg);
}

fn main() {
    simple_logger::init_with_level(Level::Debug).unwrap();

    let db_config = db::Config::new("localhost", 5432, "incrementor", "incrementor", "inc");
    let data = AppData {
        db: Arc::new(DataAccess::new(&db_config).unwrap())
    };

    HttpServer::new(move || {
        App::new()
            .data(data.clone())
            .configure(config)
    })
        .bind("127.0.0.1:8088")
        .unwrap()
        .run()
        .unwrap();
}
