#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;

use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

const BASE_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection \
    --renderer-process-limit=1 \
    --js-flags=--max-old-space-size=384 \
    --disable-background-networking \
    --disable-breakpad";

// Matikan animasi/transisi, dan hentikan GIF (video loop tanpa suara) agar tidak terus berjalan.
const INIT_JS: &str = r#"
document.addEventListener('DOMContentLoaded', () => {
  const s = document.createElement('style');
  s.textContent = '*{animation:none!important;transition:none!important;scroll-behavior:auto!important}';
  document.head.appendChild(s);
  setInterval(() => {
    document.querySelectorAll('video').forEach(v => {
      if (v.loop && v.muted && !v.dataset.wr) {
        v.dataset.wr = '1';
        try { v.pause(); } catch (e) {}
      }
    });
  }, 3000);
});
"#;

// File penanda: kalau ada, gambar diblokir saat aplikasi dibuka.
fn flag_path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|d| {
        let _ = fs::create_dir_all(&d);
        d.join("block_images")
    })
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn build_tray(app: &AppHandle, block_images: bool) -> tauri::Result<()> {
    let autostart_on = app.autolaunch().is_enabled().unwrap_or(false);

    let open = MenuItem::with_id(app, "open", "Buka WA Ringan", true, None::<&str>)?;
    let reload = MenuItem::with_id(app, "reload", "Muat ulang (bersihkan memori)", true, None::<&str>)?;
    let auto = CheckMenuItem::with_id(app, "autostart", "Jalan otomatis saat Windows menyala", true, autostart_on, None::<&str>)?;
    let noimg = CheckMenuItem::with_id(app, "noimg", "Blokir gambar (hemat RAM, akan restart)", true, block_images, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Keluar", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(app, &[&open, &reload, &sep1, &auto, &noimg, &sep2, &quit])?;

    let mut tray = TrayIconBuilder::new().tooltip("WA Ringan").menu(&menu).show_menu_on_left_click(false);
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }

    tray.on_menu_event(|app, event| match event.id().as_ref() {
        "open" => show_main(app),
        "reload" => {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.eval("location.reload()");
            }
        }
        "autostart" => {
            let al = app.autolaunch();
            if al.is_enabled().unwrap_or(false) {
                let _ = al.disable();
            } else {
                let _ = al.enable();
            }
        }
        "noimg" => {
            if let Some(p) = flag_path(app) {
                if p.exists() {
                    let _ = fs::remove_file(&p);
                } else {
                    let _ = fs::write(&p, b"1");
                }
            }
            app.restart();
        }
        "quit" => app.exit(0),
        _ => {}
    })
    .on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            show_main(tray.app_handle());
        }
    })
    .build(app)?;

    Ok(())
}

fn main() {
    let start_hidden = std::env::args().any(|a| a == "--hidden");

    tauri::Builder::default()
        // Hanya satu jendela: kalau dibuka lagi, jendela yang ada dimunculkan.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .setup(move |app| {
            let handle = app.handle().clone();
            let block_images = flag_path(&handle).map(|p| p.exists()).unwrap_or(false);

            let mut args = String::from(BASE_ARGS);
            if block_images {
                args.push_str(" --blink-settings=imagesEnabled=false");
            }

            let url = "https://web.whatsapp.com/".parse().unwrap();
            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title("WA Ringan")
                .inner_size(1000.0, 700.0)
                .min_inner_size(420.0, 500.0)
                .visible(!start_hidden)
                .user_agent(UA)
                .additional_browser_args(&args)
                .initialization_script(INIT_JS)
                .build()?;

            // Tombol X = sembunyikan ke tray, bukan keluar.
            let w = window.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = w.hide();
                }
            });

            build_tray(&handle, block_images)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("gagal menjalankan WA Ringan");
}
