//! The Tauri shell. Each command here will stay thin: parse the input, call one
//! function in `waypoint_domain` or `waypoint_read`, return the result
//! (ADR 0001 §2). There are no commands yet.

// `mobile_entry_point` lets the same `run` start the app on iOS and Android.
// On desktop it does nothing, and `main.rs` calls `run` directly.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // `generate_context!` reads tauri.conf.json and embeds the built
        // frontend (`dist/`) into the binary at compile time.
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
