use crate::{
    database::connection::Pool,
    errors::app_error::AppError,
    models::{
        schema::tasks::{self, dsl::*},
        task::*,
    },
};
use actix_web::web;
use chrono::Local;
use diesel::ExpressionMethods;
use diesel::{query_dsl::methods::FilterDsl, RunQueryDsl};
use std::str::FromStr;
use uuid::Uuid;

pub fn get_all(pool: web::Data<Pool>) -> Result<Vec<Task>, AppError> {
    let mut conn = pool.get().map_err(AppError::DieselPool)?;
    let tasks_vec = tasks
        .load::<Task>(&mut conn)
        .map_err(AppError::DieselResult)?;

    Ok(tasks_vec)
}

pub fn create(pool: web::Data<Pool>, task: CreateTask) -> Result<Task, AppError> {
    let mut conn = pool.get().map_err(AppError::DieselPool)?;
    let cur_time = Local::now().naive_local();
    let task_cond = TaskCondition::default();
    let new_task = NewTask {
        title: &task.title,
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

pub fn update(pool: web::Data<Pool>, task: UpdateTask) -> Result<Task, AppError> {
    let mut conn = pool.get().map_err(AppError::DieselPool)?;
    let res = diesel::update(tasks::table)
        .filter(tasks::id.eq(Uuid::parse_str(task.id.as_str()).map_err(AppError::Uuid)?))
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

pub fn delete(pool: web::Data<Pool>, task_uuid_str: String) -> Result<usize, AppError> {
    let mut conn = pool.get().map_err(AppError::DieselPool)?;
    let res = diesel::delete(
        tasks::table
            .filter(tasks::id.eq(Uuid::parse_str(task_uuid_str.as_str()).map_err(AppError::Uuid)?)),
    )
    .execute(&mut conn)
    .map_err(AppError::DieselResult)?;

    Ok(res)
}
