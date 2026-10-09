//! Remembers the main window's size and position between launches.
//!
//! This is deliberately not `tauri-plugin-window-state`. That plugin restores
//! geometry with `WebviewWindow::set_size` while the main window is still
//! hidden, and on Windows tao compensates for the invisible borders of an
//! undecorated window with a shadow by measuring them off the window it is about
//! to resize:
//!
//! ```text
//! // tao, platform_impl/windows/window.rs, set_inner_size
//! if window_flags.undecorated_with_shadows() {
//!     desired_height += (window_rect.bottom - window_rect.top)
//!                      - (client_rect.bottom - client_rect.top);
//! }
//! ```
//!
//! A hidden window does not report a usable client rect, so that measurement is
//! wrong, it came out 30px too large here, and because the plugin saves the
//! client size and re-applies it through the same path, the window grew by 30px on every single launch.
//!
//! Two things this custom logic does to avoid that edge case:
//!
//! 1. Persist the outer rect: `outer_position`/`outer_size` are reported
//!    consistently on Windows, Linux and macOS, and are the geometry a user
//!    actually recognises. A window that was maximised records its restored
//!    geometry on every platform, so un-maximising lands where it should.
//! 2. Apply it once the window is visible: [`reveal_main_window`] is what the
//!    frontend's splash hand-off calls, so the window is shown before any geometry
//!    is touched and tao's frame measurement is taken from a real window.
//!
//! Rather than trusting a hand-computed frame offset on three platforms, the
//! restore measures the window's own frame and then reads the result back,
//! correcting itself until the outer rect matches what was saved. If the platform
//! clamps or ignores a request, the next attempt is computed from what the window
//! actually became, so it converges instead of accumulating error.
//!
//! The splash screen and the About window are not tracked: their geometry comes from `tauri.conf.json`

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tauri::{
    command, AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewWindow, WindowEvent,
};

use crate::{error::AppResult, AppState};

const MAIN_WINDOW: &str = "main";
const FILE_NAME: &str = "window-state.json";

/// Geometry file write debounce
const SAVE_DEBOUNCE: Duration = Duration::from_millis(400);

/// Bounds guarding the saved file
const MIN_DISPLAY_EDGE: u32 = 320;
const MAX_WINDOW_EDGE: u32 = 32_768;

/// How many times the restore may re-apply the geometry while settling.
/// The first attempt is usually enough the extra passes exist for window
/// managers that ignore or clamp the first one.
const MAX_CORRECTION_ATTEMPTS: usize = 3;

/// How long to let the window settle before reading the geometry back.
const CORRECTION_DELAY: Duration = Duration::from_millis(150);

/// Whether the main window has already been revealed and its geometry restored.
/// Claimed by whichever caller gets there first the frontend hand-off or the
/// watchdog so the geometry is applied exactly once per launch.
#[derive(Default)]
struct Revealed(AtomicBool);

/// The last geometry seen while the window was not maximised, held in memory.
/// Maximising is not atomic. The window is moved and resized to the monitor size
/// before `is_maximized()` starts returning true, so those events carry the
/// maximised rect. Remembering the last normal rect is what makes un-maximising
/// land back where it was, instead of on the monitor edge at monitor size.
#[derive(Default)]
struct LastNormalGeometry(Mutex<Option<SavedGeometry>>);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SavedGeometry {
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub maximized: bool,
}

impl SavedGeometry {
    fn is_plausible(&self) -> bool {
        let size_is_sane = (MIN_DISPLAY_EDGE..=MAX_WINDOW_EDGE).contains(&self.width)
            && (MIN_DISPLAY_EDGE..=MAX_WINDOW_EDGE).contains(&self.height);
        let position_is_sane =
            self.x.abs() <= MAX_WINDOW_EDGE as i32 && self.y.abs() <= MAX_WINDOW_EDGE as i32;

        size_is_sane && position_is_sane
    }
}

/// Shows the main window, restores its geometry, and drops the splash screen.
///
/// The frontend calls this once it has painted instead of working the window
/// itself, so that the reveal and the geometry restore cannot drift apart, and so
/// the splash screen is only closed once the main window is really on screen.
/// The 15s fallback in `open_main_window` calls this code as well.
#[command]
#[specta::specta]
pub fn reveal_window(app: AppHandle) -> AppResult<()> {
    reveal_main_window(&app)?;

    if let Some(splash) = app.get_webview_window("splashscreen") {
        if let Err(error) = splash.close() {
            log::warn!("Не вдалося закрити вікно заставки: {error}");
        }
    }

    Ok(())
}

