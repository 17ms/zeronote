use crate::{
    database::connection::Pool, errors::app_error::AppError, models::task::*, services::tasks,
};
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use validator::Validate;

// Handlers for basic CRUD functionality regarding tasks

#[get("/all")]
pub async fn get_all_tasks(
    req: HttpRequest,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, AppError> {
    let headers = req.headers().clone();
    let tasks_vec = web::block(move || tasks::get_all(pool, headers))
        .await
        .map_err(AppError::WebBlocking)??;

    Ok(HttpResponse::Ok().json(tasks_vec))
}

#[post("/new")]
pub async fn create_new_task(
    req: HttpRequest,
    pool: web::Data<Pool>,
    task: web::Json<CreateTask>,
) -> Result<HttpResponse, AppError> {
    task.validate().map_err(AppError::Validator)?;
    let headers = req.headers().clone();
    let res = web::block(move || tasks::create(pool, task.into_inner(), headers))
        .await
        .map_err(AppError::WebBlocking)??;

    Ok(HttpResponse::Ok().json(res))
}

#[put("/update")]
pub async fn update_task(
    req: HttpRequest,
    pool: web::Data<Pool>,
    task: web::Json<UpdateTask>,
) -> Result<HttpResponse, AppError> {
    task.validate().map_err(AppError::Validator)?;
    let headers = req.headers().clone();
    let res = web::block(move || tasks::update(pool, task.into_inner(), headers))
        .await
        .map_err(AppError::WebBlocking)??;

    Ok(HttpResponse::Ok().json(res))
}

#[delete("/delete")]
pub async fn delete_task(
    req: HttpRequest,
    pool: web::Data<Pool>,
    task: web::Json<DeleteTask>,
) -> Result<HttpResponse, AppError> {
    task.validate().map_err(AppError::Validator)?;
    let headers = req.headers().clone();
    let res = web::block(move || tasks::delete(pool, task.into_inner().id, headers))
        .await
        .map_err(AppError::WebBlocking)??;

    Ok(HttpResponse::Ok().json(res))
}
