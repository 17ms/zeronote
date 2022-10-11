use super::connection::Pool;
use crate::{
    handlers::tasks::CreateTask,
    schema::tasks::{self, dsl::*},
};
use actix_web::web;
use chrono::{Local, NaiveDateTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// Insertable and ORM model for Diesel

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
    pub fn get_all(pool: web::Data<Pool>) -> Result<Vec<Self>, diesel::result::Error> {
        let mut conn = pool.get().unwrap();
        let tasks_vec = tasks.load::<Task>(&mut conn)?;

        Ok(tasks_vec)
    }

    pub fn create(pool: web::Data<Pool>, task: CreateTask) -> Result<Self, diesel::result::Error> {
        let mut conn = pool.get().unwrap();
        let new_task = NewTask {
            title: &task.title,
            body: &task.body,
            created_at: Local::now().naive_local(),
            updated_at: Local::now().naive_local(),
        };
        let res = diesel::insert_into(tasks::table)
            .values(new_task)
            .get_result(&mut conn)?;

        Ok(res)
    }

    pub fn update(pool: web::Data<Pool>, task: Self) -> Result<Self, diesel::result::Error> {
        let mut conn = pool.get().unwrap();
        let res = diesel::update(tasks::table)
            .filter(tasks::id.eq(task.id))
            .set(task)
            .get_result(&mut conn)?;

        Ok(res)
    }

    pub fn delete(
        pool: web::Data<Pool>,
        task_uuid: uuid::Uuid,
    ) -> Result<usize, diesel::result::Error> {
        let mut conn = pool.get().unwrap();
        let res =
            diesel::delete(tasks::table.filter(tasks::id.eq(task_uuid))).execute(&mut conn)?;

        Ok(res)
    }
}
