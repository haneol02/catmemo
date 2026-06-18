#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager, WebviewUrl, WebviewWindowBuilder,
};

// ─── Win32 cursor position (custom command — no capability needed) ────────────

#[cfg(target_os = "windows")]
mod win32 {
    #[repr(C)]
    pub struct POINT { pub x: i32, pub y: i32 }
    #[link(name = "user32")]
    extern "system" { pub fn GetCursorPos(lp: *mut POINT) -> i32; }
}

#[cfg(target_os = "windows")]
fn cursor_pos_impl() -> Result<(i32, i32), String> {
    unsafe {
        let mut pt = win32::POINT { x: 0, y: 0 };
        if win32::GetCursorPos(&mut pt) != 0 { Ok((pt.x, pt.y)) }
        else { Err("GetCursorPos failed".to_string()) }
    }
}
#[cfg(not(target_os = "windows"))]
fn cursor_pos_impl() -> Result<(i32, i32), String> { Err("not supported".to_string()) }

#[tauri::command]
async fn get_cursor_pos() -> Result<(i32, i32), String> { cursor_pos_impl() }

// ─── Data helpers ────────────────────────────────────────────────────────────

fn data_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| e.to_string())
        .map(|d| d.join("memos.json"))
}

#[tauri::command]
async fn load_data(app: tauri::AppHandle) -> Result<String, String> {
    let path = data_path(&app)?;
    if path.exists() {
        std::fs::read_to_string(&path).map_err(|e| e.to_string())
    } else {
        Ok(r##"{"memos":[],"settings":{"breed":"orange","defaultColor":"#fef9c3"}}"##.to_string())
    }
}

#[tauri::command]
async fn save_data(app: tauri::AppHandle, data: String) -> Result<(), String> {
    let path = data_path(&app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, data).map_err(|e| e.to_string())
}

// ─── Window helpers ───────────────────────────────────────────────────────────

/// Returns (x, y) for a new window, clamped so it stays on screen.
/// `anchor` = top-left of the cat window in screen coordinates.
fn clamped_position(
    app: &tauri::AppHandle,
    win_w: f64,
    win_h: f64,
    x: f64,
    y: f64,
) -> (f64, f64) {
    let (sw, sh) = app
        .get_webview_window("cat")
        .as_ref()
        .and_then(|w| w.current_monitor().ok().flatten())
        .map(|m| (m.size().width as f64, m.size().height as f64))
        .unwrap_or((1920.0, 1080.0));
    (x.max(0.0).min(sw - win_w), y.max(0.0).min(sh - win_h))
}

fn cat_top_left(app: &tauri::AppHandle) -> (f64, f64) {
    app.get_webview_window("cat")
        .and_then(|w| w.outer_position().ok())
        .map(|p| (p.x as f64, p.y as f64))
        .unwrap_or((300.0, 200.0))
}

#[tauri::command]
async fn clamp_cat_window(app: tauri::AppHandle) -> Result<(), String> {
    let cat = match app.get_webview_window("cat") {
        Some(w) => w,
        None => return Ok(()),
    };
    let pos = match cat.outer_position() {
        Ok(p) => p,
        Err(_) => return Ok(()),
    };
    let (sw, sh) = cat
        .current_monitor()
        .ok()
        .flatten()
        .map(|m| (m.size().width as i32, m.size().height as i32))
        .unwrap_or((1920, 1080));
    let sz = 220i32;
    let nx = pos.x.max(0).min(sw - sz);
    let ny = pos.y.max(0).min(sh - sz);
    if nx != pos.x || ny != pos.y {
        let _ = cat.set_position(tauri::Position::Physical(
            tauri::PhysicalPosition::new(nx, ny),
        ));
    }
    Ok(())
}

#[tauri::command]
async fn create_memo_window(app: tauri::AppHandle, memo_id: String) -> Result<(), String> {
    let label = format!("memo-{}", memo_id);
    if let Some(win) = app.get_webview_window(&label) {
        let (cat_x, cat_y) = cat_top_left(&app);
        let memo_w = 300.0_f64;
        let memo_h = 270.0_f64;
        let gap = 8.0_f64;
        let (px, py) = clamped_position(&app, memo_w, memo_h, cat_x - memo_w - gap, cat_y);
        let _ = win.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(px, py)));
        let _ = win.show();
        let _ = win.set_focus();
        return Ok(());
    }
    let (cat_x, cat_y) = cat_top_left(&app);
    let memo_w = 300.0_f64;
    let memo_h = 270.0_f64;
    let gap = 8.0_f64;
    // Spawn memo to the upper-left of the cat
    let (px, py) = clamped_position(&app, memo_w, memo_h, cat_x - memo_w - gap, cat_y);
    let url = format!("memo.html?id={}", memo_id);
    WebviewWindowBuilder::new(&app, &label, WebviewUrl::App(url.into()))
        .title("")
        .inner_size(memo_w, memo_h)
        .position(px, py)
        .decorations(false)
        .transparent(true)
        .always_on_top(false)
        .resizable(true)
        .shadow(false)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn open_list_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("list") {
        let (cat_x, cat_y) = cat_top_left(&app);
        let (px, py) = clamped_position(&app, 360.0, 520.0, cat_x - 368.0, cat_y);
        let _ = win.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(px, py)));
        let _ = win.show();
        let _ = win.set_focus();
        return Ok(());
    }
    let (cat_x, cat_y) = cat_top_left(&app);
    let (px, py) = clamped_position(&app, 360.0, 520.0, cat_x - 368.0, cat_y);
    WebviewWindowBuilder::new(&app, "list", WebviewUrl::App("list.html".into()))
        .title("")
        .inner_size(360.0, 520.0)
        .position(px, py)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("settings") {
        let (cat_x, cat_y) = cat_top_left(&app);
        let (px, py) = clamped_position(&app, 340.0, 420.0, cat_x - 348.0, cat_y);
        let _ = win.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(px, py)));
        let _ = win.show();
        let _ = win.set_focus();
        return Ok(());
    }
    let (cat_x, cat_y) = cat_top_left(&app);
    let (px, py) = clamped_position(&app, 340.0, 420.0, cat_x - 348.0, cat_y);
    WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("settings.html".into()))
        .title("")
        .inner_size(340.0, 420.0)
        .position(px, py)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
