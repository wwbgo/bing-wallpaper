#![windows_subsystem = "windows"]

fn main() {
    if let Err(error) = bing_wallpaper::run_worker_from_env() {
        let _ = bing_wallpaper::append_log(&format!("unhandled worker error: {error:#}"));
    }
}
