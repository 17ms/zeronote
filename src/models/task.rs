use crate::{errors::app_error::AppError, models::schema::tasks};
use actix_web::error::JsonPayloadError;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_derive_enum::*;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, DbEnum)]
#[DieselTypePath = "crate::models::schema::sql_types::TaskCondition"]
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
    pub owner_id: &'a str,
    pub title: &'a str,
    pub body: &'a str,
    pub condition: TaskCondition,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Queryable, AsChangeset, Serialize, Deserialize)]
pub struct Task {
    pub id: uuid::Uuid, // Requires uuid-ossp extension
    pub owner_id: String,
    pub title: String,
    pub body: String,
    pub condition: TaskCondition,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_validation() {
        let invalid_uuid = "550e840-e29b-41d4-a716-44665540000";
        assert!(validate_uuid_str(invalid_uuid).is_err());
    }

    #[test]
    fn test_task_cond_validation() {
        let valid_task_cond = "active";
        let invalid_task_cond = "down";
        assert!(validate_task_cond_str(valid_task_cond).is_ok());
        assert!(validate_task_cond_str(invalid_task_cond).is_err());
    }
}
