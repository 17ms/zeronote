mod database;
mod errors;
mod handlers;
mod schema;

use actix_web::{web, App, HttpResponse, HttpServer};
use database::connection::init_pool;
use dotenv::dotenv;
use handlers::tasks::{create_new_task, delete_task, get_all_tasks, update_task};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();
    // TODO: Logging

    let pool = init_pool();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api")
                    .service(create_new_task)
                    .service(get_all_tasks)
                    .service(delete_task)
                    .service(update_task),
            )
            .default_service(web::to(|| HttpResponse::NotFound()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
