use std::{fs, path::PathBuf, time::Duration};

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    Pool, Sqlite,
};
use tauri::{AppHandle, Manager, WebviewWindowBuilder};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};
use tauri_specta::{collect_commands, Builder};

pub mod database;
pub mod documents;
pub mod error;
pub mod utils;

use crate::error::{AppError, AppResult};

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
        database::cars::brands::fetch_car_brands,
        database::cars::brands::fetch_custom_car_brands,
        database::cars::brands::create_car_brand,
        database::cars::brands::update_car_brand,
        database::cars::brands::archive_car_brand,
        database::cars::brands::restore_car_brand,
        database::cars::models::fetch_car_models,
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
        database::settings::update_dashboard_layout,
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
        database::analytics::fetch_order_months,
        database::analytics::fetch_outstanding_balance,
        database::analytics::fetch_in_progress_balance,
        database::analytics::fetch_revenue_summary,
        database::analytics::fetch_vat_due,
        database::analytics::fetch_top_services,
        database::analytics::fetch_client_mix,
        database::analytics::fetch_client_mix_list,
        database::analytics::fetch_gross_margin,
        database::analytics::fetch_margin_services,
        utils::get_app_version
    ])
}

#[cfg(debug_assertions)]
fn export_bindings(specta_builder: &tauri_specta::Builder<tauri::Wry>) {
    let target = std::path::Path::new("../src/lib/bindings.ts");
    let staging = target.with_extension("ts.staging");

    if let Err(error) = specta_builder.export(specta_typescript::Typescript::default(), &staging) {
        // Print since log plugin is not initialized yet
        eprintln!("Не вдалося згенерувати bindings: {error}");
        return;
    }

    // The file is swapped in rather than written in place. `export` truncates first
    // which caused weird issues with Vite HMR watcher reading incomplete file in dev.
    if let Err(error) = fs::rename(&staging, target) {
        eprintln!("Не вдалося замінити src/lib/bindings.ts: {error}");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = specta_builder();

    #[cfg(debug_assertions)]
    export_bindings(&specta_builder);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);

            let app_dir = app.path().app_data_dir().unwrap();
            fs::create_dir_all(&app_dir).unwrap();

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

            // Nothing slow may run on this thread. The splash screen window
            // already exists at this point, but it can only paint once setup returns.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = start(&handle, app_dir).await {
                    log::error!("Не вдалося запустити застосунок: {error}");
                    handle
                        .dialog()
                        .message(error.to_string())
                        .title("Папай CRM — помилка запуску")
                        .blocking_show();
                    handle.exit(1);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Runs slow startup code while splashscreen is shown, then opens the main window.
async fn start(handle: &AppHandle, app_dir: PathBuf) -> AppResult<()> {
    let db_path = app_dir.join("main.db");
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));

    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect_with(options)
        .await
        .map_err(|e| AppError::Migration(format!("Не вдалося підключитися до бази даних: {e}")))?;

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .map_err(|e| {
            AppError::Migration(format!("Не вдалося застосувати міграції бази даних: {e}"))
        })?;

    database::cars::brands::sync_catalog(&db_pool).await?;

    let ui_scale = match database::settings::fetch_settings_from_pool(&db_pool).await {
        Ok(settings) => settings.ui_scale,
        Err(error) => {
            log::warn!("Не вдалося прочитати масштаб інтерфейсу: {error}");
            1.0
        }
    };

    handle.manage(AppState {
        data_dir: app_dir,
        db_pool,
        version: env!("CARGO_PKG_VERSION"),
    });

    let appstate = handle.state::<AppState>();
    print!("{:#?}", appstate);

    open_main_window(handle, ui_scale)?;

    if let Err(error) = documents::resolve_documents_dir(&appstate).await {
        log::warn!("Не вдалося підготувати теку документів: {error}");
    }

    Ok(())
}

/// Creates the main window arms the splash screen's safety net.
fn open_main_window(handle: &AppHandle, ui_scale: f64) -> AppResult<()> {
    let config = handle
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "main")
        .cloned()
        .ok_or_else(|| AppError::Generic("У конфігурації немає головного вікна".to_string()))?;

    let builder = WebviewWindowBuilder::from_config(handle, &config)
        .map_err(|e| AppError::Generic(format!("Не вдалося створити головне вікно: {e}")))?
        .initialization_script(boot_script(ui_scale));

    #[cfg(target_os = "macos")]
    let builder = macos_window_chrome(builder, ui_scale);

    builder
        .build()
        .map_err(|e| AppError::Generic(format!("Не вдалося створити головне вікно: {e}")))?;

    // Safety net for the splash screen: the main window starts hidden and the
    // frontend reveals it once the app is painted. If the frontend never signals
    // readiness, force the reveal so the app can't stay stuck behind the splash.
    let handle = handle.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(15));
        let handle_for_closure = handle.clone();
        let _ = handle.run_on_main_thread(move || {
            if let Some(splash) = handle_for_closure.get_webview_window("splashscreen") {
                let _ = splash.close();
            }
            if let Some(main) = handle_for_closure.get_webview_window("main") {
                let _ = main.show();
            }
        });
    });

    Ok(())
}

/// Height of the app's own title bar, mirroring `--titlebar-height` in `src/app.css` (9 * 0.25rem).
#[cfg(target_os = "macos")]
const TITLEBAR_HEIGHT_PX: f64 = 36.0;

/// AppKit lays the 16pt-tall traffic lights out inside the title bar view and
/// ends up placing their centre this many points below the `y` that wry is
/// handed, so centring them in the app title bar means asking for `title bar height / 2 - this`.
#[cfg(target_os = "macos")]
const TRAFFIC_LIGHT_CENTRE_OFFSET_PT: f64 = 2.0;

/// Leading inset of the close button from the window's left edge.
#[cfg(target_os = "macos")]
const TRAFFIC_LIGHT_INSET_X_PT: f64 = 12.0;

/// Injects the values the frontend needs before its first paint.
/// The window chrome differs per platform (native macOS traffic lights vs. the
/// app's own buttons — see `src/lib/platform.ts` and `title-bar.svelte`), and
/// the title bar has to reserve room for the lights using the very same UI
/// scale they were positioned with, so both are handed over here instead of
/// being guessed from the user agent or waited for from the settings query.
fn boot_script(ui_scale: f64) -> String {
    format!(
        "window.__POPEYE_BOOT__ = {{ platform: \"{}\", uiScale: {} }};",
        std::env::consts::OS,
        ui_scale
    )
}

/// Gives the main window the macOS shape: the *native* traffic lights inside the
/// app's own title bar.
///
/// FIXME: the position can only be set here, while the window is created, so
/// changing the UI scale at runtime leaves the lights where they started until the app is restarted.
#[cfg(target_os = "macos")]
fn macos_window_chrome(
    builder: WebviewWindowBuilder<'_, tauri::Wry, AppHandle>,
    ui_scale: f64,
) -> WebviewWindowBuilder<'_, tauri::Wry, AppHandle> {
    let title_bar_height = TITLEBAR_HEIGHT_PX * ui_scale;
    let traffic_light_y = (title_bar_height / 2.0 - TRAFFIC_LIGHT_CENTRE_OFFSET_PT).max(0.0);

    builder
        .decorations(true)
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .hidden_title(true)
        .traffic_light_position(tauri::LogicalPosition::new(
            TRAFFIC_LIGHT_INSET_X_PT,
            traffic_light_y,
        ))
}
