#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod kcode;

use kcode::ensure_kcode_url;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, RunEvent, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_notification::NotificationExt;

const KCODE_PORT: u16 = 58627;
const WINDOW_LABEL: &str = "main";

#[derive(Default)]
struct AppState {
    ready: AtomicBool,
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, _event| {
                    let _ = show_main_window(app);
                })
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = show_main_window(app);
        }))
        .manage(AppState::default())
        .setup(|app| {
            let handle = app.handle().clone();

            // Request native notification permission on first launch.
            if let Ok(state) = app.notification().permission_state() {
                if state != tauri_plugin_notification::PermissionState::Granted {
                    let _ = app.notification().request_permission();
                }
            }

            setup_tray(app)?;
            setup_global_shortcut(app)?;

            // Auto-start the Kimi/acode web server and open the IDE window.
            tauri::async_runtime::spawn(async move {
                match ensure_kcode_url(KCODE_PORT, 30).await {
                    Ok(url) => {
                        if let Err(e) = create_or_show_window(&handle, &url).await {
                            eprintln!("KCode failed to open window: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("KCode failed to start server: {}", e);
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                window.app_handle().exit(0);
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let RunEvent::Ready = event {
                app_handle
                    .state::<AppState>()
                    .ready
                    .store(true, Ordering::Relaxed);
            }
        });
}

fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show_i = MenuItem::with_id(app, "show", "Show KCode", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit KCode", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = show_main_window(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                let _ = show_main_window(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn setup_global_shortcut(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut = tauri_plugin_global_shortcut::Shortcut::new(
        Some(
            tauri_plugin_global_shortcut::Modifiers::SHIFT
                | tauri_plugin_global_shortcut::Modifiers::SUPER,
        ),
        tauri_plugin_global_shortcut::Code::KeyK,
    );
    app.global_shortcut().register(shortcut)?;
    Ok(())
}

fn show_main_window(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(WINDOW_LABEL) {
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

async fn create_or_show_window(app: &tauri::AppHandle, url: &str) -> Result<(), String> {
    let parsed: url::Url = url.parse().map_err(|e: url::ParseError| e.to_string())?;

    if let Some(win) = app.get_webview_window(WINDOW_LABEL) {
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
        win.navigate(parsed).map_err(|e| e.to_string())?;
    } else {
        let init_script = include_str!("../assets/inject.js");
        let mut builder = WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::External(parsed))
            .title("KCode")
            .inner_size(1400.0, 900.0)
            .min_inner_size(800.0, 600.0)
            .resizable(true)
            .center()
            .theme(Some(tauri::Theme::Dark))
            .background_color(tauri::window::Color(13, 15, 18, 255))
            .initialization_script(init_script);

        // On macOS keep the native traffic lights but draw them as a
        // transparent overlay over the webview content. Hide the window
        // title text so there is no visible title bar.
        #[cfg(target_os = "macos")]
        {
            builder = builder
                .title_bar_style(tauri::TitleBarStyle::Overlay)
                .hidden_title(true);
        }

        builder.build().map_err(|e| e.to_string())?;
    }
    Ok(())
}
