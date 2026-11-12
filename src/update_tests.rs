use super::*;
use crate::bing::BingImage;
use crate::cli::WallpaperOptions;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

struct Fixture(AppDirs);

impl Fixture {
    fn new() -> Self {
        let root = env::temp_dir().join(format!(
            "bing-update-test-{}-{}",
            std::process::id(),
            NEXT_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        let dirs = AppDirs {
            images: root.join("images"),
            state_file: root.join("state.json"),
            logs: root.join("logs"),
            root,
        };
        dirs.ensure().unwrap();
        Self(dirs)
    }

    fn seed(&self, date: &str) -> PathBuf {
        let image = self.0.images.join(format!("{date}.jpg"));
        fs::write(&image, "1080").unwrap();
        AppState {
            last_startdate: date.to_string(),
            last_image: image.clone(),
            last_local_run_date: local_date_string(),
            last_resolution: Some("1080".into()),
            lock_screen_synced: true,
            ..AppState::default()
        }
        .save(&self.0.state_file)
        .unwrap();
        image
    }

    fn state(&self) -> AppState {
        AppState::load(&self.0.state_file).unwrap().unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0.root);
    }
}

fn args(resolution: Resolution) -> RunArgs {
    RunArgs {
        wallpaper: WallpaperOptions {
            mkt: "test".into(),
            host: "fixture".into(),
            resolution,
            style: WallpaperStyle::Fill,
            keep: 1,
        },
        force: false,
        dry_run: false,
    }
}

fn image(date: &str) -> BingImage {
    BingImage {
        startdate: date.into(),
        urlbase: "/fixture".into(),
        copyright: "fixture".into(),
    }
}

// Replace only network and OS changes; run the actual update, persistence, and pruning logic.
fn download(_: &str, _: &BingImage, resolution: Resolution, path: &Path) -> Result<()> {
    fs::write(path, resolution_name(resolution))?;
    Ok(())
}

#[test]
fn retry_restores_requested_resolution_after_lock_screen_failure() {
    let fixture = Fixture::new();
    let today = local_date_string();
    let cached = fixture.seed(&today);
    let result = run_once_with(
        &fixture.0,
        &args(Resolution::Uhd),
        |_, _| Ok(image(&today)),
        download,
        |_, _| anyhow::bail!("lock screen denied"),
    );
    assert!(result.is_err());
    assert_eq!(fs::read_to_string(&cached).unwrap(), "uhd");
    assert!(!fixture.state().lock_screen_synced);

    run_once_with(
        &fixture.0,
        &args(Resolution::Fhd),
        |_, _| Ok(image(&today)),
        download,
        |_, _| Ok(()),
    )
    .unwrap();
    assert_eq!(fs::read_to_string(&cached).unwrap(), "1080");
    assert_eq!(fixture.state().last_resolution.as_deref(), Some("1080"));
    assert!(fixture.state().lock_screen_synced);
}

#[test]
fn lock_screen_failure_prunes_images_but_preserves_last_successful_image() {
    let fixture = Fixture::new();
    let previous = fixture.seed("20000101");
    for day in 2..=20 {
        fs::write(fixture.0.images.join(format!("200001{day:02}.jpg")), "old").unwrap();
    }
    let result = run_once_with(
        &fixture.0,
        &args(Resolution::Uhd),
        |_, _| Ok(image("20000201")),
        download,
        |_, _| anyhow::bail!("lock screen denied"),
    );
    assert_eq!(result.unwrap_err().to_string(), "lock screen denied");
    assert!(previous.is_file());
    assert!(fixture.0.images.join("20000201.jpg").is_file());
    assert_eq!(fs::read_dir(&fixture.0.images).unwrap().count(), 2);
    assert!(!fixture.state().lock_screen_synced);
    assert!(
        fs::read_to_string(fixture.0.logs.join("worker.log"))
            .unwrap()
            .contains("lock screen denied")
    );
}

#[test]
fn download_failure_also_prunes_images_without_removing_last_success() {
    let fixture = Fixture::new();
    let previous = fixture.seed("20000101");
    for day in 2..=20 {
        fs::write(fixture.0.images.join(format!("200001{day:02}.jpg")), "old").unwrap();
    }
    let result = run_once_with(
        &fixture.0,
        &args(Resolution::Uhd),
        |_, _| Ok(image("20000201")),
        |_, _, _, _| anyhow::bail!("download failed"),
        |_, _| panic!("must not apply an image after a failed download"),
    );
    assert_eq!(result.unwrap_err().to_string(), "download failed");
    assert!(previous.is_file());
    assert!(fs::read_dir(&fixture.0.images).unwrap().count() <= 2);
}

#[test]
fn successful_sync_commits_state_and_next_run_skips_network_and_os() {
    let fixture = Fixture::new();
    let today = local_date_string();
    let result = run_once_with(
        &fixture.0,
        &args(Resolution::Fhd),
        |_, _| Ok(image(&today)),
        download,
        |path, _| {
            assert_eq!(fs::read_to_string(path)?, "1080");
            Ok(())
        },
    )
    .unwrap();
    assert!(result.changed);
    assert!(fixture.state().lock_screen_synced);
    let result = run_once_with(
        &fixture.0,
        &args(Resolution::Fhd),
        |_, _| panic!("current image should skip metadata"),
        |_, _, _, _| panic!("current image should skip downloading"),
        |_, _| panic!("current image should skip OS changes"),
    )
    .unwrap();
    assert!(!result.changed);
}

#[test]
fn migrated_cached_image_retries_without_downloading_again() {
    let fixture = Fixture::new();
    let today = local_date_string();
    let cached = fixture.seed(&today);
    let mut state = fixture.state();
    state.lock_screen_synced = false;
    state.save(&fixture.0.state_file).unwrap();

    let failure = run_once_with(
        &fixture.0,
        &args(Resolution::Fhd),
        |_, _| Ok(image(&today)),
        |_, _, _, _| panic!("matching cached image should be reused"),
        |_, _| anyhow::bail!("lock screen denied"),
    );
    assert!(failure.is_err());
    assert_eq!(fixture.state().last_resolution.as_deref(), Some("1080"));
    run_once_with(
        &fixture.0,
        &args(Resolution::Fhd),
        |_, _| Ok(image(&today)),
        |_, _, _, _| panic!("retry should reuse the unchanged cached image"),
        |path, _| {
            assert_eq!(path, cached);
            Ok(())
        },
    )
    .unwrap();
    assert!(fixture.state().lock_screen_synced);
}

#[test]
fn dry_run_preserves_state_and_cached_image() {
    let fixture = Fixture::new();
    let today = local_date_string();
    let cached = fixture.seed(&today);
    let original_state = fs::read(&fixture.0.state_file).unwrap();
    let mut args = args(Resolution::Uhd);
    args.dry_run = true;
    run_once_with(
        &fixture.0,
        &args,
        |_, _| Ok(image(&today)),
        |_, _, _, _| panic!("dry run must not download"),
        |_, _| panic!("dry run must not apply images"),
    )
    .unwrap();
    assert_eq!(fs::read(&fixture.0.state_file).unwrap(), original_state);
    assert_eq!(fs::read_to_string(cached).unwrap(), "1080");
}
