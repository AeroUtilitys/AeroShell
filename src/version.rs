use serde::Deserialize;

#[derive(Deserialize)]
struct VersionConfig {
    aeroshell: AeroShellVersion,
}

#[derive(Deserialize)]
struct AeroShellVersion {
    dev_version: String,
    stable_version: String,
}

pub fn get_version_description() -> String {
    let version_file = include_str!("../version.toml");
    let config: VersionConfig = toml::from_str(version_file).expect("Failed to parse version.toml");

    if cfg!(debug_assertions) {
        format!("v{} (Dev Build)", config.aeroshell.dev_version)
    } else {
        format!("v{} (Stable)", config.aeroshell.stable_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let version_file = include_str!("../version.toml");
        let config: Result<VersionConfig, _> = toml::from_str(version_file);
        assert!(config.is_ok(), "version.toml should be valid TOML");

        let config = config.unwrap();
        assert!(!config.aeroshell.dev_version.is_empty(), "dev_version should not be empty");
        assert!(!config.aeroshell.stable_version.is_empty(), "stable_version should not be empty");
    }
}
