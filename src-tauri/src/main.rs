#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{WebviewUrl, WebviewWindowBuilder};

// Matikan animasi & transisi supaya CPU/RAM lebih hemat.
const INIT_JS: &str = r#"
document.addEventListener('DOMContentLoaded', () => {
  const s = document.createElement('style');
  s.textContent = '*{animation:none!important;transition:none!important;scroll-behavior:auto!important}';
  document.head.appendChild(s);
});
"#;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let url = "https://web.whatsapp.com/".parse().unwrap();
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title("WA Ringan")
                .inner_size(1000.0, 700.0)
                .min_inner_size(420.0, 500.0)
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
                .additional_browser_args(
                    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection \
                     --renderer-process-limit=1 \
                     --js-flags=--max-old-space-size=384 \
                     --disable-background-networking \
                     --disable-breakpad",
                )
                .initialization_script(INIT_JS)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("gagal menjalankan WA Ringan");
}
