use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE};

use crate::cli::WallpaperOptions;

const DAILY_TASK: &str = r"BingWallpaper\Daily";
const AUTOSTART_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const AUTOSTART_VALUE: &str = "BingWallpaper";

pub fn install(wallpaper: &WallpaperOptions, time: &str) -> Result<()> {
    let normalized_time = normalize_time(time)?;

    let worker = worker_exe()?;
    let command = worker_command(&worker, wallpaper);

    create_task(
        DAILY_TASK,
        &[
            "/SC",
            "DAILY",
            "/ST",
            normalized_time.as_str(),
            "/TR",
            command.as_str(),
        ],
    )?;
    println!("registered daily task: {DAILY_TASK}");

    register_autostart(&command)?;
    println!("registered logon autostart: {AUTOSTART_PATH}\\{AUTOSTART_VALUE}");

    Ok(())
}

pub fn uninstall() -> Result<()> {
    let output = schtasks_output(&["/Delete", "/F", "/TN", DAILY_TASK])?;
    if output.status.success() {
        println!("removed scheduled task: {DAILY_TASK}");
    } else {
        println!("scheduled task not removed or not found: {DAILY_TASK}");
        print_schtasks_text(&output);
    }

    remove_autostart()?;
    println!("removed logon autostart: {AUTOSTART_VALUE}");
    Ok(())
}

pub fn print_query_status() -> Result<()> {
    println!("\n[{DAILY_TASK}]");
    let output = schtasks_output(&["/Query", "/TN", DAILY_TASK, "/FO", "LIST"])?;
    print_schtasks_text(&output);
    if !output.status.success() {
        println!("status: not installed or unavailable");
    }

    println!("\n[logon autostart]");
    match autostart_command()? {
        Some(command) => {
            println!("status: enabled");
            println!("command: {command}");
        }
        None => println!("status: disabled"),
    }

    Ok(())
}

fn register_autostart(command: &str) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run_key, _) = hkcu
        .create_subkey(AUTOSTART_PATH)
        .context("failed to open HKCU Run key")?;
    run_key
        .set_value(AUTOSTART_VALUE, &command)
        .context("failed to write HKCU Run value")?;
    Ok(())
}

fn remove_autostart() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = match hkcu.open_subkey_with_flags(AUTOSTART_PATH, KEY_QUERY_VALUE | KEY_SET_VALUE)
    {
        Ok(key) => key,
        Err(_) => return Ok(()),
    };

    match run_key.delete_value(AUTOSTART_VALUE) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).context("failed to remove HKCU Run value"),
    }
}

fn autostart_command() -> Result<Option<String>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = match hkcu.open_subkey_with_flags(AUTOSTART_PATH, KEY_QUERY_VALUE) {
        Ok(key) => key,
        Err(_) => return Ok(None),
    };

    match run_key.get_value::<String, _>(AUTOSTART_VALUE) {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).context("failed to read HKCU Run value"),
    }
}

fn create_task(name: &str, extra: &[&str]) -> Result<()> {
    let mut args = vec!["/Create", "/F", "/TN", name];
    args.extend_from_slice(extra);

    let output = schtasks_output(&args)
        .with_context(|| format!("failed to register scheduled task {name}"))?;
    if !output.status.success() {
        print_schtasks_text(&output);
        bail!("failed to register scheduled task {name}");
    }
    Ok(())
}

fn worker_exe() -> Result<PathBuf> {
    let current = env::current_exe().context("failed to resolve current executable path")?;
    let parent = current
        .parent()
        .context("current executable path has no parent directory")?;
    let worker = parent.join("bing-wallpaper-worker.exe");

    if worker.is_file() {
        Ok(worker)
    } else {
        bail!(
            "worker executable is missing: {}\nbuild all binaries with `cargo build --bins`",
            worker.display()
        )
    }
}