pub fn reveal_main_window(app: &AppHandle) -> AppResult<()> {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        return Ok(());
    };

    let revealed = app
        .try_state::<Revealed>()
        .is_some_and(|revealed| revealed.0.swap(true, Ordering::SeqCst));

    log::debug!("Показ головного вікна, геометрію вже відновлено: {revealed}");

    window.show()?;

    if revealed {
        window.set_focus()?;

        return Ok(());
    }

    if let Some(saved) = saved_geometry(app) {
        if on_an_available_monitor(&window, saved) {
            if let Err(error) = apply_geometry(&window, saved) {
                log::warn!("Не вдалося відновити геометрію вікна: {error}");
            }

            // No more corrections for maximized windows
            if !saved.maximized {
                // The window settles after `show`, so confirm the result and correct
                verify_geometry(app.clone(), saved);
            }
        } else {
            log::info!("Збережене положення вікна поза доступними екранами, використовую типове");

            if saved.maximized {
                if let Err(error) = window.maximize() {
                    log::warn!("Не вдалося розгорнути вікно: {error}");
                }
            }
        }
    }

    window.set_focus()?;

    Ok(())
}

/// Whether any part of `saved` lands on a monitor that is currently attached.
fn on_an_available_monitor(window: &WebviewWindow, saved: SavedGeometry) -> bool {
    let Ok(monitors) = window.available_monitors() else {
        // Cannot tell, so trust the saved geometry rather than dropping it.
        return true;
    };

    monitors.iter().any(|monitor| {
        let position = monitor.position();
        let size = monitor.size();

        let overlaps_x =
            saved.x < position.x + size.width as i32 && saved.x + saved.width as i32 > position.x;
        let overlaps_y =
            saved.y < position.y + size.height as i32 && saved.y + saved.height as i32 > position.y;

        overlaps_x && overlaps_y
    })
}

/// Moves and resizes the window to `saved`, best effort.
fn apply_geometry(window: &WebviewWindow, saved: SavedGeometry) -> tauri::Result<()> {
    log::debug!("Застосовую геометрію вікна: {saved:?}");

    window.set_position(PhysicalPosition::new(saved.x, saved.y))?;

    let (frame_width, frame_height) = frame_size(window)?;

    window.set_size(PhysicalSize::new(
        saved.width.saturating_sub(frame_width),
        saved.height.saturating_sub(frame_height),
    ))?;

    if saved.maximized {
        window.maximize()?;
    }

    Ok(())
}

fn verify_geometry(app: AppHandle, saved: SavedGeometry) {
    for attempt in 0..MAX_CORRECTION_ATTEMPTS {
        thread::sleep(CORRECTION_DELAY);

        let app_for_main_thread = app.clone();
        let finished = app.run_on_main_thread(move || {
            let Some(window) = app_for_main_thread.get_webview_window(MAIN_WINDOW) else {
                return;
            };

            let Ok(actual) = read_geometry(&window) else {
                return;
            };

            if actual.maximized {
                return;
            }

            if covers_a_monitor(&window, actual).unwrap_or(false) {
                return;
            }

            if actual.width == saved.width && actual.height == saved.height {
                log::debug!(
                    "Геометрію вікна відновлено: {}x{} при ({}, {})",
                    actual.width,
                    actual.height,
                    actual.x,
                    actual.y
                );

                return;
            }

            log::debug!(
                "Геометрія вікна {}x{} замість {}x{}, спроба {}",
                actual.width,
                actual.height,
                saved.width,
                saved.height,
                attempt + 1
            );

            if let Err(error) = apply_geometry(&window, saved) {
                log::warn!("Не вдалося скоригувати геометрію вікна: {error}");
            }
        });

        if finished.is_err() {
            log::warn!("Не вдалося перевірити геометрію вікна");

            return;
        }
    }
}

/// Window size and position watchdog
pub fn track_window(window: &WebviewWindow) {
    let app = window.app_handle().clone();
    let pending = Arc::new(Mutex::new(None::<Instant>));

    app.manage(LastNormalGeometry::default());
    app.manage(Revealed::default());

    window.on_window_event(move |event| {
        if !matches!(
            event,
            WindowEvent::Moved(_) | WindowEvent::Resized(_) | WindowEvent::CloseRequested { .. }
        ) {
            return;
        }

        remember(&app, &pending);
    });
}

/// Writes the geometry out, at most once per [`SAVE_DEBOUNCE`] while the window is
/// still moving, with a trailing write once it settles.
fn remember(app: &AppHandle, pending: &Arc<Mutex<Option<Instant>>>) {
    {
        let Ok(mut last) = pending.lock() else {
            return;
        };

        let quiet = last.is_none_or(|at| at.elapsed() >= SAVE_DEBOUNCE);
        if !quiet {
            // A trailing write is already scheduled and will pick up this change.
            return;
        }

        *last = Some(Instant::now());
    }

    save_geometry(app);

    let app = app.clone();
    let pending = Arc::clone(pending);

    thread::spawn(move || {
        thread::sleep(SAVE_DEBOUNCE);

        {
            let Ok(mut last) = pending.lock() else {
                return;
            };

            if last.is_some_and(|at| at.elapsed() < SAVE_DEBOUNCE) {
                return;
            }

            *last = None;
        }

        save_geometry(&app);
    });
}

