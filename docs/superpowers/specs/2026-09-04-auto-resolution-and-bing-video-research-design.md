# 自动分辨率与 Bing 视频支持调研设计

日期：2026-09-04

## 目标

先实现前两个 TODO：

1. 默认图片分辨率根据 Windows 桌面分辨率自动判断。
2. 调研 Bing 是否提供动态图片或视频链接，并把结论记录到项目中。

## 范围

本次不实现 macOS 支持，不实现 winget 自动化，也不实现视频壁纸。

## 第 1 项：自动分辨率

### CLI 行为

- `Resolution` 枚举新增 `Auto`。
- `--resolution` 默认值从 `1080` 改为 `auto`。
- 显式传入 `1080` 或 `uhd` 时保持现有行为。

### 自动判断规则

- 在 Windows 上枚举所有显示器。
- 任一显示器宽度 `>= 2560` 或高度 `>= 1440` 时，使用 `uhd`。
- 否则使用 `1080`。
- 不使用虚拟桌面总尺寸，避免双 1080p 显示器被误判为高分辨率。

### 缓存一致性

- `AppState` 增加 `last_resolution` 字段，并使用 `serde(default)` 兼容旧状态文件。
- 每次运行解析出实际下载分辨率后，与 `state.last_resolution` 比较。
- 当天图片已缓存但实际分辨率不同时，重新下载并覆盖当前图片。
- 状态保存时写入 `last_resolution`。

### 计划任务一致性

- worker 命令同样支持并传递 `auto`。
- `scheduler::resolution_name` 增加 `Auto => "auto"` 映射。

## 第 2 项：视频/动态壁纸调研

### 调研结论

截至 2026-09-04：

- `HPImageArchive.aspx?format=js&idx=0&n=1&mkt=zh-CN&video=1` 返回的图片对象中没有 `video`、`videos`、`mp4` 等视频 URL 字段。
- `video=1` 只让 `tooltips` 增加 `play`/`pause` 文案，不返回视频资源。
- 当前 Bing 首页 HTML 中只有 `bg_video` 相关 CSS，当前图片数据没有稳定公开的视频 URL。
- Bing 首页历史上曾支持 HTML5 视频背景，但当前公开 HPImageArchive 接口不保证提供视频链接。

### 落地方式

- 新增 `docs/bing-video-support.md`，记录调研结论、请求样例和来源。
- 将 `TODO.md` 第 2 项改为已完成结论，避免继续作为未决待办。
- 在 `README.md` 增加限制说明：当前工具只下载静态图片，不处理视频或动态壁纸。

## 测试

- 增加分辨率选择函数的单元测试，覆盖阈值判断。
- 保留现有 `candidate_image_urls` 测试。
- 运行 `cargo fmt --check`。
- 运行 `cargo test --all-targets`。
