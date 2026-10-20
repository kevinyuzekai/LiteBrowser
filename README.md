# Plume 羽览

羽览（Plume）是一个超轻量的桌面网页浏览器：基于系统自带的 WebView（Windows 用 WebView2，macOS 用 WKWebView），安装包只有几 MB。

- **无需登录**，打开就能用；地址栏可输入网址或直接搜索（默认 Bing，可在设置里换成 Google / 百度 / DuckDuckGo）
- 后退 / 前进 / 刷新 / 主页、多标签页、新窗口
- **本地历史记录**：标题 + 网址 + 时间，可搜索、单条删除、一键清空；只保存在本机，不上传
- 中文 / English 界面

## 下载

到 [Releases](../../releases) 下载：

- Windows：`Plume_0.1.1_x64-setup.exe`（约 1.3 MB，推荐）或 `Plume_0.1.1_x64_zh-CN.msi` / `_en-US.msi`（约 1.7 MB）
- macOS：`Plume_0.1.1_universal.dmg`（Intel / Apple 芯片通用，约 3.4 MB）
- 微软商店 / Microsoft Store：[Plume Browser](https://apps.microsoft.com/detail/9NHJ2KGT51X3)（Windows，商店自动更新，无 SmartScreen 提示）

安装包未签名：Windows 若出现 SmartScreen 提示，点「更多信息 → 仍要运行」；macOS 首次打开请在「应用程序」里右键 →「打开」。

## 相比主流浏览器的优势

| | Plume 羽览 v0.1.1 | Chrome | Firefox | Edge / Safari |
|---|---|---|---|---|
| Windows 安装包 | 约 1.3 MB（NSIS）/ 约 1.7 MB（MSI） | 约 167 MB（离线安装包） | 约 95 MB | 系统自带 |
| macOS 安装包 | 约 3.4 MB（Universal DMG） | 约 276 MB（Universal DMG） | 约 165 MB | Safari 系统自带 |
| 网页内核 | 系统 WebView（WebView2 / WKWebView），不自带内核 | 自带 Chromium | 自带 Gecko | 系统内核 |
| 账号 / 同步 | 没有 | 可选 Google 账号同步 | 可选 Firefox 账号同步 | 可选微软 / Apple 账号同步 |

> 主流浏览器的大小是 2026 年 10 月 19 日查看官方下载链接得到的文件大小（Chrome：Google 官方离线安装包和 Mac DMG；Firefox 158.0 en-US），只是大约值，会随版本变化。

- **非常小**：不自带浏览器内核，直接用系统里的 WebView2（Windows）或 WKWebView（macOS），所以安装包只有几 MB。
- **不用登录**：没有账号，也没有同步功能，打开就能用。
- **历史只存在本机**：存在应用数据目录的 `history.json` 里，可以在「历史记录」里搜索、单条删除、一键清空。
- **不上传数据**：本项目代码里没有统计、遥测或广告，也不会自己发网络请求；联网的只有你打开的网页。
- **权限很少**：网页标签页拿不到任何程序内部接口，只有浏览器界面本身能调用这几个功能（标签页、历史记录）。

### 局限

- 不支持扩展 / 插件
- 没有同步、密码管理器和书签（v0.1.1）
- 没有下载管理界面
- 网页显示效果取决于系统 WebView：Windows 需要 WebView2 运行时（Win10/11 一般自带），macOS 取决于系统 Safari 的版本
- 安装包没有签名，第一次打开时系统可能会提示

## Advantages over mainstream browsers

| | Plume 羽览 v0.1.1 | Chrome | Firefox | Edge / Safari |
|---|---|---|---|---|
| Windows installer | ~1.3 MB (NSIS) / ~1.7 MB (MSI) | ~167 MB (offline installer) | ~95 MB | Built into the OS |
| macOS installer | ~3.4 MB (universal DMG) | ~276 MB (universal DMG) | ~165 MB | Safari ships with macOS |
| Engine | System WebView (WebView2 / WKWebView), no bundled engine | Bundled Chromium | Bundled Gecko | System engine |
| Account / sync | None | Optional Google sync | Optional Firefox account sync | Optional Microsoft / Apple sync |

> Mainstream sizes are the file sizes of the official downloads checked on 2026-10-19 (Chrome official offline installer and Mac DMG; Firefox 158.0 en-US). They are approximate and change between versions.

- **Tiny**: no bundled engine. Plume uses the system WebView2 (Windows) or WKWebView (macOS), so the installer is only a few MB.
- **No sign-in**: no accounts and no sync.
- **Local-only history**: stored in `history.json` in the app data folder, with search, per-item delete and clear all.
- **No telemetry**: this project's code has no analytics, telemetry or ads, and makes no network requests of its own; only the pages you open go online.
- **Few permissions**: web pages get no access to the app's internal commands; only the browser UI can call them.

### Limitations

- No extensions
- No sync, password manager or bookmarks (v0.1.1)
- No download manager
- Rendering depends on the system WebView: Windows needs the WebView2 runtime (usually preinstalled on Windows 10/11), and macOS depends on the system Safari version
- Installers are unsigned, so the OS may warn the first time you open the app

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

没有账号、没有统计、没有广告。浏览历史保存在本机应用数据目录的 `history.json` 中（Windows：`%APPDATA%\com.kevinyuzekai.plume`，macOS：`~/Library/Application Support/com.kevinyuzekai.plume`），网页 Cookie 由系统 WebView 管理。

## 构建

需要 Node.js 20+ 和 Rust（stable）。

```bash
npm ci
npx tauri build
```

推送 `v*` 标签后，GitHub Actions 会在 Windows 和 macOS 上自动打包并发布 Release。

## English

Plume (羽览) is a tiny desktop browser built with Tauri v2 on the system WebView. No account, URL-or-search address bar, tabs, new windows, and a local-only history view (search, delete, clear). MIT licensed.

> CI：`.github/workflows/release.yml`（手动运行）在 macOS 上打 Universal DMG、在 Windows 上打 MSI，并上传到已有 Release（不会覆盖已有文件）。
