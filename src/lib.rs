pub mod bing;
pub mod cli;
pub mod display;
pub mod scheduler;
pub mod state;
pub mod wallpaper;

use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use time::{OffsetDateTime, UtcOffset};
use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows::Win32::System::Threading::{CreateMutexW, ReleaseMutex};
use windows::core::PCWSTR;

use crate::cli::{Resolution, RunArgs, SetupArgs};
use crate::state::AppState;

pub struct AppDirs {
    pub root: PathBuf,
    pub images: PathBuf,
    pub state_file: PathBuf,
    pub logs: PathBuf,
}

impl AppDirs {
    pub fn new() -> Result<Self> {
        let root = local_app_data_dir()?.join("BingWallpaper");
        Ok(Self {
            images: root.join("images"),
            state_file: root.join("state.json"),
            logs: root.join("logs"),
            root,
        })
    }

    pub fn ensure(&self) -> Result<()> {
        fs::create_dir_all(&self.root)
            .with_context(|| format!("failed to create {}", self.root.display()))?;
        fs::create_dir_all(&self.images)
            .with_context(|| format!("failed to create {}", self.images.display()))?;
        fs::create_dir_all(&self.logs)
            .with_context(|| format!("failed to create {}", self.logs.display()))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct RunOutcome {
    pub changed: bool,
    pub startdate: String,
    pub image_path: PathBuf,
    pub copyright: String,
}

struct SingleInstanceGuard {
    handle: HANDLE,
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseMutex(self.handle);
            let _ = CloseHandle(self.handle);
        }
    }
}

pub fn run_once(args: &RunArgs) -> Result<RunOutcome> {
    let dirs = AppDirs::new()?;
    dirs.ensure()?;
    let _guard = acquire_single_instance()?;

    let state = load_state(&dirs.state_file).unwrap_or_default();
    let today = local_date_string();
    let wallpaper = &args.wallpaper;
    let resolution = effective_resolution(wallpaper.resolution)?;
    let resolution_name = resolution_name(resolution);

    if !args.force
        && !args.dry_run
        && state.last_local_run_date == today
        && Path::new(&state.last_image).is_file()
        && state.last_resolution.as_deref() == Some(resolution_name)
    {
        return Ok(RunOutcome {
            changed: false,
            startdate: state.last_startdate,
            image_path: state.last_image,
            copyright: state.copyright,
        });
    }

    let image = bing::fetch_today(&wallpaper.host, &wallpaper.mkt)?;
    let image_path = dirs.images.join(format!("{}.jpg", image.startdate));

    if args.dry_run {
        return Ok(RunOutcome {
            changed: true,
            startdate: image.startdate,
            image_path,
            copyright: image.copyright,
        });
    }

    let resolution_matches = state.last_resolution.as_deref() == Some(resolution_name);
    if should_download_image(args.force, image_path.is_file(), resolution_matches) {
        bing::download_image(&wallpaper.host, &image, resolution, &image_path)?;
    }

    wallpaper::set_wallpaper(&image_path, wallpaper.style)?;

    let new_state = AppState {
        last_startdate: image.startdate.clone(),
        last_image: image_path.clone(),
        copyright: image.copyright.clone(),
        updated_at: local_timestamp(),
        last_local_run_date: today,
        last_resolution: Some(resolution_name.to_string()),
    };
    new_state.save(&dirs.state_file)?;

    cleanup_images(&dirs.images, wallpaper.keep, &image_path)?;

    Ok(RunOutcome {
        changed: true,
        startdate: image.startdate,
        image_path,
        copyright: image.copyright,
    })
}

fn effective_resolution(resolution: Resolution) -> Result<Resolution> {
    match resolution {
        Resolution::Auto => display::auto_resolution(),
        explicit => Ok(explicit),
    }
}

fn resolution_name(resolution: Resolution) -> &'static str {
    match resolution {
        Resolution::Auto => "auto",
        Resolution::Fhd => "1080",
        Resolution::Uhd => "uhd",
    }
}

fn should_download_image(force: bool, image_exists: bool, resolution_matches: bool) -> bool {
    force || !image_exists || !resolution_matches
}

pub fn setup(args: &SetupArgs) -> Result<()> {
    let run = RunArgs {
        wallpaper: args.wallpaper.clone(),
        force: false,
        dry_run: false,
    };

    match run_once(&run) {
        Ok(outcome) => {
            if outcome.changed {
                println!(
                    "wallpaper updated: startdate={} image={} copyright={}",
                    outcome.startdate,
                    outcome.image_path.display(),
                    outcome.copyright
                );
            } else {
                println!("wallpaper is already current: {}", outcome.startdate);
            }
        }
        Err(error) => {
            eprintln!("warning: failed to update wallpaper now: {error:#}");
            let _ = append_log(&format!("setup update failed, continuing: {error:#}"));
        }
    }

    scheduler::install(&args.wallpaper, &args.time)
}

