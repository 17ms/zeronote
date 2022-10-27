pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "task_condition"))]
    pub struct TaskCondition;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TaskCondition;

    tasks (id) {
        id -> Uuid,
        title -> Varchar,
        body -> Text,
        condition -> TaskCondition,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