async fn toggle_memos(app: tauri::AppHandle) -> Result<(), String> {
    let windows = app.webview_windows();
    let memo_wins: Vec<_> = windows.into_values()
        .filter(|w| w.label().starts_with("memo-"))
        .collect();
    if memo_wins.is_empty() { return Ok(()); }
    let any_visible = memo_wins.iter().any(|w| w.is_visible().unwrap_or(false));
    for win in memo_wins {
        if any_visible { let _ = win.hide(); } else { let _ = win.show(); }
    }
    Ok(())
}

#[tauri::command]
async fn move_cat_window(app: tauri::AppHandle, x: i32, y: i32) -> Result<(), String> {
    let cat = match app.get_webview_window("cat") { Some(w) => w, None => return Ok(()) };
    let (sw, sh) = cat
        .current_monitor().ok().flatten()
        .map(|m| (m.size().width as i32, m.size().height as i32))
        .unwrap_or((1920, 1080));
    let sz = 220i32;
    let nx = x.max(0).min(sw - sz);
    let ny = y.max(0).min(sh - sz);
    let _ = cat.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(nx, ny)));
    Ok(())
}

#[tauri::command]
async fn show_cat_menu(app: tauri::AppHandle, x: i32, y: i32) -> Result<(), String> {
    let mw = 170i32;
    let mh = 290i32;
    let (sw, sh) = app
        .get_webview_window("cat").as_ref()
        .and_then(|w| w.current_monitor().ok().flatten())
        .map(|m| (m.size().width as i32, m.size().height as i32))
        .unwrap_or((1920, 1080));
    let cx = x.max(0).min(sw - mw);
    // 메뉴를 클릭 지점 위에 표시 (bottom = click Y)
    let cy = (y - mh).max(0).min(sh - mh);
    if let Some(win) = app.get_webview_window("catmenu") {
        let _ = win.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(cx, cy)));
        let _ = win.show();
        let _ = win.set_focus();
        return Ok(());
    }
    WebviewWindowBuilder::new(&app, "catmenu", WebviewUrl::App("catmenu.html".into()))
        .title("")
        .inner_size(170.0, 290.0)
        .position(cx as f64, cy as f64)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .resizable(false)
        .shadow(false)
        .skip_taskbar(true)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── main ─────────────────────────────────────────────────────────────────────

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_data,
            save_data,
            create_memo_window,
            open_list_window,
            open_settings_window,
            show_cat_menu,
            move_cat_window,
            quit_app,
            toggle_memos,
            clamp_cat_window,
            get_cursor_pos,
        ])
        .setup(|app| {
            // System tray
            let list_i = MenuItem::with_id(app, "tray_list", "메모 목록", true, None::<&str>)?;
            let sep    = PredefinedMenuItem::separator(app)?;
            let show_i = MenuItem::with_id(app, "tray_show", "보이기",   true, None::<&str>)?;
            let hide_i = MenuItem::with_id(app, "tray_hide", "숨기기",   true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "tray_quit", "종료",     true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&list_i, &sep, &show_i, &hide_i, &quit_i])?;

            let mut tray = TrayIconBuilder::new()
                .menu(&tray_menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "tray_quit" => app.exit(0),
                    "tray_hide" => {
                        if let Some(w) = app.get_webview_window("cat") { let _ = w.hide(); }
                    }
                    "tray_show" => {
                        if let Some(w) = app.get_webview_window("cat") { let _ = w.show(); }
                    }
                    "tray_list" => {
                        let h = app.clone();
                        tauri::async_runtime::spawn(async move { let _ = open_list_window(h).await; });
                    }
                    _ => {}
                });

            // Set icon if available (avoids panic when icon files are missing)
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }

            tray.build(app)?;

            // Cat window: CloseRequested → hide instead of quit
            if let Some(cat) = app.get_webview_window("cat") {
                let w = cat.clone();
                cat.on_window_event(move |e| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = e {
                        api.prevent_close();
                        let _ = w.hide();
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("CatMemo startup failed");
}
