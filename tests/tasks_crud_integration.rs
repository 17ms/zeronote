use actix_http::{Request, StatusCode};
use actix_web::{
    dev::{Service, ServiceResponse},
    test,
    web::{self, Bytes},
    App, Error,
};
use diesel::{sql_query, Connection, PgConnection, RunQueryDsl};
use dotenv::dotenv;
use serde_json::json;
use std::env;
use zeronote::{
    database::{
        connection::{init_pool, run_migrations, Pool},
        models::Task,
    },
    errors::{AppError, AppErrorResponse},
    handlers::tasks::*,
};

// Integration tests for JSON based CRUD requests (regarding tasks)
// Each test runs in a separate database (because of parallelism)

struct Context {
    pub db_name: String,
    pub psql_user: String,
    pub psql_pw: String,
}

impl Context {
    fn new(db_name: &str) -> Self {
        dotenv().ok();
        let psql_user =
            env::var("POSTGRES_USER").expect("POSTGRES_USER must be set for integration tests");
        let psql_pw = env::var("POSTGRES_PASSWORD")
            .expect("POSTGRES_PASSWORD must be set for integration tests");
        let database_url = format!(
            "postgresql://localhost/postgres?user={}&password={}",
            psql_user, psql_pw
        );
        let mut conn = PgConnection::establish(&database_url)
            .expect("Failed to connect to the database 'postgres'");

        let query = sql_query(format!("CREATE DATABASE {};", db_name));
        query
            .execute(&mut conn)
            .expect(format!("Couldn't create database {}", db_name).as_str());

        Self {
            db_name: db_name.to_string(),
            psql_user,
            psql_pw,
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        let database_url = format!(
            "postgresql://localhost/postgres?user={}&password={}",
            self.psql_user, self.psql_pw
        );
        let mut conn = PgConnection::establish(&database_url)
            .expect("Failed to connect to the database 'postgres'");

        let query = sql_query(format!("DROP DATABASE {};", self.db_name));
        query
            .execute(&mut conn)
            .expect(format!("Couldn't drop database {}", self.db_name).as_str());
    }
}

fn create_pool(ctx: &Context) -> Pool {
    let database_url = format!(
        "postgresql://localhost/{}?user={}&password={}",
        ctx.db_name, ctx.psql_user, ctx.psql_pw
    );
    // Pool is unnecessary for tests, but easier than completely changing Actix's web::Data type
    let pool = init_pool(database_url);
    let mut conn = pool.get().unwrap();

    let query = sql_query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";");
    println!("Created extension uuid-ossp");
    query
        .execute(&mut conn)
        .expect("Couldn't install postgres extension 'uuid-ossp'");
    run_migrations(&mut conn);

    pool
}

async fn get_create_task_res(
    app: &impl Service<Request, Response = ServiceResponse, Error = Error>,
    title: String,
) -> ServiceResponse {
    let req_body = json!({
        "title": title,
        "body": "Task body"
    });
    let req = test::TestRequest::post()
        .uri("/api/new")
        .set_json(req_body)
        .to_request();
    let res = test::call_service(&app, req).await;

    res
}

async fn get_update_task_res(
    app: &impl Service<Request, Response = ServiceResponse, Error = Error>,
    id: uuid::Uuid,
    new_title: String,
) -> ServiceResponse {
    let req_body = json!({
        "id": id,
        "title": new_title,
        "body": "New body"
    });
    let req = test::TestRequest::put()
        .uri("/api/update")
        .set_json(req_body)
        .to_request();
    let res = test::call_service(&app, req).await;

    res
}

async fn get_read_tasks_res(
    app: &impl Service<Request, Response = ServiceResponse, Error = Error>,
) -> ServiceResponse {
    let req = test::TestRequest::get().uri("/api/all").to_request();
    let res = test::call_service(&app, req).await;

    res
}

async fn get_delete_task_res(
    app: &impl Service<Request, Response = ServiceResponse, Error = Error>,
    id: uuid::Uuid,
) -> ServiceResponse {
    let req_body = json!({
        "id": id,
    });
    let req = test::TestRequest::delete()
        .uri("/api/delete")
        .set_json(req_body)
        .to_request();
    let res = test::call_service(&app, req).await;

    res
}

async fn get_invalid_json_res(
    app: &impl Service<Request, Response = ServiceResponse, Error = Error>,
) -> ServiceResponse {
    let req_body = json!({
        "title": "Task title",
    });
    let req = test::TestRequest::post()
        .uri("/api/new")
        .set_json(req_body)
        .to_request();
    let res = test::call_service(&app, req).await;

    res
}

