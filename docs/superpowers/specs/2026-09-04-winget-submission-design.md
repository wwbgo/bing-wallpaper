# WinGet 包提交支持设计

日期：2026-09-04

## 目标

为 `bing-wallpaper` 增加 Windows Package Manager Community Repository 的首次提交清单和发布自动化。

## 包标识

- PackageIdentifier：`wwb.BingWallpaper`
- PackageVersion：随 release tag 去掉 `v` 前缀
- ManifestVersion：`1.12.0`
- 清单路径：`manifests/w/wwb/BingWallpaper/<version>/`

## 安装器信息

- InstallerType：`inno`
- Architecture：`x64`
- Scope：`user`
- InstallModes：`silent`、`silentWithProgress`
- InstallerUrl：GitHub Release 的 `BingWallpaperSetup-<version>.exe`
- InstallerSha256：由 release asset 计算

## 自动化

- `.github/workflows/winget.yml` 监听 `release: published`。
- 下载 release 安装器并计算 SHA256。
- 生成三文件清单并运行 `winget validate`。
- 配置 `WINGET_TOKEN` 时使用 `wingetcreate submit` 创建 PR。
- 未配置 token 时上传清单 artifact 并输出人工提交步骤。

## 首次提交

- 当前仓库提交 `0.2.1` 初始清单作为模板和人工首次提交来源。
- 因为首次包不在 `winget-pkgs` 中，不使用 `winget-releaser` 进行首次创建。

## 文档

- 新增 `docs/winget.md`。
- README 增加安装命令和首次清单合并说明。
