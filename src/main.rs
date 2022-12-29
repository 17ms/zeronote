use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use std::env;
use zeronote::{
    database::connection::{init_pool, run_migrations},
    errors::app_error::AppError,
    handlers::tasks::*,
    middlewares::{
        auth::{self, CognitoConfig},
        cors::cors,
        security_headers::security_headers,
    },
    utils::{log::init_logger, ssl_builder::create_builder},
};

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client_origin_url = env::var("CLIENT_ORIGIN_URL").expect("CLIENT_ORIGIN_URL must be set");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let cognito_cfg = CognitoConfig::default();

    let pool = init_pool(database_url);
    let mut conn = pool.get()?;
    let builder = create_builder()?;
    run_migrations(&mut conn);
    init_logger()?;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(cors(&client_origin_url))
            .wrap(security_headers())
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _| AppError::JsonPayLoad(err).into()),
            )
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(cognito_cfg.clone()))
            .service(
                web::scope("/api")
                    .service(create_new_task)
                    .service(get_all_tasks)
                    .service(delete_task)
                    .service(update_task)
                    .wrap(auth::Authorization),
            )
            .default_service(web::to(|| HttpResponse::NotFound()))
    })
    .bind_openssl("0.0.0.0:3000", builder)?
    .run()
    .await?;

    Ok(())
}
