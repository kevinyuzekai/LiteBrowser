# LiteBrowser 轻量浏览器

超轻量的桌面网页浏览器：基于系统自带的 WebView（Windows 用 WebView2，macOS 用 WKWebView），安装包只有几 MB。

- **无需登录**，打开就能用；地址栏可输入网址或直接搜索（默认 Bing，可在设置里换成 Google / 百度 / DuckDuckGo）
- 后退 / 前进 / 刷新 / 主页、多标签页、新窗口
- **本地历史记录**：标题 + 网址 + 时间，可搜索、单条删除、一键清空；只保存在本机，不上传
- 中文 / English 界面

## 下载

到 [Releases](https://github.com/kevinyuzekai/LiteBrowser/releases) 下载：

- Windows：`LiteBrowser_0.1.0_x64-setup.exe`（或 `.msi`）
- macOS：`LiteBrowser_0.1.0_universal.dmg`（Intel / Apple 芯片通用）

安装包未签名：Windows 若出现 SmartScreen 提示，点「更多信息 → 仍要运行」；macOS 首次打开请在「应用程序」里右键 →「打开」。

## 快捷键（Mac 用 Cmd）

| 快捷键 | 功能 |
|---|---|
| Ctrl+L | 聚焦地址栏 |
| Ctrl+R / F5 | 刷新 |
| Ctrl+[ / Ctrl+] | 后退 / 前进 |
| Ctrl+Y | 历史记录 |
| Ctrl+T / Ctrl+W | 新标签页 / 关闭标签页 |
| Ctrl+N | 新窗口 |

## 隐私

没有账号、没有统计、没有广告。浏览历史保存在本机应用数据目录的 `history.json` 中（Windows：`%APPDATA%\com.kevinyuzekai.litebrowser`，macOS：`~/Library/Application Support/com.kevinyuzekai.litebrowser`），网页 Cookie 由系统 WebView 管理。

## 构建

需要 Node.js 20+ 和 Rust（stable）。

```bash
npm ci
npx tauri build
```

推送 `v*` 标签后，GitHub Actions 会在 Windows 和 macOS 上自动打包并发布 Release。

## English

LiteBrowser is a tiny desktop browser built with Tauri v2 on the system WebView. No account, URL-or-search address bar, tabs, new windows, and a local-only history view (search, delete, clear). MIT licensed.
