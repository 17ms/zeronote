use super::connection::Pool;
use crate::{
    errors::AppError,
    schema::tasks::{self, dsl::*},
};
use actix_web::web;
use chrono::{Local, NaiveDateTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

// Insertables and ORM models for Diesel

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = tasks)]
pub struct NewTask<'a> {
    pub title: &'a str,
    pub body: &'a str,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Queryable, AsChangeset, Serialize, Deserialize)]
pub struct Task {
    pub id: uuid::Uuid, // Requires uuid-ossp extension
    pub title: String,
    pub body: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Task {
    pub fn get_all(pool: web::Data<Pool>) -> Result<Vec<Self>, AppError> {
        let mut conn = pool.get()?;
        let tasks_vec = tasks.load::<Task>(&mut conn)?;
        // TODO: use belonging_to(user) once user auth is in use

        Ok(tasks_vec)
    }

    pub fn create(pool: web::Data<Pool>, task: CreateTask) -> Result<Self, AppError> {
        let mut conn = pool.get()?;
        let cur_time = Local::now().naive_local();
        let new_task = NewTask {
            title: &task.title,
            body: &task.body,
            created_at: cur_time,
            updated_at: cur_time,
        };
        let res = diesel::insert_into(tasks::table)
            .values(new_task)
            .get_result(&mut conn)?;

        Ok(res)
    }

    pub fn update(pool: web::Data<Pool>, task: UpdateTask) -> Result<Self, AppError> {
        let mut conn = pool.get()?;
        let res = diesel::update(tasks::table)
            .filter(tasks::id.eq(Uuid::parse_str(task.id.as_str())?))
            .set((
                tasks::title.eq(task.title),
                tasks::body.eq(task.body),
                tasks::updated_at.eq(Local::now().naive_local()),
            ))
            .get_result(&mut conn)?;

        Ok(res)
    }

    pub fn delete(pool: web::Data<Pool>, task_uuid_str: String) -> Result<usize, AppError> {
        let mut conn = pool.get()?;
        let res = diesel::delete(
            tasks::table.filter(tasks::id.eq(Uuid::parse_str(task_uuid_str.as_str())?)),
        )
        .execute(&mut conn)?;

        Ok(res)
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateTask {
    #[validate(length(
        min = 1,
        max = 60,
        message = "Title must be between 1 and 60 characters long"
    ))]
    pub title: String,
    #[validate(length(min = 1, message = "Body must be at least 1 character long"))]
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateTask {
    #[validate(
        length(equal = 36, message = "UUID must be exactly 32 hex digits + 4 dashes"),
        custom = "validate_uuid_str"
    )]
    pub id: String,
    #[validate(length(
        min = 1,
        max = 60,
        message = "Title must be between 1 and 60 characters long"
    ))]
    pub title: String,
    #[validate(length(min = 1, message = "Body must be at least 1 character long"))]
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct DeleteTask {
    #[validate(
        length(equal = 36, message = "UUID must be exactly 32 hex digits + 4 dashes"),
        custom = "validate_uuid_str"
    )]
    pub id: String,
}

fn validate_uuid_str(uuid_str: &str) -> Result<(), ValidationError> {
    match Uuid::parse_str(uuid_str) {
        Ok(_) => Ok(()),
        Err(_) => Err(ValidationError::new("Invalid UUID")),
    }
}

// TODO: Unit tests with diesel's test_transaction
