//! Filesystem locations for alfredrs state.

use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("ALFREDRS_DATA_DIR") {
        return PathBuf::from(custom);
    }
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("alfredrs")
}

pub fn ensure_data_dir() -> anyhow::Result<PathBuf> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn respects_env_override() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("ALFREDRS_DATA_DIR", dir.path());
        assert_eq!(data_dir(), dir.path());
        std::env::remove_var("ALFREDRS_DATA_DIR");
    }
}
