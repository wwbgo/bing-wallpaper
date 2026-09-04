# Bing Wallpaper

一个轻量级 Windows 必应每日壁纸工具，使用 Rust 编写。

设计目标是低资源占用：日常由 Windows 任务计划程序运行，下载并设置壁纸后立即退出，不常驻后台。

## 功能

- 从必应官方公开接口获取每日壁纸
- 支持 `auto`、`1080` 和 `uhd` 三种画质
- `auto` 会枚举显示器，在任一显示器达到 2560x1440 或以上时使用 `uhd`
- 支持填充、适应、拉伸、居中、平铺、跨屏
- 本地缓存，按日期幂等更新
- 自动清理旧壁纸
- 注册每日计划任务和当前用户登录自启
- 支持 `setup` 命令完成“立即更新 + 自动配置”
- 提供 Windows x64 安装包
- 独立 worker 二进制，无控制台窗口
- 当前只下载静态图片，不处理视频或动态壁纸

## 构建

```powershell
cargo build --release --bins
```

输出：

```text
target\release\bing-wallpaper.exe
target\release\bing-wallpaper-worker.exe
```

## 安装包

GitHub Release 会提供：

```text
BingWallpaperSetup-<version>.exe
bing-wallpaper-windows-x64-<version>.zip
```

安装包会安装到当前用户目录：

```text
%LOCALAPPDATA%\Programs\Bing Wallpaper
```

安装完成后会自动执行：

```text
setup --time 09:30 --resolution auto --style fill
```

即立即更新一次壁纸，并注册每日计划任务和登录自启。

## 使用

```powershell
# 立即更新
.\bing-wallpaper.exe run

# 使用 UHD 图片
.\bing-wallpaper.exe run --resolution uhd

# 安装每日任务和登录自启，每天 09:30 运行
.\bing-wallpaper.exe install --time 09:30

# 立即更新并安装每日任务和登录自启
.\bing-wallpaper.exe setup --time 09:30 --resolution auto --style fill

# 查看缓存和任务状态
.\bing-wallpaper.exe status

# 卸载计划任务和登录自启
.\bing-wallpaper.exe uninstall
```

## 数据目录

```text
%LOCALAPPDATA%\BingWallpaper\
  images\
  logs\
  state.json
```

## 命令参数

顶层用法：

```text
bing-wallpaper.exe [COMMAND]
```

| 命令 | 说明 |
| --- | --- |
| `run` | 立即更新壁纸 |
| `install` | 注册每日计划任务和登录自启 |
| `setup` | 立即更新，然后注册每日计划任务和登录自启 |
| `uninstall` | 卸载每日计划任务和登录自启 |
| `status` | 查看缓存和计划任务状态 |
| `help` | 打印指定命令的帮助 |

顶层参数：

| 参数 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `-h, --help` | 开关 | 无 | 打印顶层帮助 |

### `run`

立即更新壁纸。

```text
bing-wallpaper.exe run [OPTIONS]
```

| 参数 | 类型 | 默认值 | 可选值 | 说明 |
| --- | --- | --- | --- | --- |
| `--mkt <MKT>` | 字符串 | `zh-CN` | 任意 Bing market | 必应地区，例如 `zh-CN`、`en-US` |
| `--host <HOST>` | 字符串 | `https://www.bing.com` | URL | 必应接口主机 |
| `--resolution <RESOLUTION>` | 枚举 | `auto` | `auto`、`1080`、`uhd` | 首选图片质量；`auto` 根据显示器分辨率选择 |
| `--style <STYLE>` | 枚举 | `fill` | `fill`、`fit`、`stretch`、`center`、`span`、`tile` | Windows 壁纸放置模式 |
| `--keep <KEEP>` | 整数 | `14` | 正整数 | 保留最近多少张缓存图 |
| `--force` | 开关 | 关闭 | 无 | 即使当天图片已缓存也强制更新 |
| `--dry-run` | 开关 | 关闭 | 无 | 只获取元数据，不下载或设置壁纸 |
| `-h, --help` | 开关 | 无 | 无 | 显示当前命令帮助 |

### `install`

注册每日计划任务和当前用户登录自启。

```text
bing-wallpaper.exe install [OPTIONS]
```

| 参数 | 类型 | 默认值 | 可选值 | 说明 |
| --- | --- | --- | --- | --- |
| `--mkt <MKT>` | 字符串 | `zh-CN` | 任意 Bing market | 必应地区，例如 `zh-CN`、`en-US` |
| `--host <HOST>` | 字符串 | `https://www.bing.com` | URL | 必应接口主机 |
| `--resolution <RESOLUTION>` | 枚举 | `auto` | `auto`、`1080`、`uhd` | 首选图片质量；`auto` 根据显示器分辨率选择 |
| `--style <STYLE>` | 枚举 | `fill` | `fill`、`fit`、`stretch`、`center`、`span`、`tile` | Windows 壁纸放置模式 |
| `--keep <KEEP>` | 整数 | `14` | 正整数 | 保留最近多少张缓存图 |
| `--time <TIME>` | 字符串 | `09:30` | `HH:MM`，24 小时制 | 每日计划任务运行时间 |
| `-h, --help` | 开关 | 无 | 无 | 显示当前命令帮助 |

### `setup`

立即更新壁纸，然后注册每日计划任务和登录自启。

```text
bing-wallpaper.exe setup [OPTIONS]
```

| 参数 | 类型 | 默认值 | 可选值 | 说明 |
| --- | --- | --- | --- | --- |
| `--mkt <MKT>` | 字符串 | `zh-CN` | 任意 Bing market | 必应地区，例如 `zh-CN`、`en-US` |
| `--host <HOST>` | 字符串 | `https://www.bing.com` | URL | 必应接口主机 |
| `--resolution <RESOLUTION>` | 枚举 | `auto` | `auto`、`1080`、`uhd` | 首选图片质量；`auto` 根据显示器分辨率选择 |
| `--style <STYLE>` | 枚举 | `fill` | `fill`、`fit`、`stretch`、`center`、`span`、`tile` | Windows 壁纸放置模式 |
| `--keep <KEEP>` | 整数 | `14` | 正整数 | 保留最近多少张缓存图 |
| `--time <TIME>` | 字符串 | `09:30` | `HH:MM`，24 小时制 | 每日计划任务运行时间 |
| `-h, --help` | 开关 | 无 | 无 | 显示当前命令帮助 |

### `status`

查看缓存、状态文件和计划任务状态。

```text
bing-wallpaper.exe status
```

| 参数 | 类型 | 默认值 | 可选值 | 说明 |
| --- | --- | --- | --- | --- |
| `-h, --help` | 开关 | 无 | 无 | 显示当前命令帮助 |

### `uninstall`

卸载每日计划任务和登录自启。

```text
bing-wallpaper.exe uninstall
```

| 参数 | 类型 | 默认值 | 可选值 | 说明 |
| --- | --- | --- | --- | --- |
| `-h, --help` | 开关 | 无 | 无 | 显示当前命令帮助 |
