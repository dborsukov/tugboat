//! Test helpers shared by the module level tests.

use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};

/// An empty database with the real migrations applied.
pub(crate) async fn pool() -> SqlitePool {
    let options = SqliteConnectOptions::new().filename(":memory:");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("in-memory database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations apply");

    pool
}

/// A client with one car and one work order on it: the smallest history that a cascading delete would destroy.
pub(crate) async fn seed_client_with_order(pool: &SqlitePool) {
    for statement in [
        "INSERT INTO clients (id, name) VALUES (1, 'Тестовий клієнт')",
        "INSERT INTO car_brands (id, slug, name, name_key) VALUES (1, 'skoda', 'Skoda', 'skoda')",
        "INSERT INTO cars (id, client_id, brand_id, model, year) VALUES (1, 1, 1, 'Octavia', 2020)",
        "INSERT INTO work_orders (id, car_id, odometer_in, status, labor_cost_type, discount_type)
         VALUES ('A-1', 1, 100, 'working', 'hourly', 'none')",
    ] {
        sqlx::query(statement)
            .execute(pool)
            .await
            .expect("seed statement");
    }
}

/// The number of rows in `table`, to assert what survived.
pub(crate) async fn count(pool: &SqlitePool, table: &str) -> i64 {
    sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
        .fetch_one(pool)
        .await
        .expect("count query")
}
