mod common;

use actix_http::StatusCode;
use actix_web::{
    test,
    web::{self, Bytes},
    App,
};
use common::{
    create_pool, delete_endpoint_res, fetch_jwt, get_endpoint_res, post_endpoint_res,
    put_endpoint_res, Context,
};
use serde_json::json;
use zeronote::{
    errors::app_error::{AppError, AppErrorResponse},
    handlers::tasks::*,
    middlewares::auth::{self, CognitoConfig},
    models::task::{Task, TaskCondition},
};

// Integration tests for authentication/authorization with JWTs & querying the DB according to CRUD endpoints
// Each test runs synchronously, creates individual DBs & uses unique JWT

#[actix_web::test]
async fn test_create_task_req() {
    let ctx = Context::new("create_task_test");
    let pool = create_pool(&ctx);
    let bearer = fetch_jwt();
    let config = CognitoConfig::default();
    let app = test::init_service(
        App::new()
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _| AppError::JsonPayLoad(err).into()),
            )
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .service(
                web::scope("/api")
                    .service(create_new_task)
                    .wrap(auth::Authorization),
            ),
    )
    .await;

    let res = post_endpoint_res(
        &app,
        json!({"title":"Task title", "body":"Task body"}),
        &bearer,
        "/api/new",
    )
    .await;
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
    let bearer = fetch_jwt();
    let config = CognitoConfig::default();
    let app = test::init_service(
        App::new()
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _| AppError::JsonPayLoad(err).into()),
            )
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .service(
                web::scope("/api")
                    .service(update_task)
                    .service(create_new_task)
                    .wrap(auth::Authorization),
            ),
    )
    .await;

    let create_res = post_endpoint_res(
        &app,
        json!({"title":"Old title", "body":"Task body"}),
        &bearer,
        "/api/new",
    )
    .await;
    let create_res_body: Task = test::read_body_json(create_res).await;

    let update_res = put_endpoint_res(&app, json!({"id": create_res_body.id, "title": "New title", "body": "New body", "condition": "active"}), &bearer, "/api/update").await;
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
    assert_eq!(
        update_res_body.condition,
        TaskCondition::Active,
        "Task in the response doesn't match the original"
    );
    assert_ne!(
        create_res_body.created_at, update_res_body.updated_at,
        "Field 'updated_at' not updated properly"
    );
}

#[actix_web::test]
async fn test_read_tasks_req() {
    let ctx = Context::new("read_tasks_test");
    let pool = create_pool(&ctx);
    let bearer = fetch_jwt();
    let config = CognitoConfig::default();
    let app = test::init_service(
        App::new()
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _| AppError::JsonPayLoad(err).into()),
            )
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .service(
                web::scope("/api")
                    .service(get_all_tasks)
                    .service(create_new_task)
                    .wrap(auth::Authorization),
            ),
    )
    .await;

    let create_res_1 = post_endpoint_res(
        &app,
        json!({"title": "Task title 1", "body": "Task body"}),
        &bearer,
        "/api/new",
    )
    .await;
    assert!(
        create_res_1.status().is_success(),
        "Received unsuccessful HTTP response"
    );

    let create_res_2 = post_endpoint_res(
        &app,
        json!({"title": "Task title 2", "body": "Task body"}),
        &bearer,
        "/api/new",
    )
    .await;
    assert!(
        create_res_2.status().is_success(),
        "Received unsuccessful HTTP response"
    );

    let fetch_res = get_endpoint_res(&app, &bearer, "/api/all").await;
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
    let bearer = fetch_jwt();
    let config = CognitoConfig::default();
    let app = test::init_service(
        App::new()
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _| AppError::JsonPayLoad(err).into()),
            )
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .service(
                web::scope("/api")
                    .service(delete_task)
                    .service(create_new_task)
                    .service(get_all_tasks)
                    .wrap(auth::Authorization),
            ),
    )
    .await;

    //let create_res = get_create_task_res(&app, "Task title", &bearer).await;
    let create_res = post_endpoint_res(
        &app,
        json!({"title": "Task title", "body": "Task body"}),
        &bearer,
        "/api/new",
    )
    .await;
    let create_res_body: Task = test::read_body_json(create_res).await;

    //let delete_res = get_delete_task_res(&app, create_res_body.id, &bearer).await;
    let delete_res = delete_endpoint_res(
        &app,
        json!({"id": create_res_body.id}),
        &bearer,
        "/api/delete",
    )
    .await;
    assert!(
        delete_res.status().is_success(),
        "Received unsuccessful HTTP response"
    );
    let delete_res_body_bytes = test::read_body(delete_res).await;
    assert_eq!(delete_res_body_bytes, Bytes::from_static(b"1"));

    let fetch_res = get_endpoint_res(&app, &bearer, "/api/all").await;
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
    let bearer = fetch_jwt();
    let config = CognitoConfig::default();
    let app = test::init_service(
        App::new()
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _| AppError::JsonPayLoad(err).into()),
            )
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .service(
                web::scope("/api")
                    .service(create_new_task)
                    .wrap(auth::Authorization),
            ),
    )
    .await;

    //let res = get_invalid_json_res(&app, &bearer).await;
    let res = post_endpoint_res(&app, json!({"title": "Task title"}), &bearer, "/api/new").await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let res_body: AppErrorResponse = test::read_body_json(res).await;
    assert_eq!(res_body.code, "400");
}

#[actix_web::test]
#[should_panic]
async fn test_missing_jwt_req() {
    let ctx = Context::new("missing_jwt_test");
    let pool = create_pool(&ctx);
    let config = CognitoConfig::default();

    let app = test::init_service(
        App::new()
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _| AppError::JsonPayLoad(err).into()),
            )
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .service(
                web::scope("/api")
                    .service(create_new_task)
                    .wrap(auth::Authorization),
            ),
    )
    .await;

    //let res = get_missing_jwt_res(&app).await;
    let res = post_endpoint_res(
        &app,
        json!({"title": "Task title", "body": "Task body"}),
        "",
        "/api/new",
    )
    .await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let res_body: AppErrorResponse = test::read_body_json(res).await;
    assert_eq!(res_body.code, "401");
}
