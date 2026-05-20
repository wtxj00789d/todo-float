use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const DEFAULT_CONFIG_TOML: &str = r#"[llm]
provider = "openrouter"
api_key = ""
model = "z-ai/glm-4.5-air"

[llm.fallback]
provider = "bigmodel"
api_key = ""
model = "glm-4.7-flash"
"#;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct LlmProviderConfig {
    pub provider: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct LlmConfig {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub fallback: Option<LlmProviderConfig>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub llm: LlmConfig,
}

pub fn config_path() -> Result<PathBuf, String> {
    if let Ok(path) = std::env::var("TODO_FLOAT_CONFIG") {
        return Ok(PathBuf::from(path));
    }
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let dir = exe
        .parent()
        .ok_or_else(|| "Cannot find executable directory".to_string())?;
    Ok(dir.join("config.toml"))
}

pub fn load_config_from(path: &Path) -> Result<AppConfig, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|err| format!("无法读取配置文件 {}: {}", path.display(), err))?;
    toml::from_str::<AppConfig>(&text)
        .map_err(|_| format!("配置文件格式错误 {}", path.display()))
}

pub fn ensure_config_file() -> Result<PathBuf, String> {
    let path = config_path()?;
    if path.exists() {
        return Ok(path);
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("无法创建配置目录 {}: {}", parent.display(), err))?;
    }

    std::fs::write(&path, DEFAULT_CONFIG_TOML)
        .map_err(|err| format!("无法创建配置文件 {}: {}", path.display(), err))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn loads_toml_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
            [llm]
            provider = "openrouter"
            api_key = "abc"
            model = "z-ai/glm-4.5-air"

            [llm.fallback]
            provider = "bigmodel"
            api_key = "def"
            model = "glm-4.7-flash"
            "#,
        )
        .unwrap();

        let config = load_config_from(&path).unwrap();
        assert_eq!(config.llm.provider, "openrouter");
        assert_eq!(config.llm.fallback.unwrap().model, "glm-4.7-flash");
    }

    #[test]
    fn rejects_missing_api_key_for_primary() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
            [llm]
            provider = "openrouter"
            api_key = ""
            model = "z-ai/glm-4.5-air"
            "#,
        )
        .unwrap();

        let config = load_config_from(&path).unwrap();
        assert_eq!(config.llm.api_key, "");
    }

    #[test]
    fn ensure_config_file_creates_default_without_overwriting() {
        let _guard = ENV_LOCK.lock().unwrap();
        let _env = ConfigEnvGuard::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::env::set_var("TODO_FLOAT_CONFIG", &path);

        let created_path = ensure_config_file().unwrap();

        assert_eq!(created_path, path);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), DEFAULT_CONFIG_TOML);

        std::fs::write(&path, "custom = true\n").unwrap();
        ensure_config_file().unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "custom = true\n");
    }

    #[test]
    fn config_path_uses_env_override() {
        let _guard = ENV_LOCK.lock().unwrap();
        let _env = ConfigEnvGuard::new();
        let override_path = PathBuf::from(r"C:\todo-float\custom-config.toml");
        std::env::set_var("TODO_FLOAT_CONFIG", &override_path);

        let path = config_path().unwrap();

        assert_eq!(path, override_path);
    }

    #[test]
    fn default_config_path_is_absolute_config_toml() {
        let _guard = ENV_LOCK.lock().unwrap();
        let _env = ConfigEnvGuard::new();
        std::env::remove_var("TODO_FLOAT_CONFIG");

        let path = config_path().unwrap();

        assert_eq!(path.file_name().unwrap(), "config.toml");
        assert!(path.is_absolute());
    }

    #[test]
    fn parse_errors_do_not_include_secret_values() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
            [llm]
            provider = "openrouter"
            api_key = "SECRET_SHOULD_NOT_LEAK
            model = "z-ai/glm-4.5-air"
            "#,
        )
        .unwrap();

        let err = load_config_from(&path).unwrap_err();
        assert!(err.contains(&path.display().to_string()));
        assert!(!err.contains("SECRET_SHOULD_NOT_LEAK"));
        assert!(!err.contains("api_key"));
    }

    struct ConfigEnvGuard {
        previous: Option<std::ffi::OsString>,
    }

    impl ConfigEnvGuard {
        fn new() -> Self {
            Self {
                previous: std::env::var_os("TODO_FLOAT_CONFIG"),
            }
        }
    }

    impl Drop for ConfigEnvGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => std::env::set_var("TODO_FLOAT_CONFIG", value),
                None => std::env::remove_var("TODO_FLOAT_CONFIG"),
            }
        }
    }
}