fn worker_command(worker: &Path, wallpaper: &WallpaperOptions) -> String {
    let mut parts = vec![quote_arg(&worker.display().to_string())];

    parts.push("--mkt".to_string());
    parts.push(quote_arg(&wallpaper.mkt));
    parts.push("--host".to_string());
    parts.push(quote_arg(&wallpaper.host));
    parts.push("--resolution".to_string());
    parts.push(quote_arg(resolution_name(wallpaper.resolution)));
    parts.push("--style".to_string());
    parts.push(quote_arg(style_name(wallpaper.style)));
    parts.push("--keep".to_string());
    parts.push(wallpaper.keep.to_string());

    parts.join(" ")
}

fn resolution_name(resolution: crate::cli::Resolution) -> &'static str {
    match resolution {
        crate::cli::Resolution::Auto => "auto",
        crate::cli::Resolution::Fhd => "1080",
        crate::cli::Resolution::Uhd => "uhd",
    }
}

fn style_name(style: crate::cli::WallpaperStyle) -> &'static str {
    match style {
        crate::cli::WallpaperStyle::Fill => "fill",
        crate::cli::WallpaperStyle::Fit => "fit",
        crate::cli::WallpaperStyle::Stretch => "stretch",
        crate::cli::WallpaperStyle::Center => "center",
        crate::cli::WallpaperStyle::Span => "span",
        crate::cli::WallpaperStyle::Tile => "tile",
    }
}

fn quote_arg(value: &str) -> String {
    if !value.is_empty() && !value.contains([' ', '\t', '"']) {
        return value.to_string();
    }

    let mut quoted = String::from("\"");
    let mut backslashes = 0;

    for ch in value.chars() {
        if ch == '\\' {
            backslashes += 1;
            continue;
        }

        if ch == '"' {
            quoted.extend(std::iter::repeat_n('\\', backslashes * 2 + 1));
            quoted.push('"');
        } else {
            quoted.extend(std::iter::repeat_n('\\', backslashes));
            quoted.push(ch);
        }
        backslashes = 0;
    }

    quoted.extend(std::iter::repeat_n('\\', backslashes * 2));
    quoted.push('"');
    quoted
}

fn normalize_time(value: &str) -> Result<String> {
    let mut parts = value.split(':');
    let hour = parts.next().unwrap_or_default();
    let minute = parts.next().unwrap_or_default();

    if parts.next().is_some() {
        bail!("invalid time {value:?}, expected HH:MM");
    }

    let hour: u32 = hour
        .parse()
        .with_context(|| format!("invalid hour in {value:?}"))?;
    let minute: u32 = minute
        .parse()
        .with_context(|| format!("invalid minute in {value:?}"))?;

    if hour > 23 || minute > 59 {
        bail!("invalid time {value:?}, expected HH:MM in 24-hour format");
    }

    Ok(format!("{hour:02}:{minute:02}"))
}

fn schtasks_output(args: &[&str]) -> Result<std::process::Output> {
    Command::new("schtasks")
        .args(args)
        .output()
        .context("failed to run schtasks")
}

fn print_schtasks_text(output: &std::process::Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.trim().is_empty() {
        println!("{stdout}");
    }
    if !stderr.trim().is_empty() {
        eprintln!("{stderr}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_time() {
        assert_eq!(normalize_time("9:5").unwrap(), "09:05");
        assert_eq!(normalize_time("09:30").unwrap(), "09:30");
    }

    #[test]
    fn rejects_invalid_time() {
        assert!(normalize_time("24:00").is_err());
        assert!(normalize_time("09:60").is_err());
        assert!(normalize_time("09:30:00").is_err());
    }

    #[test]
    fn quotes_windows_paths_with_trailing_backslashes() {
        assert_eq!(
            quote_arg("C:\\Program Files\\"),
            "\"C:\\Program Files\\\\\""
        );
        assert_eq!(quote_arg("simple"), "simple");
    }
}
