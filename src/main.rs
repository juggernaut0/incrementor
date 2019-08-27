use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use crate::db::DataAccess;

mod db;

struct AppData {
    db: DataAccess,
}

fn hello(name: web::Path<String>, data: web::Data<AppData>) -> String {
    data.db.test_call();
    format!("Hello {}!", name)
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/hello/{name}", web::get().to(hello));
}

fn main() {
    HttpServer::new(|| {
        let db_config = db::Config::new("localhost", 5432, "postgres", "postgres", "");
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
