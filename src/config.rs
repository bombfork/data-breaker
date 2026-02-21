use std::path::PathBuf;

use directories::ProjectDirs;

pub const REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/bombfork/data-breaker-registry/main/brokers.json";

pub fn project_dirs() -> anyhow::Result<ProjectDirs> {
    ProjectDirs::from("", "bombfork", "data-breaker")
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))
}

pub fn db_path() -> anyhow::Result<PathBuf> {
    let dirs = project_dirs()?;
    let data_dir = dirs.data_dir();
    std::fs::create_dir_all(data_dir)?;
    Ok(data_dir.join("data-breaker.db"))
}
