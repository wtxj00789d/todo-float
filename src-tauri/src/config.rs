use serde::Deserialize;
use std::path::{Path, PathBuf};

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
        .map_err(|err| format!("配置文件格式错误 {}: {}", path.display(), err))
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
