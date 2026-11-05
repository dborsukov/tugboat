use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    Pool, Sqlite,
};
use tauri::{utils::config::Color, AppHandle, Manager, WebviewWindowBuilder};
use tauri_plugin_dialog::DialogExt;
use tauri_specta::{collect_commands, Builder};

pub mod database;
pub mod documents;
pub mod error;
pub mod hashing;
pub mod logging;
pub mod reports;
pub mod utils;
pub mod window_geometry;

use crate::{
    database::settings::structs::ThemeMode,
    error::{AppError, AppResult},
};

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
        database::expenses::fetch_expenses_count,
        database::expenses::fetch_expenses_list,
        database::expenses::fetch_expense_by_id,
        database::expenses::create_expense,
        database::expenses::update_expense,
        database::expenses::delete_expense,
        database::expenses::delete_expenses,
        database::expenses::archive_expense,
        database::expenses::archive_expenses,
        database::expenses::restore_expense,
        database::expenses::restore_expenses,
        database::expenses::fetch_expense_categories,
        database::expenses::create_expense_category,
        database::expenses::update_expense_category,
        database::expenses::archive_expense_category,
        database::expenses::restore_expense_category,
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
        database::settings::update_sidebar_metric,
        database::settings::update_sidebar_recent_items,
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
        database::work_orders::fetch_part_suppliers,
        database::work_orders::fetch_part_items_list,
        database::work_orders::create_part_item,
        database::work_orders::update_part_item,
        database::work_orders::update_part_items_status,
        database::work_orders::update_part_items_supplier,
        database::work_orders::update_part_items_discount,
        database::work_orders::reorder_part_items,
        database::work_orders::delete_part_item,
        database::work_orders::delete_part_items,
        database::work_orders::preview_labor_item,
        database::work_orders::preview_part_item,
        database::work_orders::preview_work_order,
        documents::fetch_work_order_documents,
        documents::download_work_order_document,
        documents::open_work_order_document,
        documents::generate_document,
        reports::generate_report,
        reports::fetch_reports,
        reports::download_report,
        reports::open_report_copy,
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
        database::vat::fetch_work_order_vat,
        database::vat::void_work_order_vat,
        database::vat::record_work_order_vat,
        database::vat::fetch_expense_vat,
        database::vat::void_expense_vat,
        database::vat::fetch_part_item_vat,
        database::vat::void_part_item_vat,
        database::vat::fetch_vat_periods,
        database::vat::close_vat_period,
        database::vat::reopen_vat_period,
        database::recent::record_recent_item,
        database::recent::fetch_recent_items,
        database::recent::clear_recent_items,
        database::analytics::fetch_expense_summary,
        utils::get_app_version,
        logging::open_logs_dir,
        logging::log_diagnostics,
        window_geometry::reveal_window
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
    logging::install_panic_hook();

    let started = Instant::now();
    let specta_builder = specta_builder();

    #[cfg(debug_assertions)]
    export_bindings(&specta_builder);

    tauri::Builder::default()
        // Must be registered first
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            log::info!("Запущено другий екземпляр, показую вже відкрите вікно");
            // Bring the running window forward instead of opening a second one.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);

            let app_dir = app.path().app_data_dir().map_err(|error| {
                eprintln!("Не вдалося визначити теку даних: {error}");
                error
            })?;
            fs::create_dir_all(&app_dir).map_err(|error| {
                eprintln!("Не вдалося створити теку {}: {error}", app_dir.display());
                error
            })?;

            let log_dir = logging::log_dir();
            logging::init(app.handle(), &log_dir)?;

            log::info!(
                "Запуск PopeyeCRM {} | build={} | os={} arch={} | дані={} | журнали={}",
                env!("CARGO_PKG_VERSION"),
                if cfg!(debug_assertions) {
                    "debug"
                } else {
                    "release"
                },
                std::env::consts::OS,
                std::env::consts::ARCH,
                app_dir.display(),
                log_dir.display(),
            );

            // Nothing slow may run on this thread. The splash screen window
            // already exists at this point, but it can only paint once setup returns.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = start(&handle, app_dir, started).await {
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
        // Window position is written on each move, this is a sanity check
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|handle, event| {
            if let tauri::RunEvent::Exit = event {
                window_geometry::save_geometry(handle);
                log::info!("Застосунок завершує роботу");
            }
        });
}

