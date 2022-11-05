use super::connection::Pool;
use crate::{
    errors::AppError,
    schema::tasks::{self, dsl::*},
};
use actix_web::{error::JsonPayloadError, web};
use chrono::{Local, NaiveDateTime};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use uuid::Uuid;
use validator::{Validate, ValidationError};

// Insertables and ORM models for Diesel

#[derive(Debug, DbEnum, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskCondition {
    Undone,
    Active,
    Done,
}

impl Default for TaskCondition {
    fn default() -> Self {
        TaskCondition::Undone
    }
}

impl FromStr for TaskCondition {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, AppError> {
        match s.trim().to_lowercase().as_str() {
            "undone" => Ok(Self::Undone),
            "active" => Ok(Self::Active),
            "done" => Ok(Self::Done),
            _ => Err(AppError::JsonPayLoad(JsonPayloadError::ContentType)),
        }
    }
}

impl Display for TaskCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TaskCondition::Undone => "Undone",
                TaskCondition::Active => "Active",
                TaskCondition::Done => "Done",
            }
        )
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = tasks)]
pub struct NewTask<'a> {
    pub title: &'a str,
    pub body: &'a str,
    pub condition: TaskCondition,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Queryable, AsChangeset, Serialize, Deserialize)]
pub struct Task {
    pub id: uuid::Uuid, // Requires uuid-ossp extension
    pub title: String,
    pub body: String,
    pub condition: TaskCondition,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Task {
    pub fn get_all(pool: web::Data<Pool>) -> Result<Vec<Self>, AppError> {
        let mut conn = pool.get().map_err(AppError::DieselPool)?;
        let tasks_vec = tasks
            .load::<Self>(&mut conn)
            .map_err(AppError::DieselResult)?;

        Ok(tasks_vec)
    }

    pub fn create(pool: web::Data<Pool>, task: CreateTask) -> Result<Self, AppError> {
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

    pub fn update(pool: web::Data<Pool>, task: UpdateTask) -> Result<Self, AppError> {
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
        let res = diesel::delete(tasks::table.filter(
            tasks::id.eq(Uuid::parse_str(task_uuid_str.as_str()).map_err(AppError::Uuid)?),
        ))
        .execute(&mut conn)
        .map_err(AppError::DieselResult)?;

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
    #[validate(custom = "validate_task_cond_str")]
    pub condition: String,
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

fn validate_task_cond_str(cond_str: &str) -> Result<(), ValidationError> {
    match TaskCondition::from_str(cond_str) {
        Ok(_) => Ok(()),
        Err(_) => Err(ValidationError::new("Invalid task condition")),
    }
}
