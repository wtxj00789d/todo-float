# Todo Float

Todo Float 是一个轻量级 Windows 桌面待办浮窗。它的目标很窄：开机时检查今天是否有待办，只在需要时弹出一个克制的小窗口；平时可以用自然语言追加事项，例如“明天提醒我拿文件和找凯莉”。

## 用途

- 开机轻量检查当天 todo，避免启动时调用大模型。
- 每个自然日期只弹出一次；跨到第二天后会按新日期重新判断。
- 自然语言输入会解析成事项和日期，当前支持日期级 todo，并保留 `due_time` 字段作为后续时间提醒能力。
- 日期规则按本机日期计算，并采用周一为一周起始、周日为一周结束。
- 主模型为 OpenRouter `z-ai/glm-4.5-air`，备用模型为智谱直连 `glm-4.7-flash`。默认固定这两个模型，是因为它们都有免费可用额度或免费档，注册账号并开通 API 后，把 API Key 填进配置即可使用。

## 安装

### 直接运行

本机完成构建后，可直接运行：

```powershell
src-tauri\target\release\todo-float.exe
```

安装包位置：

```powershell
src-tauri\target\release\bundle\nsis\Todo Float_0.1.0_x64-setup.exe
src-tauri\target\release\bundle\msi\Todo Float_0.1.0_x64_en-US.msi
```

第一次正常打开应用时，它会注册 Windows 开机启动项。开机启动时使用 `--startup-check` 模式，只查本地数据库和当天弹窗状态，不调用大模型。

### 从源码构建

需要 Windows、Node.js、Rust 和 Tauri 依赖环境。

```powershell
npm install
npm.cmd run tauri build
```

开发模式：

```powershell
npm.cmd run tauri dev
```

## 初始化配置

安装后第一次启动会在 `todo-float.exe` 同目录自动创建空的 `config.toml`。如果已有配置，应用不会覆盖。也可以从仓库根目录复制 `config.example.toml`，再填入自己的 API Key：

```toml
[llm]
provider = "openrouter"
api_key = ""
model = "z-ai/glm-4.5-air"

[llm.fallback]
provider = "bigmodel"
api_key = ""
model = "glm-4.7-flash"
```

说明：

- `config.toml` 不会提交到 git。
- 如果主模型 `api_key` 为空，点击“解析”时会提示补配置。
- OpenRouter API Key 教程：[OpenRouter Quickstart](https://openrouter.ai/docs/quickstart)。
- 智谱 API Key 教程：[智谱 AI 使用概述](https://docs.bigmodel.cn/cn/api/introduction)。
- 可以用环境变量 `TODO_FLOAT_CONFIG` 指定配置文件路径。
- 数据库默认放在 exe 同目录，文件名为 `todo-float.sqlite3`；也可以用 `TODO_FLOAT_DB` 指定路径。

## 使用

1. 打开 `todo-float.exe`。
2. 点击“添加”，在自然语言输入框里写下待办。
3. 可点击“语音”调用 Windows 语音输入。
4. 点击“解析”，检查预览里的事项和日期。
5. 点击“确认保存”。
6. 如果今天不想再弹窗，点击“今天不再弹出”。

开机启动时，应用只会读取本地数据库：如果今天没有 todo，或今天已经弹出过且没有新增 todo，就不会显示窗口。

## 工程结构

- `src/`：React 浮窗界面。
- `src-tauri/src/`：Tauri/Rust 后端、SQLite、日期规则、LLM 调用。
- `src-tauri/icons/`：应用图标资源，由 gpt-image-2 生成源图后通过 Tauri icon 管线切出。
- `config.example.toml`：配置模板。
- `mockups/`：早期浏览器 mockup。
- `docs/superpowers/`：设计和实现计划记录。

## 验证

```powershell
npm.cmd test
cd src-tauri
cargo test
```
