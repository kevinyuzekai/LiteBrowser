// Plume Browser — a tiny Tauri v2 browser using the system WebView.
// No accounts, no proxy features. History is stored locally only.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::webview::{NewWindowResponse, PageLoadEvent, WebviewBuilder};
use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Url, Webview, WebviewUrl, Window,
    WindowBuilder, WindowEvent,
};

/// Height (logical px) of the tab strip + toolbar drawn by the UI webview.
const TOP: f64 = 78.0;
/// Reserved, never-resolving host used by page-side shortcut forwarding.
const CMD_HOST: &str = "plume-cmd.invalid";
const MAX_HISTORY: usize = 5000;

#[derive(Serialize, Deserialize, Clone)]
struct Entry {
    id: u64,
    url: String,
    title: String,
    time: u64,
}

#[derive(Default)]
struct WinState {
    active: Option<String>,
    overlay: bool,
}

struct AppState {
    history: Mutex<Vec<Entry>>,
    history_path: Mutex<Option<PathBuf>>,
    next_id: AtomicU64,
    // tab label -> window label
    tabs: Mutex<HashMap<String, String>>,
    // tab label -> last history entry id (for title updates)
    tab_hist: Mutex<HashMap<String, u64>>,
    // tab label -> title seen before the page finished loading
    pending_title: Mutex<HashMap<String, String>>,
    wins: Mutex<HashMap<String, WinState>>,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn save_history(state: &AppState) {
    let path = state.history_path.lock().unwrap().clone();
    if let Some(p) = path {
        let h = state.history.lock().unwrap();
        if let Ok(s) = serde_json::to_string(&*h) {
            let tmp = p.with_extension("tmp");
            if std::fs::write(&tmp, s).is_ok() {
                let _ = std::fs::rename(&tmp, &p);
            }
        }
    }
}

fn is_internal(u: &Url) -> bool {
    u.scheme() == "tauri" || u.host_str() == Some("tauri.localhost")
}

fn recordable(u: &Url) -> bool {
    matches!(u.scheme(), "http" | "https") && u.host_str() != Some(CMD_HOST) && !is_internal(u)
}

fn record_visit(app: &AppHandle, tab: &str, url: &Url) {
    if !recordable(url) {
        return;
    }
    let state = app.state::<AppState>();
    let id;
    {
        let mut h = state.history.lock().unwrap();
        let t = now_ms();
        if let Some(last) = h.last_mut().filter(|e| e.url == url.as_str()) {
            last.time = t;
            id = last.id;
        } else {
            id = state.next_id.fetch_add(1, Ordering::SeqCst);
            let title = state.pending_title.lock().unwrap().remove(tab).unwrap_or_default();
            h.push(Entry { id, url: url.to_string(), title, time: t });
            let len = h.len();
            if len > MAX_HISTORY {
                h.drain(0..len - MAX_HISTORY);
            }
        }
    }
    state.tab_hist.lock().unwrap().insert(tab.to_string(), id);
    save_history(&state);
}

fn record_title(app: &AppHandle, tab: &str, cur: Option<Url>, title: &str) {
    let state = app.state::<AppState>();
    let id = state.tab_hist.lock().unwrap().get(tab).copied();
    let changed = {
        let mut h = state.history.lock().unwrap();
        let hit = id.and_then(|id| h.iter_mut().rev().find(|e| e.id == id))
            .filter(|e| cur.as_ref().map(|u| u.as_str() == e.url).unwrap_or(false));
        if hit.is_none() {
            // the page for this title has not finished loading yet; attach it on Finished
            state.pending_title.lock().unwrap().insert(tab.to_string(), title.to_string());
        }
        match hit {
            Some(e) if e.title != title => {
                e.title = title.to_string();
                true
            }
            _ => false,
        }
    };
    if changed {
        save_history(&state);
    }
}

fn logical_size(window: &Window) -> (f64, f64) {
    let sf = window.scale_factor().unwrap_or(1.0);
    let s = window.inner_size().map(|s| s.to_logical::<f64>(sf)).unwrap_or(LogicalSize::new(1200.0, 800.0));
    (s.width, s.height)
}

fn relayout(window: &Window) {
    let app = window.app_handle();
    let state = app.state::<AppState>();
    let (w, h) = logical_size(window);
    let (active, overlay) = {
        let wins = state.wins.lock().unwrap();
        let ws = wins.get(window.label());
        (ws.and_then(|s| s.active.clone()), ws.map(|s| s.overlay).unwrap_or(false))
    };
    let ui_label = format!("ui-{}", window.label());
    for wv in window.webviews() {
        // Linux: child webviews are packed into a GtkBox (positions are ignored),
        // so give the UI strip a fixed height unless the overlay panel is open.
        #[cfg(target_os = "linux")]
        {
            let is_ui = wv.label() == ui_label;
            let content_h = (h - TOP).max(1.0) as i32;
            let _ = wv.with_webview(move |pv| {
                use gtk::prelude::*;
                let w = pv.inner();
                let (exp, req) = if is_ui {
                    (overlay, if overlay { -1 } else { TOP as i32 })
                } else {
                    (true, if overlay { -1 } else { content_h })
                };
                w.set_size_request(-1, req);
                if let Some(b) = w.parent().and_then(|p| p.downcast::<gtk::Box>().ok()) {
                    b.set_homogeneous(false);
                    b.set_child_packing(&w, exp, true, 0, gtk::PackType::Start);
                    b.queue_resize();
                }
            });
        }
        if wv.label() == ui_label {
            let _ = wv.set_position(LogicalPosition::new(0.0, 0.0));
            let _ = wv.set_size(LogicalSize::new(w, if overlay { h } else { TOP }));
        } else {
            let _ = wv.set_position(LogicalPosition::new(0.0, TOP));
            let _ = wv.set_size(LogicalSize::new(w, (h - TOP).max(1.0)));
            if !overlay && active.as_deref() == Some(wv.label()) {
                let _ = wv.show();
            } else {
                let _ = wv.hide();
            }
        }
    }
}

fn ensure_ui(webview: &Webview) -> Result<(), String> {
    if webview.label().starts_with("ui-") {
        Ok(())
    } else {
        Err("forbidden".into())
    }
}

fn find_tab(app: &AppHandle, tab: &str) -> Result<Webview, String> {
    if !tab.starts_with("tab-") {
        return Err("bad tab".into());
    }
    app.get_webview(tab).ok_or_else(|| "no such tab".to_string())
}

/// Resolves the URL a tab should load. `plume:newtab?...` maps to the bundled
/// new-tab page (served from the app origin, which differs per platform:
/// tauri://localhost on macOS/Linux, http://tauri.localhost on Windows).
fn resolve_url(ui: &Webview, s: &str) -> Result<Url, String> {
    if let Some(q) = s.strip_prefix("plume:newtab") {
        let base = ui.url().map_err(|e| e.to_string())?;
        return base.join(&format!("newtab.html{q}")).map_err(|e| e.to_string());
    }
    let u = Url::parse(s).map_err(|e| e.to_string())?;
    match u.scheme() {
        "http" | "https" => Ok(u),
        "about" if s == "about:blank" => Ok(u),
        _ => Err("only http/https URLs are supported".into()),
    }
}

/// Script injected into every web page: forwards browser shortcuts to the app
/// by requesting a reserved, never-loaded URL that the navigation handler cancels.
const PAGE_SCRIPT: &str = r#"
(function(){
  if (window.top !== window) return;
  var H = 'https://plume-cmd.invalid/';
  function send(c){ try { window.location.href = H + c; } catch(e){} }
  window.addEventListener('keydown', function(e){
    var m = e.ctrlKey || e.metaKey; if (!m) { if (e.key === 'F5') { e.preventDefault(); send('reload'); } return; }
    var k = (e.key || '').toLowerCase(), c = null;
    if (k === 'l') c = 'focus'; else if (k === 'r') c = 'reload'; else if (k === '[') c = 'back';
    else if (k === ']') c = 'forward'; else if (k === 'y') c = 'history'; else if (k === 't') c = 'newtab';
    else if (k === 'w') c = 'closetab'; else if (k === 'n') c = 'newwindow';
    if (c) { e.preventDefault(); e.stopPropagation(); send(c); }
  }, true);
})();
"#;

fn handle_cmd(app: &AppHandle, tab: &str, cmd: &str) {
    match cmd {
        "back" => { if let Some(w) = app.get_webview(tab) { let _ = w.eval("history.back()"); } }
        "forward" => { if let Some(w) = app.get_webview(tab) { let _ = w.eval("history.forward()"); } }
        "reload" => { if let Some(w) = app.get_webview(tab) { let _ = w.eval("location.reload()"); } }
        "newwindow" => { let _ = open_window(app, None); }
        _ => {
            let win = app.state::<AppState>().tabs.lock().unwrap().get(tab).cloned();
            if let Some(win) = win {
                let _ = app.emit_to(
                    tauri::EventTarget::webview(format!("ui-{win}")),
                    "plume-cmd",
                    serde_json::json!({ "tab": tab, "cmd": cmd }),
                );
            }
        }
    }
}

fn emit_ui(app: &AppHandle, tab: &str, event: &str, payload: serde_json::Value) {
    let win = app.state::<AppState>().tabs.lock().unwrap().get(tab).cloned();
    if let Some(win) = win {
        let _ = app.emit_to(tauri::EventTarget::webview(format!("ui-{win}")), event, payload);
    }
}

fn build_tab(app: &AppHandle, window: &Window, url: Url) -> Result<String, String> {
    let state = app.state::<AppState>();
    let label = format!("tab-{}", state.next_id.fetch_add(1, Ordering::SeqCst));
    state.tabs.lock().unwrap().insert(label.clone(), window.label().to_string());

    let a1 = app.clone();
    let l1 = label.clone();
    let a2 = app.clone();
    let l2 = label.clone();
    let a3 = app.clone();
    let l3 = label.clone();
    let a4 = app.clone();
    let l4 = label.clone();
    let builder = WebviewBuilder::new(&label, WebviewUrl::External(url))
        .initialization_script(PAGE_SCRIPT)
        .on_navigation(move |u| {
            if u.host_str() == Some(CMD_HOST) {
                let cmd = u.path().trim_start_matches('/').to_string();
                let a = a1.clone();
                let l = l1.clone();
                // Never create/close webviews inside a WebView2/WKWebView navigation callback:
                // hand the command to a worker task, which dispatches to the main thread.
                tauri::async_runtime::spawn(async move { handle_cmd(&a, &l, &cmd) });
                return false;
            }
            // Only web pages; other schemes (mailto:, custom app links...) are ignored.
            matches!(u.scheme(), "http" | "https" | "about" | "data" | "blob") || is_internal(u)
        })
        .on_page_load(move |_wv, p| {
            let u = p.url().clone();
            if u.host_str() == Some(CMD_HOST) {
                return;
            }
            let loading = matches!(p.event(), PageLoadEvent::Started);
            if loading {
                a2.state::<AppState>().pending_title.lock().unwrap().remove(&l2);
            } else {
                record_visit(&a2, &l2, &u);
            }
            emit_ui(&a2, &l2, "plume-url", serde_json::json!({ "tab": l2, "url": u.as_str(), "loading": loading }));
        })
        .on_document_title_changed(move |wv, title| {
            record_title(&a3, &l3, wv.url().ok(), &title);
            emit_ui(&a3, &l3, "plume-title", serde_json::json!({ "tab": l3, "title": title }));
        })
        .on_new_window(move |u, _f| {
            // target=_blank / window.open -> open as a new tab in this window
            if matches!(u.scheme(), "http" | "https") {
                emit_ui(&a4, &l4, "plume-cmd", serde_json::json!({ "tab": l4, "cmd": "open", "url": u.as_str() }));
            }
            NewWindowResponse::Deny
        });
    let (w, h) = logical_size(window);
    window
        .add_child(builder, LogicalPosition::new(0.0, TOP), LogicalSize::new(w, (h - TOP).max(1.0)))
        .map_err(|e| e.to_string())?;
    Ok(label)
}

#[tauri::command]
async fn tab_create(app: AppHandle, webview: Webview, url: String) -> Result<String, String> {
    ensure_ui(&webview)?;
    let window = webview.window();
    let label = build_tab(&app, &window, resolve_url(&webview, &url)?)?;
    {
        let state = app.state::<AppState>();
        let mut wins = state.wins.lock().unwrap();
        let ws = wins.entry(window.label().to_string()).or_default();
        ws.active = Some(label.clone());
        ws.overlay = false;
    }
    relayout(&window);
    Ok(label)
}

#[tauri::command]
async fn tab_show(app: AppHandle, webview: Webview, tab: String) -> Result<(), String> {
    ensure_ui(&webview)?;
    let wv = find_tab(&app, &tab)?;
    let window = webview.window();
    {
        let state = app.state::<AppState>();
        let mut wins = state.wins.lock().unwrap();
        wins.entry(window.label().to_string()).or_default().active = Some(tab);
    }
    relayout(&window);
    // keep keyboard shortcuts (Ctrl/Cmd+W etc.) working after the focused tab was closed
    let _ = wv.set_focus();
    Ok(())
}

#[tauri::command]
async fn tab_close(app: AppHandle, webview: Webview, tab: String) -> Result<(), String> {
    ensure_ui(&webview)?;
    let wv = find_tab(&app, &tab)?;
    let state = app.state::<AppState>();
    state.tabs.lock().unwrap().remove(&tab);
    state.tab_hist.lock().unwrap().remove(&tab);
    state.pending_title.lock().unwrap().remove(&tab);
    wv.close().map_err(|e| e.to_string())
}

#[tauri::command]
async fn tab_navigate(app: AppHandle, webview: Webview, tab: String, url: String) -> Result<(), String> {
    ensure_ui(&webview)?;
    let wv = find_tab(&app, &tab)?;
    wv.navigate(resolve_url(&webview, &url)?).map_err(|e| e.to_string())?;
    let _ = wv.set_focus();
    Ok(())
}

#[tauri::command]
async fn tab_action(app: AppHandle, webview: Webview, tab: String, action: String) -> Result<(), String> {
    ensure_ui(&webview)?;
    let wv = find_tab(&app, &tab)?;
    let js = match action.as_str() {
        "back" => "history.back()",
        "forward" => "history.forward()",
        "reload" => "location.reload()",
        "stop" => "window.stop()",
        "focus" => {
            let _ = wv.set_focus();
            return Ok(());
        }
        _ => return Err("unknown action".into()),
    };
    wv.eval(js).map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_overlay(app: AppHandle, webview: Webview, overlay: bool) -> Result<(), String> {
    ensure_ui(&webview)?;
    let window = webview.window();
    {
        let state = app.state::<AppState>();
        let mut wins = state.wins.lock().unwrap();
        wins.entry(window.label().to_string()).or_default().overlay = overlay;
    }
    relayout(&window);
    if overlay {
        let _ = webview.set_focus();
    }
    Ok(())
}

#[tauri::command]
async fn new_window(app: AppHandle, webview: Webview) -> Result<(), String> {
    ensure_ui(&webview)?;
    open_window(&app, None).map_err(|e| e.to_string())
}

/// Closes the window that owns this UI (used when the last tab is closed).
#[tauri::command]
async fn close_window(webview: Webview) -> Result<(), String> {
    ensure_ui(&webview)?;
    webview.window().destroy().map_err(|e| e.to_string())
}

#[tauri::command]
async fn history_list(app: AppHandle, webview: Webview, query: String, limit: Option<usize>) -> Result<Vec<Entry>, String> {
    ensure_ui(&webview)?;
    let q = query.trim().to_lowercase();
    let lim = limit.unwrap_or(500);
    let h = app.state::<AppState>().history.lock().unwrap().clone();
    Ok(h.into_iter()
        .rev()
        .filter(|e| q.is_empty() || e.url.to_lowercase().contains(&q) || e.title.to_lowercase().contains(&q))
        .take(lim)
        .collect())
}

#[tauri::command]
async fn history_delete(app: AppHandle, webview: Webview, id: u64) -> Result<(), String> {
    ensure_ui(&webview)?;
    let state = app.state::<AppState>();
    state.history.lock().unwrap().retain(|e| e.id != id);
    save_history(&state);
    Ok(())
}

#[tauri::command]
async fn history_clear(app: AppHandle, webview: Webview) -> Result<(), String> {
    ensure_ui(&webview)?;
    let state = app.state::<AppState>();
    state.history.lock().unwrap().clear();
    state.tab_hist.lock().unwrap().clear();
    save_history(&state);
    Ok(())
}

fn open_window(app: &AppHandle, _url: Option<Url>) -> tauri::Result<()> {
    let state = app.state::<AppState>();
    let n = state.next_id.fetch_add(1, Ordering::SeqCst);
    let label = format!("w{n}");
    let window = WindowBuilder::new(app, &label)
        .title("Plume Browser")
        .inner_size(1200.0, 800.0)
        .min_inner_size(480.0, 320.0)
        .build()?;
    state.wins.lock().unwrap().insert(label.clone(), WinState::default());
    let (w, _h) = logical_size(&window);
    window.add_child(
        WebviewBuilder::new(format!("ui-{label}"), WebviewUrl::App("index.html".into())),
        LogicalPosition::new(0.0, 0.0),
        LogicalSize::new(w, TOP),
    )?;
    let win2 = window.clone();
    window.on_window_event(move |e| match e {
        WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => relayout(&win2),
        WindowEvent::Destroyed => {
            let app = win2.app_handle();
            let state = app.state::<AppState>();
            state.wins.lock().unwrap().remove(win2.label());
            state.tabs.lock().unwrap().retain(|_, w| w != win2.label());
        }
        _ => {}
    });
    Ok(())
}

/// macOS: the default app menu binds Cmd+W to "Close Window" (closing all tabs at once)
/// and Cmd+N etc. are swallowed by the menu bar before the page sees them, so we install
/// a browser menu whose accelerators drive the tab UI. Edit items keep copy/paste working.
#[cfg(target_os = "macos")]
fn setup_macos_menu(app: &AppHandle) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
    let item = |id: &str, label: &str, acc: &str| MenuItem::with_id(app, id, label, true, Some(acc));
    let app_menu = Submenu::with_items(app, "Plume Browser", true, &[
        &PredefinedMenuItem::about(app, Some("About Plume Browser"), None)?,
        &PredefinedMenuItem::separator(app)?,
        &PredefinedMenuItem::hide(app, None)?,
        &PredefinedMenuItem::hide_others(app, None)?,
        &PredefinedMenuItem::show_all(app, None)?,
        &PredefinedMenuItem::separator(app)?,
        &PredefinedMenuItem::quit(app, None)?,
    ])?;
    let file = Submenu::with_items(app, "File", true, &[
        &item("newtab", "New Tab", "CmdOrCtrl+T")?,
        &item("newwindow", "New Window", "CmdOrCtrl+N")?,
        &item("focus", "Open Location…", "CmdOrCtrl+L")?,
        &PredefinedMenuItem::separator(app)?,
        &item("closetab", "Close Tab", "CmdOrCtrl+W")?,
        &item("closewindow", "Close Window", "CmdOrCtrl+Shift+W")?,
    ])?;
    let edit = Submenu::with_items(app, "Edit", true, &[
        &PredefinedMenuItem::undo(app, None)?,
        &PredefinedMenuItem::redo(app, None)?,
        &PredefinedMenuItem::separator(app)?,
        &PredefinedMenuItem::cut(app, None)?,
        &PredefinedMenuItem::copy(app, None)?,
        &PredefinedMenuItem::paste(app, None)?,
        &PredefinedMenuItem::select_all(app, None)?,
    ])?;
    let view = Submenu::with_items(app, "View", true, &[
        &item("reload", "Reload", "CmdOrCtrl+R")?,
        &item("back", "Back", "CmdOrCtrl+[")?,
        &item("forward", "Forward", "CmdOrCtrl+]")?,
        &PredefinedMenuItem::separator(app)?,
        &item("history", "History", "CmdOrCtrl+Y")?,
    ])?;
    let window = Submenu::with_items(app, "Window", true, &[
        &PredefinedMenuItem::minimize(app, None)?,
        &PredefinedMenuItem::fullscreen(app, None)?,
    ])?;
    app.set_menu(Menu::with_items(app, &[&app_menu, &file, &edit, &view, &window])?)?;
    app.on_menu_event(|app, event| {
        let id = event.id().as_ref().to_string();
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            if id == "newwindow" {
                let _ = open_window(&app, None);
                return;
            }
            let focused = app.windows().into_values().find(|w| w.is_focused().unwrap_or(false));
            if let Some(w) = focused {
                if id == "closewindow" {
                    let _ = w.destroy();
                    return;
                }
                let _ = app.emit_to(
                    tauri::EventTarget::webview(format!("ui-{}", w.label())),
                    "plume-cmd",
                    serde_json::json!({ "tab": null, "cmd": id }),
                );
            }
        });
    });
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            history: Mutex::new(Vec::new()),
            history_path: Mutex::new(None),
            next_id: AtomicU64::new(1),
            tabs: Mutex::new(HashMap::new()),
            tab_hist: Mutex::new(HashMap::new()),
            pending_title: Mutex::new(HashMap::new()),
            wins: Mutex::new(HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            tab_create, tab_show, tab_close, tab_navigate, tab_action, set_overlay, new_window, close_window,
            history_list, history_delete, history_clear
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            if let Ok(dir) = handle.path().app_data_dir() {
                let _ = std::fs::create_dir_all(&dir);
                let p = dir.join("history.json");
                let state = handle.state::<AppState>();
                if let Ok(s) = std::fs::read_to_string(&p) {
                    if let Ok(v) = serde_json::from_str::<Vec<Entry>>(&s) {
                        let max = v.iter().map(|e| e.id).max().unwrap_or(0);
                        state.next_id.store(max + 1, Ordering::SeqCst);
                        *state.history.lock().unwrap() = v;
                    }
                }
                *state.history_path.lock().unwrap() = Some(p);
            }
            #[cfg(target_os = "macos")]
            setup_macos_menu(&handle)?;
            open_window(&handle, None)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Plume");
}
