mod events;
mod export;
mod model;
mod platform;
mod security;
mod settings;
mod update;

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use events::{EventLog, LogEvent};
use model::{DeviceKind, RiskLevel, SystemInfo, UsbDevice, UsbEvent};
use settings::{Settings, SettingsStore};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_notification::NotificationExt;

const TRAY_ID: &str = "main-tray";
const MINIMIZED_ARG: &str = "--minimized";
/// Show the "still running in the tray" hint only once per session.
static TRAY_HINT_SHOWN: AtomicBool = AtomicBool::new(false);

fn system_info_inner() -> SystemInfo {
    let env = |k: &str| std::env::var(k).unwrap_or_default();
    SystemInfo {
        is_windows: cfg!(windows),
        elevated: platform::is_elevated(),
        computer_name: if cfg!(windows) { env("COMPUTERNAME") } else { env("HOSTNAME") },
        user_name: if cfg!(windows) { env("USERNAME") } else { env("USER") },
        os: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
    }
}

/// Run blocking Win32 work off the main thread.
async fn blocking<T, F>(f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}

/// Device list with security analysis applied.
fn analyzed_devices(settings: &Settings) -> Result<Vec<UsbDevice>, String> {
    let mut list = platform::list_devices()?;
    security::annotate(&mut list, &platform::keyboard_parent_ids(), settings, chrono::Utc::now());
    Ok(list)
}

#[tauri::command]
async fn list_devices(store: State<'_, SettingsStore>) -> Result<Vec<UsbDevice>, String> {
    let settings = store.get();
    blocking(move || analyzed_devices(&settings)).await
}

// ---------------------------------------------------------------- event log

#[tauri::command]
async fn list_events(app: AppHandle, limit: Option<usize>) -> Result<Vec<LogEvent>, String> {
    blocking(move || Ok(app.state::<EventLog>().list(limit.unwrap_or(2000)))).await
}

#[tauri::command]
async fn clear_events(app: AppHandle) -> Result<(), String> {
    blocking(move || app.state::<EventLog>().clear()).await
}

#[tauri::command]
async fn export_events(app: AppHandle, path: String, format: String, lang: String) -> Result<(), String> {
    blocking(move || {
        let list = app.state::<EventLog>().list(usize::MAX);
        let content = match format.as_str() {
            "csv" => events::to_csv(&list, &lang),
            "json" => serde_json::to_string_pretty(&list).map_err(|e| e.to_string())?,
            other => return Err(format!("unknown format: {other}")),
        };
        std::fs::write(&path, content).map_err(|e| e.to_string())
    })
    .await
}

#[tauri::command]
fn system_info() -> SystemInfo {
    system_info_inner()
}

#[tauri::command]
async fn set_device_enabled(instance_id: String, enabled: bool) -> Result<(), String> {
    blocking(move || platform::set_enabled(&instance_id, enabled)).await
}

#[tauri::command]
async fn uninstall_device(instance_id: String) -> Result<(), String> {
    blocking(move || platform::uninstall(&instance_id)).await
}

#[tauri::command]
async fn eject_device(instance_id: String) -> Result<(), String> {
    blocking(move || platform::eject(&instance_id)).await
}

#[tauri::command]
fn relaunch_as_admin(app: AppHandle) -> Result<(), String> {
    platform::relaunch_as_admin()?;
    app.exit(0);
    Ok(())
}

#[tauri::command]
async fn export_devices(
    app: AppHandle,
    path: String,
    format: String,
    lang: String,
    devices: Vec<UsbDevice>,
) -> Result<String, String> {
    let version = app.package_info().version.to_string();
    blocking(move || {
        let system = system_info_inner();
        let meta = export::ReportMeta {
            system: &system,
            generated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            app_version: &version,
            lang: &lang,
        };
        let content = match format.as_str() {
            "csv" => export::to_csv(&devices, &lang),
            "json" => export::to_json(&devices, &meta),
            "html" => export::to_html(&devices, &meta),
            other => return Err(format!("unknown format: {other}")),
        };
        std::fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(export::digest(&devices))
    })
    .await
}

// ---------------------------------------------------------------- settings

#[tauri::command]
fn get_settings(store: State<'_, SettingsStore>) -> Settings {
    store.get()
}

