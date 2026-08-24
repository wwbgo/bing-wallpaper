use clap::Parser;

use bing_wallpaper::cli::Cli;

fn main() -> anyhow::Result<()> {
    match Cli::parse() {
        Cli::Run(args) => {
            let outcome = bing_wallpaper::run_once(&args)?;
            if args.dry_run {
                println!(
                    "dry run: startdate={} image={} copyright={}",
                    outcome.startdate,
                    outcome.image_path.display(),
                    outcome.copyright
                );
            } else if outcome.changed {
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
        Cli::Install(args) => {
            bing_wallpaper::scheduler::install(&args.wallpaper, &args.time)?;
        }
        Cli::Uninstall => {
            bing_wallpaper::scheduler::uninstall()?;
        }
        Cli::Status => {
            bing_wallpaper::status()?;
        }
    }

    Ok(())
}
