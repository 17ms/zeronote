mod database;
mod handlers;
mod schema;

use actix_web::{web, App, HttpServer};
use database::connection::init_pool;
use dotenv::dotenv;
use handlers::tasks::{create_new_task, delete_task, get_all_tasks, update_task};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();
    let pool = init_pool();

    HttpServer::new(move || {
        App::new().app_data(web::Data::new(pool.clone())).service(
            web::scope("/api")
                .service(create_new_task)
                .service(get_all_tasks)
                .service(delete_task)
                .app_data(update_task),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