/// Runs slow startup code, then opens the main window.
async fn start(handle: &AppHandle, app_dir: PathBuf, started: Instant) -> AppResult<()> {
    open_splash_window(handle)?;

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

    let applied: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&db_pool)
        .await
        .unwrap_or(0);

    log::info!(
        "Базу даних готово за {:?}: застосовано {applied} {}",
        started.elapsed(),
        utils::plural(applied.max(0) as usize, "міграцію", "міграції", "міграцій")
    );

    database::cars::brands::sync_catalog(&db_pool).await?;

    let (ui_scale, theme) = match database::settings::fetch_settings_from_pool(&db_pool).await {
        Ok(settings) => (settings.ui_scale, settings.theme),
        Err(error) => {
            log::warn!("Не вдалося прочитати налаштування: {error}");
            (1.0, ThemeMode::System)
        }
    };

    handle.manage(AppState {
        data_dir: app_dir,
        db_pool,
        version: env!("CARGO_PKG_VERSION"),
    });

    let appstate = handle.state::<AppState>();

    open_main_window(handle, ui_scale, theme)?;

    if let Err(error) = documents::invoices_dir(&appstate) {
        log::warn!("Не вдалося підготувати теку рахунків: {error}");
    }

    log::info!("Вікно відкрито за {:?}", started.elapsed());

    Ok(())
}

/// Looks a window definition up in `tauri.conf.json` by its label.
fn window_config(handle: &AppHandle, label: &str) -> AppResult<tauri::utils::config::WindowConfig> {
    handle
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == label)
        .cloned()
        .ok_or_else(|| AppError::Generic(format!("У конфігурації немає вікна «{label}»")))
}

/// Creates the splash screen window on the platforms that use one.
#[cfg(not(target_os = "macos"))]
fn open_splash_window(handle: &AppHandle) -> AppResult<()> {
    let config = window_config(handle, "splashscreen")?;

    WebviewWindowBuilder::from_config(handle, &config)
        .and_then(|builder| builder.build())
        .map_err(|e| AppError::Generic(format!("Не вдалося створити вікно заставки: {e}")))?;

    Ok(())
}

/// macOS shows the main window straight away
#[cfg(target_os = "macos")]
fn open_splash_window(_handle: &AppHandle) -> AppResult<()> {
    Ok(())
}

/// Creates the main window arms the splash screen's safety net.
fn open_main_window(handle: &AppHandle, ui_scale: f64, theme: ThemeMode) -> AppResult<()> {
    let config = window_config(handle, "main")?;

    let builder = WebviewWindowBuilder::from_config(handle, &config)
        .map_err(|e| AppError::Generic(format!("Не вдалося створити головне вікно: {e}")))?
        .initialization_script(boot_script(ui_scale));

    #[cfg(target_os = "macos")]
    let builder = macos_window_chrome(builder, ui_scale);

    let window = builder
        .build()
        .map_err(|e| AppError::Generic(format!("Не вдалося створити головне вікно: {e}")))?;

    let os_theme = window.theme().unwrap_or(tauri::Theme::Light);
    if let Err(error) = window.set_background_color(Some(background_color(theme, os_theme))) {
        log::warn!("Не вдалося задати колір тла вікна: {error}");
    }

    // Remember this window's size and position from now on
    window_geometry::track_window(&window);

    // Safety net for the splash screen: the main window starts hidden and the
    // frontend reveals it once the app is painted. If the frontend never signals
    // readiness, force the reveal so the app can't stay stuck behind the splash.
    // The reveal also restores the saved window geometry, so it goes through the
    // same path the frontend uses.
    let handle = handle.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(15));
        let handle_for_closure = handle.clone();
        let _ = handle.run_on_main_thread(move || {
            if let Err(error) = window_geometry::reveal_main_window(&handle_for_closure) {
                log::warn!("Не вдалося показати головне вікно: {error}");
            }
        });
    });

    Ok(())
}

const BACKGROUND_LIGHT: Color = Color(0xfb, 0xfa, 0xf7, 0xff);
const BACKGROUND_DARK: Color = Color(0x19, 0x1d, 0x24, 0xff);

/// Resolves the window background for a theme; `System` follows the appearance
/// the window reports.
fn background_color(theme: ThemeMode, os_theme: tauri::Theme) -> Color {
    match theme {
        ThemeMode::Light => BACKGROUND_LIGHT,
        ThemeMode::Dark => BACKGROUND_DARK,
        ThemeMode::System => match os_theme {
            tauri::Theme::Dark => BACKGROUND_DARK,
            _ => BACKGROUND_LIGHT,
        },
    }
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