#[actix_web::test]
async fn test_create_task_req() {
    let ctx = Context::new("create_task_test");
    let pool = create_pool(&ctx);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(web::scope("/api").service(create_new_task)),
    )
    .await;

    let res = get_create_task_res(&app, "Task title".to_string()).await;
    assert!(
        res.status().is_success(),
        "Received unsuccessful HTTP response"
    );
    let res_body: Task = test::read_body_json(res).await;
    assert_eq!(
        res_body.title,
        "Task title".to_string(),
        "Task in the response doesn't match the original"
    );
    assert_eq!(
        res_body.body,
        "Task body".to_string(),
        "Task in the response doesn't match the original"
    );
}

#[actix_web::test]
async fn test_update_task_req() {
    let ctx = Context::new("update_task_test");
    let pool = create_pool(&ctx);
    let app = test::init_service(
        App::new().app_data(web::Data::new(pool.clone())).service(
            web::scope("/api")
                .service(update_task)
                .service(create_new_task),
        ),
    )
    .await;

    let create_res = get_create_task_res(&app, "Old title".to_string()).await;
    let create_res_body: Task = test::read_body_json(create_res).await;

    let update_res = get_update_task_res(&app, create_res_body.id, "New title".to_string()).await;
    assert!(
        update_res.status().is_success(),
        "Received unsuccessful HTTP response"
    );
    let update_res_body: Task = test::read_body_json(update_res).await;
    assert_eq!(
        update_res_body.title,
        "New title".to_string(),
        "Task in the response doesn't match the original"
    );
    assert_eq!(
        update_res_body.body,
        "New body".to_string(),
        "Task in the response doesn't match the original"
    );
}

#[actix_web::test]
async fn test_read_tasks_req() {
    let ctx = Context::new("read_tasks_test");
    let pool = create_pool(&ctx);
    let app = test::init_service(
        App::new().app_data(web::Data::new(pool.clone())).service(
            web::scope("/api")
                .service(get_all_tasks)
                .service(create_new_task),
        ),
    )
    .await;

    let create_res_1 = get_create_task_res(&app, "Task title 1".to_string()).await;
    assert!(
        create_res_1.status().is_success(),
        "Received unsuccessful HTTP response"
    );

    let create_res_2 = get_create_task_res(&app, "Task title 2".to_string()).await;
    assert!(
        create_res_2.status().is_success(),
        "Received unsuccessful HTTP response"
    );

    let fetch_res = get_read_tasks_res(&app).await;
    assert!(
        fetch_res.status().is_success(),
        "Received unsuccessful HTTP response"
    );
    let mut fetch_res_body: Vec<Task> = test::read_body_json(fetch_res).await;
    assert_eq!(
        fetch_res_body.len(),
        2,
        "Response contains wrong amount of reuslts"
    );
    assert_eq!(
        fetch_res_body.pop().unwrap().title,
        "Task title 2".to_string(),
        "Task in the response doesn't match the original"
    );
    assert_eq!(
        fetch_res_body.pop().unwrap().title,
        "Task title 1".to_string(),
        "Task in the response doesn't match the original"
    );
}

#[actix_web::test]
async fn test_delete_task_req() {
    let ctx = Context::new("delete_task_test");
    let pool = create_pool(&ctx);
    let app = test::init_service(
        App::new().app_data(web::Data::new(pool.clone())).service(
            web::scope("/api")
                .service(delete_task)
                .service(create_new_task)
                .service(get_all_tasks),
        ),
    )
    .await;

    let create_res = get_create_task_res(&app, "Task title".to_string()).await;
    let create_res_body: Task = test::read_body_json(create_res).await;

    let delete_res = get_delete_task_res(&app, create_res_body.id).await;
    assert!(
        delete_res.status().is_success(),
        "Received unsuccessful HTTP response"
    );
    let delete_res_body_bytes = test::read_body(delete_res).await;
    assert_eq!(delete_res_body_bytes, Bytes::from_static(b"1"));

    let fetch_res = get_read_tasks_res(&app).await;
    assert!(
        fetch_res.status().is_success(),
        "Received unsuccessful HTTP response"
    );
    let fetch_res_body_bytes = test::read_body(fetch_res).await;
    assert_eq!(fetch_res_body_bytes, Bytes::from_static(b"[]"));
}

#[actix_web::test]
async fn test_invalid_json_body_req() {
    let ctx = Context::new("invalid_json_test");
    let pool = create_pool(&ctx);
    let app = test::init_service(
        App::new()
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _| AppError::json_default_err_handler(err).into()),
            )
            .app_data(web::Data::new(pool.clone()))
            .service(web::scope("/api").service(create_new_task)),
    )
    .await;

    let res = get_invalid_json_res(&app).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let res_body: AppErrorResponse = test::read_body_json(res).await;
    assert_eq!(res_body.code, "400");
}
