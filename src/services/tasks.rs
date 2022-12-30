use crate::{
    database::connection::Pool,
    errors::app_error::AppError,
    models::{
        schema::tasks::{self, dsl::*},
        task::*,
    },
};
use actix_http::header::HeaderMap;
use actix_web::web;
use chrono::Local;
use diesel::ExpressionMethods;
use diesel::{query_dsl::methods::FilterDsl, RunQueryDsl};
use jwt::{Header, RegisteredClaims, Token};
use std::str::FromStr;
use uuid::Uuid;

fn extract_sub(headers: HeaderMap) -> Result<String, AppError> {
    let bearer = headers
        .get("Authorization")
        .ok_or("Authorization header empty".to_owned())
        .map_err(AppError::AuthNotFound)?;
    let access_token = bearer.to_str().unwrap().split(" ").collect::<Vec<&str>>()[1];

    /* Skipping signature verification is in this case acceptable
    as the middleware does it before any endpoint handler is invoked */
    let unverified: Token<Header, RegisteredClaims, _> =
        Token::parse_unverified(access_token).map_err(AppError::JwtParse)?;
    let sub = unverified
        .claims()
        .subject
        .as_ref()
        .ok_or("No subject claim found in JWT".to_owned())
        .map_err(AppError::JwtMissingClaim)?
        .clone();

    Ok(sub)
}

pub fn get_all(pool: web::Data<Pool>, headers: HeaderMap) -> Result<Vec<Task>, AppError> {
    let mut conn = pool.get().map_err(AppError::DieselPool)?;
    let token_sub = extract_sub(headers)?;
    let tasks_vec = tasks
        .filter(tasks::owner_id.eq(token_sub))
        .get_results::<Task>(&mut conn)?;

    Ok(tasks_vec)
}

pub fn create(
    pool: web::Data<Pool>,
    task: CreateTask,
    headers: HeaderMap,
) -> Result<Task, AppError> {
    let mut conn = pool.get().map_err(AppError::DieselPool)?;
    let cur_time = Local::now().naive_local();
    let task_cond = TaskCondition::default();
    let token_sub = extract_sub(headers)?;

    let new_task = NewTask {
        title: &task.title,
        owner_id: &token_sub,
        body: &task.body,
        condition: task_cond,
        created_at: cur_time,
        updated_at: cur_time,
    };
    let res = diesel::insert_into(tasks::table)
        .values(new_task)
        .get_result(&mut conn)
        .map_err(AppError::DieselResult)?;

    Ok(res)
}

pub fn update(
    pool: web::Data<Pool>,
    task: UpdateTask,
    headers: HeaderMap,
) -> Result<Task, AppError> {
    let mut conn = pool.get().map_err(AppError::DieselPool)?;
    let token_sub = extract_sub(headers)?;

    let res = diesel::update(tasks::table)
        .filter(tasks::id.eq(Uuid::parse_str(task.id.as_str()).map_err(AppError::Uuid)?))
        .filter(tasks::owner_id.eq(token_sub))
        .set((
            tasks::title.eq(task.title),
            tasks::body.eq(task.body),
            tasks::condition.eq(TaskCondition::from_str(&task.condition)?),
            tasks::updated_at.eq(Local::now().naive_local()),
        ))
        .get_result(&mut conn)
        .map_err(AppError::DieselResult)?;

    Ok(res)
}

pub fn delete(
    pool: web::Data<Pool>,
    task_uuid_str: String,
    headers: HeaderMap,
) -> Result<usize, AppError> {
    let mut conn = pool.get().map_err(AppError::DieselPool)?;
    let token_sub = extract_sub(headers)?;

    let res = diesel::delete(
        tasks::table
            .filter(tasks::id.eq(Uuid::parse_str(task_uuid_str.as_str()).map_err(AppError::Uuid)?))
            .filter(tasks::owner_id.eq(token_sub)),
    )
    .execute(&mut conn)
    .map_err(AppError::DieselResult)?;

    Ok(res)
}