pub fn save_geometry(app: &AppHandle) {
    if let Err(error) = save_geometry_inner(app) {
        log::warn!("Не вдалося зберегти геометрію вікна: {error}");
    }
}

fn save_geometry_inner(app: &AppHandle) -> AppResult<()> {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        log::debug!("Головного вікна немає, геометрію не збережено");

        return Ok(());
    };

    if !window.is_visible()? || window.is_minimized()? {
        log::debug!("Головне вікно приховане або згорнуте, геометрію не збережено");

        return Ok(());
    }

    let last_normal = app.try_state::<LastNormalGeometry>();
    let remembered = last_normal
        .as_ref()
        .and_then(|remembered| remembered.0.lock().ok().and_then(|memory| *memory));

    if window.is_maximized()? {
        // A maximised window reports the maximised rect, so only the flag is worth updating
        let Some(saved) = remembered.or_else(|| saved_geometry(app)) else {
            return Ok(());
        };

        write_geometry(
            app,
            SavedGeometry {
                maximized: true,
                ..saved
            },
        )?;

        return Ok(());
    }

    let geometry = read_geometry(&window)?;

    if covers_a_monitor(&window, geometry)? {
        log::debug!("Вікно розгорнуто, геометрію для відновлення не змінено");

        return Ok(());
    }

    write_geometry(app, geometry)?;

    if let Some(remembered) = last_normal.as_ref() {
        if let Ok(mut memory) = remembered.0.lock() {
            *memory = Some(geometry);
        }
    }

    Ok(())
}

/// The window's outer rect plus its maximised state, read off a live window.
fn read_geometry(window: &WebviewWindow) -> tauri::Result<SavedGeometry> {
    let position = window.outer_position()?;
    let size = window.outer_size()?;
    let maximized = window.is_maximized()?;

    Ok(SavedGeometry {
        width: size.width,
        height: size.height,
        x: position.x,
        y: position.y,
        maximized,
    })
}

/// The difference between the outer rect and the client area (the window frame)
/// as the platform reports it for this window right now.
/// Measured from the window's own rects instead of assumed per platform.
fn frame_size(window: &WebviewWindow) -> tauri::Result<(u32, u32)> {
    let frame_width = window
        .outer_size()?
        .width
        .saturating_sub(window.inner_size()?.width);
    let frame_height = window
        .outer_size()?
        .height
        .saturating_sub(window.inner_size()?.height);

    Ok((frame_width, frame_height))
}

/// Whether a geometry is the window covering a whole monitor, i.e. the rect a
/// window reports while it is being maximised or restored.
///
/// This is needed on top of `is_maximized` because maximising is not atomic: the
/// window is moved and resized to the monitor first and only then does the
/// maximised flag flip, so the events in between carry the maximised rect while
/// still reporting `is_maximized() == false`. Storing that rect would make
/// un-maximising land on the monitor edge at monitor size.
///
/// The comparison is done against the monitor's work area rather than its full
/// bounds, because a maximised window is placed on the work area, which also
/// keeps this working on platforms where the maximised rect is not exactly the
/// work area. The allowance keeps a genuinely large window from being mistaken for
/// a maximised one.
fn covers_a_monitor(window: &WebviewWindow, geometry: SavedGeometry) -> tauri::Result<bool> {
    let Ok(Some(monitor)) = window.current_monitor() else {
        return Ok(false);
    };

    let area = monitor.work_area();
    let allowance = area.size.width.min(area.size.height) / 2;

    let covers_width = geometry.width >= area.size.width.saturating_sub(allowance);
    let covers_height = geometry.height >= area.size.height.saturating_sub(allowance);
    let starts_left = geometry.x <= area.position.x + allowance as i32;
    let starts_top = geometry.y <= area.position.y + allowance as i32;

    Ok(covers_width && covers_height && starts_left && starts_top)
}

fn state_path(app: &AppHandle) -> AppResult<PathBuf> {
    let state = app
        .try_state::<AppState>()
        .ok_or_else(|| crate::error::AppError::Generic("Стан застосунку ще не готовий".into()))?;

    Ok(state.data_dir.join(FILE_NAME))
}

pub fn saved_geometry(app: &AppHandle) -> Option<SavedGeometry> {
    let path = state_path(app).ok()?;
    let bytes = std::fs::read(path).ok()?;
    let geometry: SavedGeometry = serde_json::from_slice(&bytes).ok()?;

    geometry.is_plausible().then_some(geometry)
}

fn write_geometry(app: &AppHandle, geometry: SavedGeometry) -> AppResult<()> {
    let path = state_path(app)?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&path, serde_json::to_vec_pretty(&geometry)?)?;
    log::debug!("Збережено геометрію вікна: {geometry:?}");

    Ok(())
}
