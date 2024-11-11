use crevette::RepoInfo;
use crevette::Crevette;
use crevette::Error;
use std::error::Error as _;
use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        let mut source = e.source();
        while let Some(e) = source {
            eprintln!("  {e}");
            source = e.source();
        }
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run() -> Result<(), Error> {
    #[allow(unused_mut)]
    let mut actions: Vec<fn(&Crevette) -> Result<RepoInfo, Error>> = Vec::new();
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--help" | "-h" | "--version" | "-V" => {
                show_help();
                return Ok(())
            },
            "--debcargo" => {
                if !cfg!(feature = "debcargo") {
                    eprintln!("Reinstall with debcargo enabled:\ncargo install crevette --features=debcargo");
                    return Err(Error::UnsupportedVersion(0));
                }
                #[cfg(feature = "debcargo")]
                {
                    actions.push(|c| {
                        let dirs = directories_next::BaseDirs::new().unwrap();
                        let cache_dir = dirs.cache_dir().join("crevette");
                        c.convert_debcargo_repo(&cache_dir)
                    });
                }
            },
            "--guix" => {
                if !cfg!(feature = "guix") {
                    eprintln!("Reinstall with guix enabled:\ncargo install crevette --features=guix");
                    return Err(Error::UnsupportedVersion(0));
                }
                #[cfg(feature = "guix")]
                {
                    actions.push(|c| {
                        let dirs = directories_next::BaseDirs::new().unwrap();
                        let cache_dir = dirs.cache_dir().join("crevette");
                        c.convert_guix_repo(&cache_dir)
                    });
                }
            },
            other => {
                eprintln!("unknown argument: {other}");
            },
        }
    }
    if actions.is_empty() {
        actions.push(|c| c.convert_into_repo());
    }

    let c = Crevette::new()?;
    for a in &actions {
        let res = a(&c)?;
        println!(
            "Wrote '{}'\nRun `cargo crev publish` to upload the file to {}\nThen run `cargo vet import yourname {}`\n",
            res.local_path.display(),
            res.repo_git_url.as_deref().unwrap_or("your git repo (not configured yet?)"),
            res.repo_https_url.as_deref().unwrap_or("https://<your repo URL>/audits.toml"),
        );
    }
    Ok(())
}

fn show_help() {
    eprintln!("https://lib.rs/crevette {}
    Run without args to update your crev repo.
    Run with --debcargo to make a vet file from Debian package list.
    Run with --guix to make a vet file from Debian package list.
    ", env!("CARGO_PKG_VERSION"));
}
