use std::path::Path;

use anyhow::anyhow;
use anyhow::Result;
use clap::ArgMatches;

use crate::config::*;
use crate::repository::Repository;
use crate::package::PackageName;
use crate::package::PackageVersionConstraint;
use crate::util::progress::ProgressBars;

pub async fn lint(repo_path: &Path, matches: &ArgMatches, progressbars: ProgressBars, config: &Configuration, repo: Repository) -> Result<()> {
    let linter = crate::ui::find_linter_command(repo_path, config)?
        .ok_or_else(|| anyhow!("No linter command found"))?;
    let pname = matches.value_of("package_name").map(String::from).map(PackageName::from);
    let pvers = matches.value_of("package_version").map(String::from).map(PackageVersionConstraint::new).transpose()?;

    let bar = progressbars.bar();
    bar.set_message("Linting package scripts...");

    let iter = repo.packages()
        .filter(|p| pname.as_ref().map(|n| p.name() == n).unwrap_or(true))
        .filter(|p| pvers.as_ref().map(|v| v.matches(p.version())).unwrap_or(true));

    crate::commands::util::lint_packages(iter, &linter, config, bar).await
}
