# 自动分辨率与 Bing 视频支持调研实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 实现 `auto` 分辨率选择，并记录 Bing 视频/动态壁纸调研结论。

**架构：** 将 Windows 显示器尺寸检测放在独立平台模块中，CLI 和运行流程通过纯函数解析 `Resolution::Auto`。状态文件增加实际下载分辨率，避免缓存跨分辨率复用。

**技术栈：** Rust、clap、windows crate、serde、ureq。

---

### 任务 1：新增 `Auto` 分辨率和显示器检测

**文件：**
- 创建：`src/display.rs`
- 修改：`src/lib.rs`
- 修改：`src/cli.rs`
- 测试：`src/display.rs`

- [ ] **步骤 1：新增 `Resolution::Auto` 并默认使用**

在 `src/cli.rs` 中：

```rust
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Resolution {
    #[value(name = "auto")]
    Auto,
    #[value(name = "1080", alias = "fhd")]
    Fhd,
    #[value(name = "uhd", alias = "4k")]
    Uhd,
}
```

将 `WallpaperOptions.resolution` 的 `default_value` 从 `"1080"` 改为 `"auto"`。

- [ ] **步骤 2：创建显示器检测模块**

在 `src/display.rs` 中定义纯策略函数：

```rust
use crate::cli::Resolution;

pub fn choose_auto_resolution(width: i32, height: i32) -> Resolution {
    if width >= 2560 || height >= 1440 {
        Resolution::Uhd
    } else {
        Resolution::Fhd
    }
}
```

再实现 Windows 枚举函数，使用 `EnumDisplayMonitorsW` 和 `GetMonitorInfoW` 获取每台显示器的 `rcMonitor` 尺寸，返回最大宽度和最大高度。

- [ ] **步骤 3：在 `lib.rs` 注册模块**

```rust
pub mod display;
```

- [ ] **步骤 4：编写阈值测试**

在 `src/display.rs` 添加：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_uses_uhd_for_qhd_and_above() {
        assert_eq!(choose_auto_resolution(2560, 1440), Resolution::Uhd);
        assert_eq!(choose_auto_resolution(1920, 1440), Resolution::Uhd);
        assert_eq!(choose_auto_resolution(2560, 1080), Resolution::Uhd);
    }

    #[test]
    fn auto_uses_fhd_below_qhd() {
        assert_eq!(choose_auto_resolution(1920, 1080), Resolution::Fhd);
        assert_eq!(choose_auto_resolution(1366, 768), Resolution::Fhd);
    }
}
```

- [ ] **步骤 5：运行测试验证失败**

运行：`cargo test display::tests --lib`
预期：编译错误，因为 `src/display.rs` 尚未完整实现。

- [ ] **步骤 6：补齐实现并运行测试**

运行：`cargo test display::tests --lib`
预期：通过。

- [ ] **步骤 7：Commit**

```bash
git add src/display.rs src/lib.rs src/cli.rs
git commit -m "feat: add auto resolution and display detection"
```

---

### 任务 2：将实际分辨率接入下载流程和状态

**文件：**
- 修改：`src/lib.rs`
- 修改：`src/state.rs`
- 修改：`src/scheduler.rs`

- [ ] **步骤 1：扩展状态字段**

在 `src/state.rs` 的 `AppState` 中增加：

```rust
#[serde(default)]
pub last_resolution: Option<String>,
```

默认值为 `None`。

- [ ] **步骤 2：解析实际分辨率**

在 `src/lib.rs` 增加：

```rust
fn effective_resolution(resolution: Resolution) -> Result<Resolution> {
    match resolution {
        Resolution::Auto => display::auto_resolution(),
        explicit => Ok(explicit),
    }
}
```

- [ ] **步骤 3：调整 `run_once` 缓存判断和状态保存**

实际分辨率解析后，与 `state.last_resolution.as_deref()` 比较。只有当天已运行、图片存在且分辨率一致时才短路。下载和保存状态使用 `effective` 而不是原始 `args.wallpaper.resolution`。

- [ ] **步骤 4：更新 worker 命令映射**

在 `src/scheduler.rs` 的 `resolution_name` 中增加：

```rust
crate::cli::Resolution::Auto => "auto",
```

- [ ] **步骤 5：运行测试**

运行：`cargo test --all-targets`
预期：现有测试全部通过，无编译错误。

- [ ] **步骤 6：Commit**

```bash
git add src/lib.rs src/state.rs src/scheduler.rs
git commit -m "feat: persist effective wallpaper resolution"
```

---

### 任务 3：记录 Bing 视频调研结论

**文件：**
- 创建：`docs/bing-video-support.md`
- 修改：`TODO.md`
- 修改：`README.md`

- [ ] **步骤 1：创建调研文档**

写入请求样例、响应字段、结论和参考来源。

- [ ] **步骤 2：更新 TODO**

将第 2 项改为已完成结论，保留第 1 项结果和其他未实现项。

- [ ] **步骤 3：更新 README 限制说明**

在功能列表后增加简短限制段落：只下载静态图片，不处理视频或动态壁纸。

- [ ] **步骤 4：运行文档格式检查**

运行：`git diff --check`
预期：无 whitespace errors。

- [ ] **步骤 5：Commit**

```bash
git add docs/bing-video-support.md TODO.md README.md
git commit -m "docs: record Bing video support research"
```

---

### 最终验证

- [ ] 运行 `cargo fmt --check`
- [ ] 运行 `cargo test --all-targets`
