use std::{fs, path::PathBuf};

use sqlx::{Pool, Sqlite};
use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

pub mod database;
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
        database::create_client,
        database::delete_client,
        database::fetch_clients,
        database::update_client,
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
            let options = sqlx::sqlite::SqliteConnectOptions::new()
                .filename(db_path)
                .create_if_missing(true);
            let db_pool = tauri::async_runtime::block_on(async {
                let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
                sqlx::migrate!("./migrations").run(&pool).await.unwrap();
                pool
            });

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
