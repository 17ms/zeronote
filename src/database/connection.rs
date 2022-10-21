use diesel::{
    r2d2::{self, ConnectionManager},
    PgConnection,
};

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn init_pool(database_url: String) -> Pool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create a connection pool");

    pool
}
