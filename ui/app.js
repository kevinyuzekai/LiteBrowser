// Plume UI (browser chrome). Pages are separate native webviews managed by Rust.
(function () {
  'use strict';
  var T = window.__TAURI__;
  var invoke = T.core.invoke, listen = T.event.listen;
  var $ = function (id) { return document.getElementById(id); };

  var I18N = {
    zh: { back: '后退 (Ctrl+[)', forward: '前进 (Ctrl+])', reload: '刷新 (Ctrl+R)', home: '主页', history: '历史记录',
      settings: '设置', newTab: '新标签页 (Ctrl+T)', addr: '输入网址或搜索内容', searchHistory: '搜索历史记录',
      clearAll: '清空全部', close: '关闭', empty: '还没有历史记录', engine: '默认搜索引擎', homeUrl: '主页网址',
      lang: '界面语言', homeHint: '留空 = 新标签页', newTabTitle: '新标签页', confirmClear: '再点一次确认清空', del: '删除',
      today: '今天', yesterday: '昨天', closeTab: '关闭标签页',
      keys: '快捷键（Mac 用 Cmd）：\nCtrl+L 地址栏　Ctrl+R / F5 刷新　Ctrl+[ 后退　Ctrl+] 前进\nCtrl+Y 历史记录　Ctrl+T 新标签页　Ctrl+W 关闭标签页　Ctrl+N 新窗口',
      about: 'Plume Browser v0.1.2 · 无需登录 · 历史记录只保存在本机' },
    en: { back: 'Back (Ctrl+[)', forward: 'Forward (Ctrl+])', reload: 'Reload (Ctrl+R)', home: 'Home', history: 'History',
      settings: 'Settings', newTab: 'New tab (Ctrl+T)', addr: 'Enter a URL or search', searchHistory: 'Search history',
      clearAll: 'Clear all', close: 'Close', empty: 'No history yet', engine: 'Search engine', homeUrl: 'Home page',
      lang: 'Language', homeHint: 'Empty = new tab page', newTabTitle: 'New tab', confirmClear: 'Click again to clear', del: 'Delete',
      today: 'Today', yesterday: 'Yesterday', closeTab: 'Close tab',
      keys: 'Shortcuts (Cmd on Mac):\nCtrl+L address bar  Ctrl+R / F5 reload  Ctrl+[ back  Ctrl+] forward\nCtrl+Y history  Ctrl+T new tab  Ctrl+W close tab  Ctrl+N new window',
      about: 'Plume Browser v0.1.2 · No account needed · History is stored only on this device' }
  };
  var ENGINES = {
    bing: { home: 'https://www.bing.com/', q: 'https://www.bing.com/search?q=' },
    google: { home: 'https://www.google.com/', q: 'https://www.google.com/search?q=' },
    baidu: { home: 'https://www.baidu.com/', q: 'https://www.baidu.com/s?wd=' },
    ddg: { home: 'https://duckduckgo.com/', q: 'https://duckduckgo.com/?q=' }
  };
  var S = {
    engine: localStorage.getItem('plume.engine') || 'bing',
    home: localStorage.getItem('plume.home') || '',
    lang: localStorage.getItem('plume.lang') || (/^zh/i.test(navigator.language || '') ? 'zh' : 'en')
  };
  function L(k) { return (I18N[S.lang] || I18N.en)[k] || I18N.en[k] || k; }
  function applyLang() {
    document.documentElement.lang = S.lang === 'zh' ? 'zh-CN' : 'en';
    document.querySelectorAll('[data-t]').forEach(function (e) { e.title = L(e.dataset.t); });
    document.querySelectorAll('[data-l]').forEach(function (e) { e.textContent = L(e.dataset.l); });
    document.querySelectorAll('[data-p]').forEach(function (e) { e.placeholder = L(e.dataset.p); });
    $('addr').placeholder = L('addr');
    renderTabs();
  }
  function newTabUrl() { return 'plume:newtab?lang=' + S.lang + '&engine=' + S.engine; }
  function homeUrl() { return S.home || newTabUrl(); }
  // the bundled new-tab page lives on the app origin (tauri://localhost or http://tauri.localhost)
  function isInternal(u) { return /^plume:|^tauri:\/\/localhost\/|^https?:\/\/tauri\.localhost\//i.test(u || ''); }
  function shownUrl(u) { return !u || u === 'about:blank' || isInternal(u) ? '' : u; }

  // ---- URL / search parsing ----
  function toUrl(input) {
    var s = (input || '').trim();
    if (!s) return null;
    if (/^plume:/i.test(s)) return s;
    if (/^https?:\/\//i.test(s)) return s;
    if (/^about:blank$/i.test(s)) return 'about:blank';
    if (!/\s/.test(s)) {
      if (/^(localhost|\d{1,3}(\.\d{1,3}){3})(:\d+)?(\/.*)?$/i.test(s)) return 'http://' + s;
      if (/^[^\/?#]+\.[a-z\u00a1-\uffff-]{2,}(:\d+)?([\/?#].*)?$/i.test(s)) return 'https://' + s;
    }
    return (ENGINES[S.engine] || ENGINES.bing).q + encodeURIComponent(s);
  }

  // ---- tabs ----
  var tabs = [];   // {id, url, title, loading}
  var active = null;
  function tab(id) { for (var i = 0; i < tabs.length; i++) if (tabs[i].id === id) return tabs[i]; return null; }
  function hostOf(u) { try { return new URL(u).host; } catch (e) { return u; } }
  function renderTabs() {
    var box = $('tablist'); box.textContent = '';
    tabs.forEach(function (t) {
      var d = document.createElement('div');
      d.className = 'tab' + (t.id === active ? ' on' : '') + (t.loading ? ' ld' : '');
      var s = document.createElement('span'); s.className = 'tt';
      s.textContent = (!isInternal(t.url) && t.title) || (shownUrl(t.url) ? hostOf(t.url) : L('newTabTitle'));
      d.title = s.textContent;
      var x = document.createElement('button'); x.className = 'tx'; x.textContent = '\u00d7'; x.title = L('closeTab');
      x.onclick = function (e) { e.stopPropagation(); closeTab(t.id); };
      d.onclick = function () { showTab(t.id); };
      d.onauxclick = function (e) { if (e.button === 1) closeTab(t.id); };
      d.appendChild(s); d.appendChild(x); box.appendChild(d);
    });
    var t = tab(active);
    if (shownTab !== active) addrDirty = false;
    if (!addrDirty || document.activeElement !== $('addr') || shownTab !== active) $('addr').value = t ? shownUrl(t.url) : '';
    shownTab = active;
    document.title = (t && t.title ? t.title + ' - ' : '') + (S.lang === 'zh' ? 'Plume Browser' : 'Plume Browser');
  }
  var shownTab = null, addrDirty = false;
  function newTab(url, focusAddr) {
    hidePanel(true);
    return invoke('tab_create', { url: url || homeUrl() }).then(function (id) {
      tabs.push({ id: id, url: url || homeUrl(), title: '', loading: true });
      active = id; renderTabs();
      if (focusAddr) focusAddress();
    }).catch(console.error);
  }
  function showTab(id) {
    if (!tab(id)) return;
    active = id; hidePanel(true);
    invoke('tab_show', { tab: id }).catch(console.error); renderTabs();
  }
  function closeTab(id) {
    var i = tabs.findIndex(function (t) { return t.id === id; });
    if (i < 0) return;
    invoke('tab_close', { tab: id }).catch(console.error);
    tabs.splice(i, 1);
    if (!tabs.length) { invoke('close_window').catch(console.error); return; }
    if (active === id) showTab(tabs[Math.min(i, tabs.length - 1)].id); else renderTabs();
  }
  function go(input) {
    addrDirty = false;
    var u = toUrl(input); if (!u) return;
    if (!active) { newTab(u); return; }
    hidePanel(true);
    var t = tab(active); t.url = u; t.loading = true; renderTabs();
    $('addr').blur();
    invoke('tab_navigate', { tab: active, url: u }).catch(function (e) { console.error(e); });
  }
  function act(a) { if (active) invoke('tab_action', { tab: active, action: a }).catch(console.error); }
  function focusAddress() { hidePanel(true); var a = $('addr'); a.focus(); a.select(); }

  // ---- panel (history / settings) ----
  var panel = null;
  function showPanel(which) {
    panel = which;
    $('panel').hidden = false; $('hist').hidden = which !== 'hist'; $('sett').hidden = which !== 'sett';
    invoke('set_overlay', { overlay: true }).catch(console.error);
    if (which === 'hist') { $('hq').value = ''; loadHistory(); setTimeout(function () { $('hq').focus(); }, 30); }
    if (which === 'sett') { $('engine').value = S.engine; $('homeurl').value = S.home; $('lang').value = S.lang; }
  }
  function hidePanel(silent) {
    if (!panel) return;
    panel = null; $('panel').hidden = true;
    invoke('set_overlay', { overlay: false }).then(function () { if (!silent) act('focus'); }).catch(console.error);
  }
  function togglePanel(w) { if (panel === w) hidePanel(); else showPanel(w); }
  function pad(n) { return (n < 10 ? '0' : '') + n; }
  function dayLabel(d) {
    var now = new Date(), y = new Date(now); y.setDate(now.getDate() - 1);
    if (d.toDateString() === now.toDateString()) return L('today');
    if (d.toDateString() === y.toDateString()) return L('yesterday');
    return d.getFullYear() + '-' + pad(d.getMonth() + 1) + '-' + pad(d.getDate());
  }
  var hTimer = 0;
  function loadHistory() {
    invoke('history_list', { query: $('hq').value, limit: 1000 }).then(function (list) {
      var ul = $('hlist'); ul.textContent = ''; var lastDay = '';
      $('hempty').hidden = list.length > 0;
      list.forEach(function (e) {
        var d = new Date(e.time), dl = dayLabel(d);
        if (dl !== lastDay) { var h = document.createElement('li'); h.className = 'day'; h.textContent = dl; ul.appendChild(h); lastDay = dl; }
        var li = document.createElement('li');
        var tm = document.createElement('span'); tm.className = 'tm'; tm.textContent = pad(d.getHours()) + ':' + pad(d.getMinutes());
        var a = document.createElement('a'); a.textContent = e.title || e.url; a.title = e.url;
        var sm = document.createElement('small'); sm.textContent = hostOf(e.url); a.appendChild(sm);
        a.onclick = function () { go(e.url); };
        var del = document.createElement('button'); del.className = 'del'; del.textContent = '\u2715'; del.title = L('del');
        del.onclick = function () { invoke('history_delete', { id: e.id }).then(loadHistory); };
        li.appendChild(tm); li.appendChild(a); li.appendChild(del); ul.appendChild(li);
      });
    }).catch(console.error);
  }

  // ---- shortcuts ----
  function command(c, url) {
    switch (c) {
      case 'focus': focusAddress(); break;
      case 'reload': hidePanel(true); act('reload'); break;
      case 'back': act('back'); break;
      case 'forward': act('forward'); break;
      case 'history': togglePanel('hist'); break;
      case 'newtab': newTab(null, true); break;
      case 'closetab': if (active) closeTab(active); break;
      case 'newwindow': invoke('new_window').catch(console.error); break;
      case 'open': newTab(url); break;
    }
  }
  document.addEventListener('keydown', function (e) {
    if (e.key === 'Escape' && panel) { hidePanel(); return; }
    if (e.key === 'F5') { e.preventDefault(); command('reload'); return; }
    if (!(e.ctrlKey || e.metaKey)) return;
    var k = (e.key || '').toLowerCase();
    var map = { l: 'focus', r: 'reload', '[': 'back', ']': 'forward', y: 'history', t: 'newtab', w: 'closetab', n: 'newwindow' };
    if (map[k]) { e.preventDefault(); command(map[k]); }
  });

  // ---- events from Rust ----
  listen('plume-url', function (ev) {
    var p = ev.payload, t = tab(p.tab); if (!t) return;
    t.url = p.url; t.loading = p.loading; renderTabs();
  });
  listen('plume-title', function (ev) {
    var p = ev.payload, t = tab(p.tab); if (!t) return;
    t.title = p.title; renderTabs();
  });
  listen('plume-cmd', function (ev) {
    var p = ev.payload; if (p.tab && p.cmd !== 'open' && !tab(p.tab)) return;
    command(p.cmd, p.url);
  });

  // ---- wire up ----
  $('addr').addEventListener('input', function () { addrDirty = true; });
  $('addrform').onsubmit = function (e) { e.preventDefault(); go($('addr').value); };
  $('addr').onfocus = function () { setTimeout(function () { $('addr').select(); }, 0); };
  $('addr').onblur = function () { renderTabs(); };
  $('back').onclick = function () { act('back'); };
  $('fwd').onclick = function () { act('forward'); };
  $('reload').onclick = function () { command('reload'); };
  $('home').onclick = function () { go(homeUrl()); };
  $('newtab').onclick = function () { newTab(null, true); };
  $('histbtn').onclick = function () { togglePanel('hist'); };
  $('setbtn').onclick = function () { togglePanel('sett'); };
  document.querySelectorAll('.close').forEach(function (b) { b.onclick = function () { hidePanel(); }; });
  $('hq').oninput = function () { clearTimeout(hTimer); hTimer = setTimeout(loadHistory, 150); };
  // inline two-step confirm (native confirm() is not available in every system WebView)
  var clearArmed = 0;
  $('hclear').onclick = function () {
    var b = this;
    if (!clearArmed) { b.textContent = L('confirmClear'); clearArmed = setTimeout(function () { clearArmed = 0; b.textContent = L('clearAll'); }, 4000); return; }
    clearTimeout(clearArmed); clearArmed = 0; b.textContent = L('clearAll');
    invoke('history_clear').then(loadHistory);
  };
  $('engine').onchange = function () { S.engine = this.value; localStorage.setItem('plume.engine', S.engine); };
  $('homeurl').onchange = function () {
    var v = this.value.trim(); var u = v ? toUrl(v) : '';
    if (u && !/^https?:/.test(u)) u = '';
    S.home = u; this.value = u; localStorage.setItem('plume.home', u);
  };
  $('lang').onchange = function () { S.lang = this.value; localStorage.setItem('plume.lang', S.lang); applyLang(); };

  applyLang();
  newTab();
})();
