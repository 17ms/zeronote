use crate::{
    database::{
        connection::Pool,
        models::{CreateTask, DeleteTask, Task, UpdateTask},
    },
    errors::AppError,
};
use actix_web::{delete, get, post, put, web, HttpResponse};
use validator::Validate;

// Handlers for basic CRUD functionality regarding tasks

#[get("/all")]
pub async fn get_all_tasks(pool: web::Data<Pool>) -> Result<HttpResponse, AppError> {
    let tasks_vec = web::block(move || Task::get_all(pool))
        .await
        .map_err(AppError::WebBlocking)??;

    Ok(HttpResponse::Ok().json(tasks_vec))
}

#[post("/new")]
pub async fn create_new_task(
    pool: web::Data<Pool>,
    task: web::Json<CreateTask>,
) -> Result<HttpResponse, AppError> {
    task.validate().map_err(AppError::Validator)?;
    let res = web::block(move || Task::create(pool, task.into_inner()))
        .await
        .map_err(AppError::WebBlocking)??;

    Ok(HttpResponse::Ok().json(res))
}

#[put("/update")]
pub async fn update_task(
    pool: web::Data<Pool>,
    task: web::Json<UpdateTask>,
) -> Result<HttpResponse, AppError> {
    task.validate().map_err(AppError::Validator)?;
    let res = web::block(move || Task::update(pool, task.into_inner()))
        .await
        .map_err(AppError::WebBlocking)??;

    Ok(HttpResponse::Ok().json(res))
}

#[delete("/delete")]
pub async fn delete_task(
    pool: web::Data<Pool>,
    task: web::Json<DeleteTask>,
) -> Result<HttpResponse, AppError> {
    task.validate().map_err(AppError::Validator)?;
    let res = web::block(move || Task::delete(pool, task.into_inner().id))
        .await
        .map_err(AppError::WebBlocking)??;

    Ok(HttpResponse::Ok().json(res))
}
