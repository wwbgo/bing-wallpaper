use clap::{Args, Parser, ValueEnum};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Resolution {
    #[value(name = "1080", alias = "fhd")]
    Fhd,
    #[value(name = "uhd", alias = "4k")]
    Uhd,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum WallpaperStyle {
    Fill,
    Fit,
    Stretch,
    Center,
    Span,
    Tile,
}

#[derive(Clone, Debug, Args)]
pub struct WallpaperOptions {
    /// Bing market, for example zh-CN or en-US.
    #[arg(long, default_value = "zh-CN")]
    pub mkt: String,

    /// Bing host, defaults to the global endpoint.
    #[arg(long, default_value = "https://www.bing.com")]
    pub host: String,

    /// Preferred image quality.
    #[arg(long, value_enum, default_value = "1080")]
    pub resolution: Resolution,

    /// Windows wallpaper placement mode.
    #[arg(long, value_enum, default_value = "fill")]
    pub style: WallpaperStyle,

    /// Number of cached images to keep.
    #[arg(long, default_value_t = 14)]
    pub keep: usize,
}

#[derive(Clone, Debug, Args)]
pub struct RunArgs {
    #[command(flatten)]
    pub wallpaper: WallpaperOptions,

    /// Update even if today's image is already cached.
    #[arg(long)]
    pub force: bool,

    /// Fetch metadata and print the plan without changing the wallpaper.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Clone, Debug, Args)]
pub struct InstallArgs {
    #[command(flatten)]
    pub wallpaper: WallpaperOptions,

    /// Local time when the daily scheduled task runs.
    #[arg(long, default_value = "09:30")]
    pub time: String,
}

#[derive(Debug, Parser)]
#[command(
    name = "bing-wallpaper",
    about = "Lightweight daily Bing wallpaper updater"
)]
pub enum Cli {
    /// Update the wallpaper now.
    Run(RunArgs),

    /// Register daily and logon scheduled tasks.
    Install(InstallArgs),

    /// Remove scheduled tasks.
    Uninstall,

    /// Show cache, state, and scheduled task status.
    Status,
}

#[derive(Debug, Parser)]
pub struct WorkerCli {
    #[command(flatten)]
    pub wallpaper: WallpaperOptions,
}