#[tauri::command]
fn set_settings(app: AppHandle, store: State<'_, SettingsStore>, settings: Settings) -> Result<Settings, String> {
    let previous = store.get();
    let saved = store.set(settings)?;
    apply_settings(&app, &saved, Some(&previous));
    Ok(saved)
}

#[tauri::command]
async fn check_update(app: AppHandle) -> Result<update::UpdateInfo, String> {
    let current = app.package_info().version.to_string();
    blocking(move || update::check(&current)).await
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateProgress {
    downloaded: u64,
    total: Option<u64>,
}

#[tauri::command]
async fn download_update(app: AppHandle, url: String, sha256: Option<String>) -> Result<String, String> {
    let current = app.package_info().version.to_string();
    blocking(move || {
        update::download(&current, &url, sha256.as_deref(), |downloaded, total| {
            let _ = app.emit("update-progress", UpdateProgress { downloaded, total });
        })
        .map(|p| p.to_string_lossy().into_owned())
    })
    .await
}

#[tauri::command]
fn install_update(app: AppHandle, path: String) -> Result<(), String> {
    update::launch_installer(&path)?;
    // The installer also closes us; exiting first avoids a "file in use" prompt.
    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(600));
        handle.exit(0);
    });
    Ok(())
}

/// Push settings that live outside the web UI: window flags, autostart, tray labels.
fn apply_settings(app: &AppHandle, s: &Settings, previous: Option<&Settings>) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_always_on_top(s.always_on_top);
    }

    let autostart = app.autolaunch();
    let enabled = autostart.is_enabled().unwrap_or(false);
    if s.start_with_windows && !enabled {
        let _ = autostart.enable();
    } else if !s.start_with_windows && enabled {
        let _ = autostart.disable();
    }

    if previous.map(|p| p.lang != s.lang).unwrap_or(true) {
        if let (Some(tray), Ok(menu)) = (app.tray_by_id(TRAY_ID), tray_menu(app, &s.lang)) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

// ---------------------------------------------------------------- window & tray

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn tray_menu(app: &AppHandle, lang: &str) -> tauri::Result<Menu<tauri::Wry>> {
    let tr = lang == "tr";
    let show = MenuItem::with_id(app, "show", if tr { "UsbLens'i göster" } else { "Show UsbLens" }, true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", if tr { "Ayarlar" } else { "Settings" }, true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", if tr { "Çıkış" } else { "Quit" }, true, None::<&str>)?;
    Menu::with_items(app, &[&show, &settings, &sep, &quit])
}

fn build_tray(app: &AppHandle, lang: &str) -> tauri::Result<()> {
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("UsbLens")
        .menu(&tray_menu(app, lang)?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main(app),
            "settings" => {
                show_main(app);
                let _ = app.emit("open-settings", ());
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

fn tray_hint(app: &AppHandle, s: &Settings) {
    if !s.system_notifications || TRAY_HINT_SHOWN.swap(true, Ordering::SeqCst) {
        return;
    }
    let (title, body) = if s.lang == "tr" {
        ("UsbLens arka planda çalışıyor", "USB cihazları izlenmeye devam ediyor. Açmak için tepsi simgesine tıklayın.")
    } else {
        ("UsbLens is still running", "USB devices are still being monitored. Click the tray icon to open it.")
    };
    let _ = app.notification().builder().title(title).body(body).show();
}

// ---------------------------------------------------------------- storage triggers

/// Security analysis, allowlist enforcement and logging for a plug / unplug event.
/// Runs on the monitor worker thread, so it may block briefly.
fn process_usb_event(app: &AppHandle, mut event: UsbEvent) -> UsbEvent {
    let s = app.state::<SettingsStore>().get();

    if event.action == "arrival" && event.instance_id.is_some() {
        // Let Windows finish enumerating the interfaces (keyboard, storage) of the new device.
        std::thread::sleep(Duration::from_millis(1200));
        let id = event.instance_id.clone().unwrap_or_default();
        if let Ok(list) = analyzed_devices(&s) {
            if let Some(d) = list.into_iter().find(|d| d.instance_id.eq_ignore_ascii_case(&id)) {
                event.alerts = d.security.reasons.clone();
                event.level = d.security.level;
                if d.security.allowed == Some(false) && s.allowlist_action == "block" {
                    match platform::set_enabled(&d.security.root_id, false) {
                        Ok(()) => event.blocked = true,
                        Err(e) => event.block_error = Some(e),
                    }
                }
                event.device = Some(d);
            }
        }
        if event.level >= RiskLevel::Medium && s.system_notifications {
            security_notification(app, &event, &s.lang);
        }
    }

    if s.event_log {
        let user = std::env::var(if cfg!(windows) { "USERNAME" } else { "USER" }).unwrap_or_default();
        let at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        if let Err(e) = app.state::<EventLog>().append(&LogEvent::from_event(&event, at, user)) {
            eprintln!("event log: {e}");
        }
    }
    event
}

fn security_notification(app: &AppHandle, event: &UsbEvent, lang: &str) {
    let tr = lang == "tr";
    let name = event.device.as_ref().map(|d| d.name.as_str()).unwrap_or("USB");
    let title = match (event.blocked, tr) {
        (true, true) => format!("⛔ Engellendi: {name}"),
        (true, false) => format!("⛔ Blocked: {name}"),
        (false, true) => format!("⚠ Güvenlik uyarısı: {name}"),
        (false, false) => format!("⚠ Security alert: {name}"),
    };
    let body = event
        .alerts
        .iter()
        .map(|r| security::reason_text(r, lang))
        .collect::<Vec<_>>()
        .join("\n");
    let _ = app.notification().builder().title(title).body(body).show();
}

fn on_usb_event(app: &AppHandle, event: &UsbEvent) {
    if event.action != "arrival" || event.blocked {
        return;
    }
    let Some(device) = &event.device else { return };
    if device.kind != DeviceKind::Storage {
        return;
    }
    let s = app.state::<SettingsStore>().get();

    if s.bring_to_front_on_storage {
        show_main(app);
    }
    if s.open_explorer_on_storage {
        let id = device.instance_id.clone();
        // Drive letters are assigned a little after the device arrives.
        std::thread::spawn(move || {
            for _ in 0..12 {
                if let Some(letter) = platform::get_device(&id).and_then(|d| d.drive_letters.into_iter().next()) {
                    open_in_explorer(&letter);
                    return;
                }
                std::thread::sleep(Duration::from_millis(500));
            }
        });
    }
}

fn open_in_explorer(letter: &str) {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("explorer.exe").arg(format!("{letter}\\")).spawn();
    }
    #[cfg(not(windows))]
    {
        let _ = letter;
    }
}

// ---------------------------------------------------------------- entry point

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must be first: a second launch just focuses the running window.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if !args.iter().any(|a| a == MINIMIZED_ARG) {
                show_main(app);
            }
        }))
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![MINIMIZED_ARG])))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            let store = SettingsStore::load(config_dir.join("settings.json"));
            let s = store.get();
            app.manage(store);
            let data_dir = app.path().app_data_dir()?;
            app.manage(EventLog::new(data_dir.join("events.jsonl")));

            let handle = app.handle().clone();
            if let Err(e) = build_tray(&handle, &s.lang) {
                eprintln!("tray icon could not be created: {e}");
            }
            apply_settings(&handle, &s, None);

            // The window is created hidden (see tauri.conf.json) to avoid a flash.
            let launched_minimized = std::env::args().any(|a| a == MINIMIZED_ARG);
            if let Some(w) = app.get_webview_window("main") {
                if launched_minimized || s.start_minimized {
                    if !s.close_to_tray {
                        let _ = w.show();
                        let _ = w.minimize();
                    }
                } else {
                    let _ = w.show();
                }
            }

            let monitor_handle = handle.clone();
            if let Err(e) = platform::start_monitor(move |event| {
                let event = process_usb_event(&monitor_handle, event);
                on_usb_event(&monitor_handle, &event);
                let _ = monitor_handle.emit("usb-event", event);
            }) {
                eprintln!("USB monitor could not start: {e}");
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let s = app.state::<SettingsStore>().get();
                if s.close_to_tray && app.tray_by_id(TRAY_ID).is_some() {
                    api.prevent_close();
                    let _ = window.hide();
                    tray_hint(app, &s);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            list_devices,
            system_info,
            set_device_enabled,
            uninstall_device,
            eject_device,
            relaunch_as_admin,
            export_devices,
            get_settings,
            set_settings,
            check_update,
            download_update,
            install_update,
            list_events,
            clear_events,
            export_events,
        ])
        .run(tauri::generate_context!())
        .expect("error while running UsbLens");
}
