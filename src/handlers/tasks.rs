use crate::database::{connection::Pool, models::Task};
use actix_web::{delete, get, post, put, web, Error, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTask {
    pub title: String,
    pub body: String,
}

#[get("/all")]
pub async fn get_all_tasks(pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let tasks_vec = web::block(move || Task::get_all(pool))
        .await
        .unwrap()
        .unwrap();

    Ok(HttpResponse::Ok().json(tasks_vec))
}

#[post("/new")]
pub async fn create_new_task(
    pool: web::Data<Pool>,
    task: web::Json<CreateTask>,
) -> Result<HttpResponse, Error> {
    let res = web::block(move || Task::create(pool, task.into_inner()))
        .await
        .unwrap()
        .unwrap();

    Ok(HttpResponse::Ok().json(res))
}

#[put("/update")]
pub async fn update_task(
    pool: web::Data<Pool>,
    task: web::Json<Task>,
) -> Result<HttpResponse, Error> {
    let res = web::block(move || Task::update(pool, task.into_inner()))
        .await
        .unwrap()
        .unwrap();

    Ok(HttpResponse::Ok().json(res))
}

#[delete("/delete")]
pub async fn delete_task(
    pool: web::Data<Pool>,
    task: web::Json<Task>,
) -> Result<HttpResponse, Error> {
    let res = web::block(move || Task::delete(pool, task.into_inner().id))
        .await
        .unwrap()
        .unwrap();

    Ok(HttpResponse::Ok().json(res))
}
