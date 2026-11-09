pub mod analytics;
pub mod cars;
pub mod clients;
pub mod expenses;
pub mod recent;
pub mod services;
pub mod settings;
pub mod utils;
pub mod vat;
pub mod work_orders;
pub mod workers;

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod tests {
    use sqlx::{
        sqlite::{SqliteConnectOptions, SqlitePoolOptions},
        SqlitePool,
    };

    #[test]
    fn init_migration_guards_ranges_and_signs() {
        tauri::async_runtime::block_on(async {
            let options = SqliteConnectOptions::new()
                .filename(":memory:")
                .foreign_keys(false);
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .expect("in-memory database");

            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("migrations apply");

            assert!(
                rejects(
                    &pool,
                    "INSERT INTO expenses (date, amount, description)
                 VALUES ('2026-01-01', '-5.00', 'x')"
                )
                .await
            );
            assert!(
                accepts(
                    &pool,
                    "INSERT INTO expenses (date, amount, description)
                 VALUES ('2026-01-01', '5.00', 'x')"
                )
                .await
            );

            assert!(
                rejects(
                    &pool,
                    "INSERT INTO cars (client_id, brand_id, model, year)
                 VALUES (1, 1, 'X', 1800)"
                )
                .await
            );
            assert!(
                accepts(
                    &pool,
                    "INSERT INTO cars (client_id, brand_id, model, year)
                 VALUES (1, 1, 'X', 2020)"
                )
                .await
            );

            assert!(rejects(
                &pool,
                "INSERT INTO work_orders (id, car_id, odometer_in, status, labor_cost_type, discount_type)
                 VALUES ('A-1', 1, -5, 'draft', 'hourly', 'none')"
            )
            .await);

            assert!(
                rejects(
                    &pool,
                    "INSERT INTO services (name, name_key, price_type, fixed_price)
                 VALUES ('X', 'x', 'fixed', '-1')"
                )
                .await
            );

            assert!(
                rejects(
                    &pool,
                    "INSERT INTO services (name, name_key, price_type, fixed_price)
                 VALUES ('X', 'x', 'banana', '1')"
                )
                .await
            );
            // Unique across archived rows too; the keys are lower-cased by Rust.
            assert!(
                accepts(
                    &pool,
                    "INSERT INTO services (name, name_key, price_type)
                 VALUES ('Заміна', 'заміна', 'fixed')"
                )
                .await
            );
            assert!(
                rejects(
                    &pool,
                    "INSERT INTO services (name, name_key, price_type)
                 VALUES ('Заміна інша', 'заміна', 'fixed')"
                )
                .await
            );

            assert!(rejects(
                &pool,
                "INSERT INTO work_orders (id, car_id, odometer_in, status, labor_cost_type, discount_type, vat_scheme)
                 VALUES ('A-2', 1, 0, 'draft', 'hourly', 'none', 'whatever')"
            )
            .await);
        });
    }

    async fn rejects(pool: &SqlitePool, sql: &str) -> bool {
        sqlx::query(sql).execute(pool).await.is_err()
    }

    async fn accepts(pool: &SqlitePool, sql: &str) -> bool {
        sqlx::query(sql).execute(pool).await.is_ok()
    }
}
