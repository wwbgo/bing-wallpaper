# WinGet 提交说明

PackageIdentifier: `wwb.BingWallpaper`

当前清单使用 schema `1.12.0`，与 `microsoft/winget-pkgs` 的 PR 模板一致。

## 首次提交

1. Fork `microsoft/winget-pkgs`。
2. 将本仓库 `winget/manifests/w/wwb/BingWallpaper/0.2.1/` 下的三个文件复制到 fork 的对应目录：
   `manifests/w/wwb/BingWallpaper/0.2.1/`
3. 在本地验证：
   `winget validate --manifest manifests/w/wwb/BingWallpaper/0.2.1`
4. 提交 PR，标题建议：
   `New package: wwb.BingWallpaper version 0.2.1`

## 自动提交

1. 创建 classic PAT，scope 至少为 `public_repo`。
2. 在本仓库配置 secret：`WINGET_TOKEN`。
3. 发布 release 后，`.github/workflows/winget.yml` 会自动：
   - 下载安装器。
   - 计算 SHA256。
   - 生成 1.12 清单。
   - 使用 `winget validate` 验证。
   - 在配置 token 的情况下调用 `wingetcreate submit` 创建 PR。
4. 未配置 token 时，workflow 仍会上传 manifest artifact，可下载后按“首次提交”步骤人工提交。

## 自动更新后续版本

当首个 `wwb.BingWallpaper` 清单合入 `winget-pkgs` 后，同一 workflow 的生成结果会包含新版本清单。`wingetcreate submit` 可用于后续更新；如果之后希望使用专门的更新 action，也可以再接入 `winget-releaser`。

## 安装命令

首次清单合入后：

```powershell
winget install wwb.BingWallpaper
```
