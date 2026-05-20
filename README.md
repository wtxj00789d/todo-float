# Todo Float

Todo Float 是一个轻量级 Windows 待办浮窗。它会在开机时检查“今天要做”的事项，只在今天有待办、且今天还没提醒过时弹出。

你也可以直接用自然语言添加待办，例如：

```text
明天提醒我拿文件和找凯莉
```

应用会尽量拆成：

```text
1. 拿文件
2. 找凯莉
```

## 安装

下载并运行 Release 里的安装包：

[Todo Float v0.1.0](https://github.com/wtxj00789d/todo-float/releases/tag/v0.1.0)

推荐下载：

```text
Todo.Float_0.1.0_x64-setup.exe
```

安装完成后，第一次打开应用时会自动注册 Windows 开机启动。之后开机时应用只做本地检查，不会在开机阶段调用大模型。

## 初始化配置

第一次启动时，应用会在 `todo-float.exe` 同目录自动创建一个空的 `config.toml`。如果文件已经存在，应用不会覆盖它。

打开 `config.toml`，填入自己的 API Key：

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

需要填写的位置是两个 `api_key = ""`。

获取 API Key：

- OpenRouter： [OpenRouter Quickstart](https://openrouter.ai/docs/quickstart)
- 智谱 AI： [智谱 AI 使用概述](https://docs.bigmodel.cn/cn/api/introduction)

如果暂时只想用一个模型，也可以只填 OpenRouter 的 `api_key`；智谱的备用 key 可以先留空。

## 怎么用

1. 打开 Todo Float。
2. 点击“添加”。
3. 在输入框里写自然语言待办，例如：

```text
今天提醒我填写出行记录去催雇主信
```

4. 点击“解析”。
5. 检查预览里的事项和日期。如果不满意，可以直接在预览里改文字或日期。
6. 点击“确认保存”。

也可以点击“语音”按钮，调用 Windows 自带语音输入。

如果今天不想再弹窗，点击“今天不再弹出”。这个操作只影响当天；第二天如果有当天待办，仍会重新判断是否弹出。

## 开机弹窗规则

- 今天没有待办：不弹出。
- 今天已经弹出过，且没有新增今天的待办：不再弹出。
- 今天已经点过“今天不再弹出”：当天不再弹出。
- 第二天会重新按新日期判断。

日期按电脑本机日期计算；周一是一周开始，周日是一周结束。

## 为什么默认是这两个 LLM

默认使用：

- OpenRouter `z-ai/glm-4.5-air`
- 智谱直连 `glm-4.7-flash`

原因很朴素：它们目前有免费可用额度或免费档，注册账号并开通 API 后就可以填 key 使用。

这不是广告，也没有赞助关系。只是为了让这个小工具尽量低成本、容易跑起来。

