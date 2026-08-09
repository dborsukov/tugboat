use std::{fs, path::PathBuf, time::Duration};

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    Pool, Sqlite,
};
use tauri::Manager;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};
use tauri_specta::{collect_commands, Builder};

pub mod database;
pub mod error;
pub mod utils;

#[derive(Debug)]
pub struct AppState {
    pub data_dir: PathBuf,
    pub db_pool: Pool<Sqlite>,
    pub version: &'static str,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        database::cars::fetch_cars_count,
        database::cars::fetch_car_by_id,
        database::cars::fetch_cars_list,
        database::cars::fetch_car_details,
        database::cars::fetch_car_details_list,
        database::cars::create_car,
        database::cars::update_car,
        database::cars::delete_car,
        database::cars::fetch_car_brands,
        database::clients::fetch_clients_count,
        database::clients::fetch_clients_list,
        database::clients::fetch_client_details_dto,
        database::clients::create_client,
        database::clients::update_client,
        database::clients::delete_client,
        database::services::fetch_services_count,
        database::services::fetch_service_by_id,
        database::services::fetch_services_list,
        database::services::create_service,
        database::services::update_service,
        database::services::delete_service,
        database::settings::fetch_settings,
        database::settings::update_settings,
        database::work_orders::fetch_work_order_details_dto,
        database::work_orders::fetch_work_order_details_dto_by_car_id,
        database::work_orders::fetch_work_order_details_dto_by_client_id,
        database::work_orders::fetch_work_order_details_dto_list,
        database::work_orders::fetch_work_orders_count,
        database::work_orders::fetch_work_orders_list,
        database::work_orders::create_work_order,
        database::work_orders::update_work_order,
        database::work_orders::delete_work_order,
        database::work_orders::fetch_labor_item_by_id,
        database::work_orders::fetch_labor_items_list,
        database::work_orders::create_labor_item,
        database::work_orders::update_labor_item,
        database::work_orders::delete_labor_item,
        database::work_orders::fetch_part_item_by_id,
        database::work_orders::fetch_part_items_list,
        database::work_orders::create_part_item,
        database::work_orders::update_part_item,
        database::work_orders::delete_part_item,
        utils::get_app_version
    ]);

    // gen only in dev
    #[cfg(debug_assertions)]
    specta_builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/lib/bindings.ts",
        )
        .unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);

            let app_dir = app.path().app_data_dir().unwrap();
            fs::create_dir_all(&app_dir).unwrap();

            let db_path = app_dir.join("main.db");
            let options = SqliteConnectOptions::new()
                .filename(db_path)
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                .synchronous(SqliteSynchronous::Normal)
                .foreign_keys(true);
            let db_pool = tauri::async_runtime::block_on(async {
                let pool = SqlitePoolOptions::new()
                    .max_connections(5)
                    .acquire_timeout(Duration::from_secs(5))
                    .connect_with(options)
                    .await
                    .unwrap();
                sqlx::migrate!("./migrations").run(&pool).await.unwrap();
                pool
            });

            let log_level = if cfg!(debug_assertions) {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            };

            app.handle().plugin(
                tauri_plugin_log::Builder::new()
                    .level(log_level)
                    .target(Target::new(TargetKind::Folder {
                        path: app_dir.join("logs"),
                        file_name: None,
                    }))
                    .rotation_strategy(RotationStrategy::KeepSome(50))
                    .max_file_size(500_000 /* bytes */)
                    .build(),
            )?;

            app.manage(AppState {
                data_dir: app_dir,
                db_pool,
                version: env!("CARGO_PKG_VERSION"),
            });

            let appstate = app.state::<AppState>();
            print!("{:#?}", appstate);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
