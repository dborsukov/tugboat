use std::{fs, path::PathBuf, time::Duration};

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    Pool, Sqlite,
};
use tauri::Manager;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};
use tauri_specta::{collect_commands, Builder};

pub mod database;
pub mod documents;
pub mod error;
pub mod utils;

use crate::error::AppError;

#[derive(Debug)]
pub struct AppState {
    pub data_dir: PathBuf,
    pub db_pool: Pool<Sqlite>,
    pub version: &'static str,
}

pub fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![
        database::cars::fetch_cars_count,
        database::cars::fetch_car_by_id,
        database::cars::fetch_cars_list,
        database::cars::fetch_car_details,
        database::cars::fetch_car_details_list,
        database::cars::create_car,
        database::cars::update_car,
        database::cars::delete_car,
        database::cars::archive_car,
        database::cars::restore_car,
        database::cars::fetch_car_delete_impact,
        database::cars::fetch_car_brands,
        database::clients::fetch_clients_count,
        database::clients::fetch_clients_list,
        database::clients::fetch_client_details_dto,
        database::clients::create_client,
        database::clients::update_client,
        database::clients::delete_client,
        database::clients::archive_client,
        database::clients::restore_client,
        database::clients::fetch_client_delete_impact,
        database::services::fetch_services_count,
        database::services::fetch_service_by_id,
        database::services::fetch_services_list,
        database::services::create_service,
        database::services::update_service,
        database::services::delete_service,
        database::services::delete_services,
        database::services::archive_service,
        database::services::archive_services,
        database::services::restore_service,
        database::services::restore_services,
        database::workers::fetch_workers_count,
        database::workers::fetch_workers_list,
        database::workers::fetch_worker_by_id,
        database::workers::create_worker,
        database::workers::update_worker,
        database::workers::delete_worker,
        database::workers::delete_workers,
        database::settings::fetch_settings,
        database::settings::update_theme,
        database::settings::update_ui_scale,
        database::settings::update_default_markup,
        database::settings::update_default_vat_rate,
        database::settings::update_default_prices_include_vat,
        database::settings::update_rate_variants,
        database::settings::update_vin_lookup_templates,
        database::settings::update_license_plate_lookup_templates,
        database::settings::update_timezone,
        database::settings::update_default_phone_country_code,
        database::settings::update_company_info,
        database::work_orders::fetch_work_order_details_dto,
        database::work_orders::fetch_work_order_details_dto_by_car_id,
        database::work_orders::fetch_work_order_details_dto_by_client_id,
        database::work_orders::fetch_work_order_details_dto_list,
        database::work_orders::fetch_work_orders_count,
        database::work_orders::fetch_work_orders_list,
        database::work_orders::create_work_order,
        database::work_orders::create_warranty_work_order,
        database::work_orders::update_work_order,
        database::work_orders::delete_work_order,
        database::work_orders::archive_work_order,
        database::work_orders::restore_work_order,
        database::work_orders::fetch_work_order_delete_impact,
        database::work_orders::fetch_labor_item_by_id,
        database::work_orders::fetch_labor_items_list,
        database::work_orders::create_labor_item,
        database::work_orders::update_labor_item,
        database::work_orders::update_labor_items_discount,
        database::work_orders::reorder_labor_items,
        database::work_orders::delete_labor_item,
        database::work_orders::delete_labor_items,
        database::work_orders::fetch_part_item_by_id,
        database::work_orders::fetch_part_items_list,
        database::work_orders::create_part_item,
        database::work_orders::update_part_item,
        database::work_orders::update_part_items_status,
        database::work_orders::update_part_items_discount,
        database::work_orders::reorder_part_items,
        database::work_orders::delete_part_item,
        database::work_orders::delete_part_items,
        database::work_orders::preview_labor_item,
        database::work_orders::preview_part_item,
        database::work_orders::preview_work_order,
        documents::fetch_work_order_documents,
        documents::fetch_documents_dir_info,
        documents::document_target_path,
        documents::generate_document,
        documents::pick_documents_folder,
        documents::update_documents_dir,
        utils::get_app_version
    ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = specta_builder();

    // gen only in dev
    #[cfg(debug_assertions)]
    specta_builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/lib/bindings.ts",
        )
        .unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);

            // Safety net for the splash screen: the main window starts hidden
            // and the frontend reveals it once the app is painted. If the
            // frontend never signals readiness, force the
            // reveal so the app can't stay stuck behind the splash.
            {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_secs(15));
                    let handle_for_closure = handle.clone();
                    let _ = handle.run_on_main_thread(move || {
                        if let Some(splash) = handle_for_closure.get_webview_window("splashscreen")
                        {
                            let _ = splash.close();
                        }
                        if let Some(main) = handle_for_closure.get_webview_window("main") {
                            let _ = main.show();
                        }
                    });
                });
            }

            let app_dir = app.path().app_data_dir().unwrap();
            fs::create_dir_all(&app_dir).unwrap();

            let db_path = app_dir.join("main.db");
            let options = SqliteConnectOptions::new()
                .filename(db_path)
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                .synchronous(SqliteSynchronous::Normal)
                .foreign_keys(true)
                .busy_timeout(Duration::from_secs(5));
            let db_pool = tauri::async_runtime::block_on(async {
                let pool = SqlitePoolOptions::new()
                    .max_connections(5)
                    .acquire_timeout(Duration::from_secs(5))
                    .connect_with(options)
                    .await
                    .map_err(|e| {
                        AppError::Migration(format!("Не вдалося підключитися до бази даних: {e}"))
                    })?;
                sqlx::migrate!("./migrations")
                    .run(&pool)
                    .await
                    .map_err(|e| {
                        AppError::Migration(format!(
                            "Не вдалося застосувати міграції бази даних: {e}"
                        ))
                    })?;
                Ok::<_, AppError>(pool)
            })?;

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

            // Make sure the documents folder (configured or default) exists.
            {
                let appstate = app.state::<AppState>();
                let _ = tauri::async_runtime::block_on(documents::resolve_documents_dir(&appstate));
            }

            let appstate = app.state::<AppState>();
            print!("{:#?}", appstate);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
