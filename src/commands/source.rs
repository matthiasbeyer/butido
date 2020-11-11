use std::io::Write;
use std::path::PathBuf;

use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use clap_v3::ArgMatches;

use crate::config::*;
use crate::package::PackageName;
use crate::package::PackageVersionConstraint;
use crate::repository::Repository;
use crate::source::*;

pub async fn source<'a>(matches: &ArgMatches, config: &Configuration<'a>, repo: Repository) -> Result<()> {
    match matches.subcommand() {
        ("verify", Some(matches))       => verify(matches, config, repo).await,
        ("list-missing", Some(matches)) => list_missing(matches, config, repo).await,
        ("url", Some(matches))          => url(matches, config, repo).await,
        (other, _) => return Err(anyhow!("Unknown subcommand: {}", other)),
    }
}

pub async fn verify<'a>(matches: &ArgMatches, config: &Configuration<'a>, repo: Repository) -> Result<()> {
    use tokio::stream::StreamExt;

    let source_cache_root = PathBuf::from(config.source_cache_root());
    let sc                = SourceCache::new(source_cache_root);
    let pname             = matches.value_of("package_name").map(String::from).map(PackageName::from);
    let pvers             = matches.value_of("package_version").map(String::from).map(PackageVersionConstraint::new).transpose()?;

    repo.packages()
        .filter(|p| pname.as_ref().map(|n| p.name() == n).unwrap_or(true))
        .filter(|p| pvers.as_ref().map(|v| v.matches(p.version())).unwrap_or(true))
        .map(|p| {
            let source = sc.source_for(p);
            async move {
                let out = std::io::stdout();
                if source.exists() {
                    if source.verify_hash().await? {
                        writeln!(out.lock(), "Ok: {}", source.path().display())?;
                    } else {
                        writeln!(out.lock(), "Hash Mismatch: {}", source.path().display())?;
                    }
                } else {
                    writeln!(out.lock(), "Source missing: {}", source.path().display())?;
                }

                Ok(())
            }
        })
        .collect::<futures::stream::FuturesUnordered<_>>()
        .collect::<Result<()>>()
        .await
}

pub async fn list_missing<'a>(_: &ArgMatches, config: &Configuration<'a>, repo: Repository) -> Result<()> {
    let sc          = SourceCache::new(PathBuf::from(config.source_cache_root()));
    let out         = std::io::stdout();
    let mut outlock = out.lock();

    repo.packages()
        .map(|p| {
            let s = sc.source_for(p);
            if !s.exists() {
                writeln!(outlock, "{} {} -> {}", p.name(), p.version(), s.path().display())?;
            }

            Ok(())
        })
        .collect()
}

pub async fn url<'a>(matches: &ArgMatches, config: &Configuration<'a>, repo: Repository) -> Result<()> {
    let out         = std::io::stdout();
    let mut outlock = out.lock();

    let pname = matches.value_of("package_name").map(String::from).map(PackageName::from);
    let pvers = matches.value_of("package_version").map(String::from).map(PackageVersionConstraint::new).transpose()?;

    repo.packages()
        .filter(|p| pname.as_ref().map(|n| p.name() == n).unwrap_or(true))
        .filter(|p| pvers.as_ref().map(|v| v.matches(p.version())).unwrap_or(true))
        .map(|p| writeln!(outlock, "{} {} -> {}", p.name(), p.version(), p.source().url()).map_err(Error::from))
        .collect()
}

