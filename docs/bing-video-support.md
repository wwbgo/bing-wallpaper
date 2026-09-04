# Bing 视频/动态壁纸支持调研

日期：2026-09-04

## 结论

当前项目使用的 Bing 公开接口 `HPImageArchive.aspx` 不提供稳定可用的视频或动态壁纸 URL，因此本项目继续只支持静态图片。

## 已验证的请求

```text
https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=zh-CN&video=1
```

返回的 `images[0]` 当前字段包括：

- `startdate`
- `fullstartdate`
- `enddate`
- `url`
- `urlbase`
- `copyright`
- `copyrightlink`
- `title`
- `quiz`
- `wp`
- `hsh`
- `drk`
- `top`
- `bot`
- `hs`

其中没有 `video`、`videos`、`mp4` 或类似视频资源字段。

## `video=1` 的实际效果

在请求中附加 `video=1` 后，当前返回的 `tooltips` 会增加：

```json
"play": "Play video",
"pause": "Pause video"
```

但图片对象仍不包含视频 URL。该参数不能保证返回视频资源。

## 首页现状

当前 Bing 首页 HTML 中存在 `bg_video` 相关 CSS，说明首页播放器具备视频背景能力；但当前图片数据中没有公开、稳定的视频 URL 可供外部工具直接下载。

## 历史情况

Bing 首页曾在 2011 年引入 HTML5 视频背景，后续也偶尔出现视频背景；但 HPImageArchive 长期没有形成稳定的视频字段约定。因此不能作为本工具的视频数据源。

## 项目处理

- 不实现视频壁纸。
- 保留静态图片下载和设置壁纸能力。
- 如果未来 Bing 在 HPImageArchive 中稳定返回视频字段，再重新评估。

## 参考来源

- Bing Image Archive now with HTML5 video support: <https://istartedsomething.com/20111011/bing-image-archive-now-with-html5-video-support/>
- Is there a way to get Bing's photo of the day: <https://stackoverflow.com/questions/10639914/is-there-a-way-to-get-bings-photo-of-the-day>
- Microsoft Learn Bing HPImageArchive discussion: <https://learn.microsoft.com/en-us/answers/questions/1473961/bing-hpimagearchive-aspx-question>
