use std::time::Duration;

use anyhow::{Context, Result};

use crate::cli::{VersionCmd, VersionSub};
use crate::{paths, update, versionstore};

pub fn run(cmd: VersionCmd) -> Result<()> {
    match cmd.cmd {
        None => show(),
        Some(VersionSub::List) => list(),
        Some(VersionSub::Use { version }) => use_version(&version),
        Some(VersionSub::Upgrade) => upgrade(),
    }
}

fn show() -> Result<()> {
    println!("pogly-cli v{}", update::CURRENT_VERSION);
    println!("Install dir: {}", paths::local_dir().display());
    if let Some((_, latest)) = update::update_available() {
        println!("A new version is available: v{latest} — run `pogly version upgrade`");
    }
    Ok(())
}

fn list() -> Result<()> {
    let versions = versionstore::installed_versions();
    if versions.is_empty() {
        println!("No versions installed in {}", paths::bin_dir().display());
    } else {
        let current = versionstore::pointer();
        for version in &versions {
            let marker = if Some(version) == current.as_ref() {
                "*"
            } else {
                " "
            };
            println!("{marker} {version}");
        }
    }
    if let Some((_, latest)) = update::update_available() {
        println!("\nLatest release: v{latest} (run `pogly version upgrade`)");
    }
    Ok(())
}

fn use_version(version: &str) -> Result<()> {
    let version = version.trim_start_matches('v');
    if !versionstore::installed_versions()
        .iter()
        .any(|v| v == version)
    {
        println!("v{version} is not installed, downloading...");
        let release = update::fetch_release(version, Duration::from_secs(10))?;
        versionstore::install_version(version, &release)?;
    }
    versionstore::set_pointer(version)?;
    println!("Now using pogly-cli v{version}");
    Ok(())
}

fn upgrade() -> Result<()> {
    let release = update::fetch_release("latest", Duration::from_secs(10))?;
    let latest = update::release_version(&release).context("latest release has no tag")?;
    let up_to_date = matches!(
        (
            semver::Version::parse(&latest),
            semver::Version::parse(update::CURRENT_VERSION)
        ),
        (Ok(l), Ok(c)) if l <= c
    );
    if up_to_date {
        println!("Already up to date (v{})", update::CURRENT_VERSION);
        return Ok(());
    }
    println!("Downloading v{latest}...");
    versionstore::install_version(&latest, &release)?;
    versionstore::set_pointer(&latest)?;
    if let Err(e) = versionstore::replace_launcher(&release) {
        eprintln!("warning: launcher was not updated: {e}");
    }
    println!("Upgraded: v{} -> v{latest}", update::CURRENT_VERSION);
    Ok(())
}
