use actix_web::{web, App, HttpServer};
use crate::db::DataAccess;
use log::Level;

mod api_v1;
mod db;
mod util;

struct AppData {
    db: DataAccess,
}

fn config(cfg: &mut web::ServiceConfig) {
    api_v1::configure(cfg);
}

fn main() {
    simple_logger::init_with_level(Level::Debug).unwrap();

    HttpServer::new(|| {
        let db_config = db::Config::new("localhost", 5432, "incrementor", "incrementor", "inc");
        App::new()
            .data(AppData {
                db: DataAccess::new(&db_config).unwrap()
            })
            .configure(config)
    })
        .bind("127.0.0.1:8088")
        .unwrap()
        .run()
        .unwrap();
}
