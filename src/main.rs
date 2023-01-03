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

fn parse_env() -> (String, String) {
    dotenv().ok();

    let cors_url = env::var("CLIENT_ORIGIN_URL").expect("DATABASE_URL must be set");

    let db_user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
    let db_pass = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
    let db_host = env::var("POSTGRES_HOST").expect("POSTGRES_HOST must be set");
    let db_port = env::var("POSTGRES_PORT").expect("POSTGRES_PORT must be set");
    let db_name = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set");

    (
        cors_url,
        format!(
            "postgres://{}:{}@{}:{}/{}",
            db_user, db_pass, db_host, db_port, db_name
        ),
    )
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cors_url, db_url) = parse_env();
    let cognito_cfg = CognitoConfig::default();

    let pool = init_pool(db_url);
    let mut conn = pool.get()?;
    let builder = create_builder()?;
    run_migrations(&mut conn);
    init_logger()?;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(cors(&cors_url))
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
    .bind_openssl("0.0.0.0:443", builder)?
    .run()
    .await?;

    Ok(())
}