pub fn run_worker_from_env() -> Result<()> {
    use clap::Parser;

    let cli = cli::WorkerCli::parse();
    let run = RunArgs {
        wallpaper: cli.wallpaper,
        force: false,
        dry_run: false,
    };
    match run_once(&run) {
        Ok(outcome) => {
            if outcome.changed {
                append_log(&format!(
                    "wallpaper updated: startdate={} path={}",
                    outcome.startdate,
                    outcome.image_path.display()
                ))?;
            }
            Ok(())
        }
        Err(error) => {
            let _ = append_log(&format!("worker failed: {error:#}"));
            Err(error)
        }
    }
}

pub fn status() -> Result<()> {
    let dirs = AppDirs::new()?;
    println!("app directory: {}", dirs.root.display());
    println!("image directory: {}", dirs.images.display());
    println!("state file: {}", dirs.state_file.display());
    println!("log directory: {}", dirs.logs.display());

    match AppState::load(&dirs.state_file) {
        Ok(Some(state)) => {
            println!("\n[state]");
            println!("startdate: {}", state.last_startdate);
            println!("local_run_date: {}", state.last_local_run_date);
            println!("image: {}", state.last_image.display());
            println!("copyright: {}", state.copyright);
            println!(
                "resolution: {}",
                state.last_resolution.as_deref().unwrap_or("unknown")
            );
            println!("updated_at: {}", state.updated_at);
        }
        Ok(None) => println!("\n[state]\nnot created yet"),
        Err(error) => println!("\n[state]\nstate file is invalid: {error:#}"),
    }

    scheduler::print_query_status()
}

pub fn append_log(message: &str) -> Result<()> {
    let dirs = AppDirs::new()?;
    dirs.ensure()?;

    let log_path = dirs.logs.join("worker.log");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("failed to open log file {}", log_path.display()))?;

    writeln!(file, "[{}] {message}", local_timestamp())?;
    Ok(())
}

fn acquire_single_instance() -> Result<SingleInstanceGuard> {
    let name = wide_null("Local\\bing-wallpaper-update");

    unsafe {
        let handle = CreateMutexW(None, true, PCWSTR(name.as_ptr()))
            .context("failed to create single-instance mutex")?;
        if GetLastError() == ERROR_ALREADY_EXISTS {
            let _ = CloseHandle(handle);
            anyhow::bail!("another Bing Wallpaper update is already running");
        }

        Ok(SingleInstanceGuard { handle })
    }
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn load_state(path: &Path) -> Result<AppState> {
    match AppState::load(path) {
        Ok(Some(state)) => Ok(state),
        Ok(None) => Ok(AppState::default()),
        Err(error) => {
            let _ = append_log(&format!(
                "failed to read state, using empty state: {error:#}"
            ));
            Ok(AppState::default())
        }
    }
}

fn cleanup_images(dir: &Path, keep: usize, current: &Path) -> Result<()> {
    let keep = keep.max(1);
    let mut images = Vec::new();

    for entry in fs::read_dir(dir)
        .with_context(|| format!("failed to read image directory {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("jpg") {
            images.push(path);
        }
    }

    images.sort();
    images.reverse();

    for old in images.into_iter().skip(keep) {
        if old != current {
            fs::remove_file(&old)
                .with_context(|| format!("failed to remove old image {}", old.display()))?;
        }
    }

    Ok(())
}

fn local_app_data_dir() -> Result<PathBuf> {
    if let Some(path) = env::var_os("LOCALAPPDATA") {
        return Ok(PathBuf::from(path));
    }

    if let Some(profile) = env::var_os("USERPROFILE") {
        return Ok(PathBuf::from(profile).join("AppData").join("Local"));
    }

    bail_local_app_data()
}

fn bail_local_app_data() -> Result<PathBuf> {
    anyhow::bail!("LOCALAPPDATA is not available")
}

fn local_date_string() -> String {
    let now = local_now();
    let (year, month, day) = now.to_calendar_date();
    format!("{year:04}{:02}{:02}", u8::from(month), day)
}

fn local_timestamp() -> String {
    let now = local_now();
    let (year, month, day) = now.to_calendar_date();
    let (hour, minute, second) = now.to_hms();
    format!(
        "{year:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        u8::from(month),
        day,
        hour,
        minute,
        second
    )
}

fn local_now() -> OffsetDateTime {
    OffsetDateTime::now_local()
        .unwrap_or_else(|_| OffsetDateTime::now_utc().to_offset(UtcOffset::UTC))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn force_downloads_cached_image_even_when_resolution_matches() {
        assert!(should_download_image(true, true, true));
    }

    #[test]
    fn normal_run_uses_cached_image_when_resolution_matches() {
        assert!(!should_download_image(false, true, true));
    }

    #[test]
    fn normal_run_downloads_when_image_is_missing_or_resolution_differs() {
        assert!(should_download_image(false, false, true));
        assert!(should_download_image(false, true, false));
    }
}
