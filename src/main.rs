use actix_web::{web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use std::env;
use zeronote::{
    database::connection::{init_pool, run_migrations},
    errors::AppError,
    handlers::tasks::*,
};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = init_pool(database_url);
    let mut conn = pool.get().unwrap();
    run_migrations(&mut conn);

    // TODO: Logging

    HttpServer::new(move || {
        App::new()
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _| AppError::json_default_err_handler(err).into()),
            )
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
