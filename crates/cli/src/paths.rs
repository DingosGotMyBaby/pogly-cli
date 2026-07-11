use std::path::PathBuf;

fn env_dir(var: &str) -> PathBuf {
    std::env::var_os(var)
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

pub fn config_dir() -> PathBuf {
    env_dir("APPDATA").join("Pogly").join("cli")
}

pub fn local_dir() -> PathBuf {
    env_dir("LOCALAPPDATA").join("Pogly").join("cli")
}

pub fn config_file() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn state_file() -> PathBuf {
    local_dir().join("state.toml")
}

pub fn version_pointer() -> PathBuf {
    local_dir().join("version")
}

pub fn bin_dir() -> PathBuf {
    local_dir().join("bin")
}

pub fn launcher_path() -> PathBuf {
    local_dir().join("pogly.exe")
}
