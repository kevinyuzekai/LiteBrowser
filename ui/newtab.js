// Plume new tab page. Lives on the app origin but runs in a web-page tab, so it has no app permissions;
// search simply navigates this tab.
(function () {
  'use strict';
  var $ = function (id) { return document.getElementById(id); };
  var qs = new URLSearchParams(location.search);
  var lang = qs.get('lang') === 'en' ? 'en' : (qs.get('lang') === 'zh' ? 'zh' : (/^zh/i.test(navigator.language || '') ? 'zh' : 'en'));
  var engine = qs.get('engine') || 'bing';
  var ENGINES = {
    bing: { name: 'Bing', q: 'https://www.bing.com/search?q=' },
    google: { name: 'Google', q: 'https://www.google.com/search?q=' },
    baidu: { name: '百度', q: 'https://www.baidu.com/s?wd=' },
    ddg: { name: 'DuckDuckGo', q: 'https://duckduckgo.com/?q=' }
  };
  var E = ENGINES[engine] || ENGINES.bing;
  var T = {
    zh: { title: '新标签页', brand: 'Plume', ph: '用 ' + E.name + ' 搜索，或输入网址', add: '添加快捷方式', name: '名称', url: '网址', save: '完成', cancel: '取消', del: '移除' },
    en: { title: 'New tab', brand: 'Plume', ph: 'Search ' + E.name + ' or type a URL', add: 'Add shortcut', name: 'Name', url: 'URL', save: 'Done', cancel: 'Cancel', del: 'Remove' }
  }[lang];
  document.documentElement.lang = lang === 'zh' ? 'zh-CN' : 'en';
  document.title = T.title;
  $('brand').textContent = T.brand;
  $('q').placeholder = T.ph;
  $('an').placeholder = T.name; $('au').placeholder = T.url; $('asave').textContent = T.save; $('acancel').textContent = T.cancel;

  function toUrl(input) {
    var s = (input || '').trim();
    if (!s) return null;
    if (/^https?:\/\//i.test(s)) return s;
    if (!/\s/.test(s)) {
      if (/^(localhost|\d{1,3}(\.\d{1,3}){3})(:\d+)?(\/.*)?$/i.test(s)) return 'http://' + s;
      if (/^[^\/?#]+\.[a-z\u00a1-\uffff-]{2,}(:\d+)?([\/?#].*)?$/i.test(s)) return 'https://' + s;
    }
    return E.q + encodeURIComponent(s);
  }
  $('sf').onsubmit = function (e) { e.preventDefault(); var u = toUrl($('q').value); if (u) location.href = u; };

  var DEFAULTS = [
    ['Google', 'https://www.google.com/'], ['YouTube', 'https://www.youtube.com/'], ['GitHub', 'https://github.com/'],
    ['Wikipedia', 'https://www.wikipedia.org/'], ['Bilibili', 'https://www.bilibili.com/'], [lang === 'zh' ? '百度' : 'Baidu', 'https://www.baidu.com/'],
    ['Bing', 'https://www.bing.com/'], [lang === 'zh' ? '知乎' : 'Zhihu', 'https://www.zhihu.com/']
  ];
  var KEY = 'plume.tiles';
  function load() { try { var v = JSON.parse(localStorage.getItem(KEY)); if (Array.isArray(v)) return v; } catch (e) {} return DEFAULTS.slice(); }
  function save(v) { try { localStorage.setItem(KEY, JSON.stringify(v)); } catch (e) {} }
  var tiles = load();
  function render() {
    var box = $('tiles'); box.textContent = '';
    tiles.forEach(function (t, i) {
      var a = document.createElement('a'); a.className = 'tile'; a.href = t[1]; a.title = t[1];
      var ic = document.createElement('span'); ic.className = 'ic'; ic.textContent = (t[0] || '?').charAt(0).toUpperCase();
      try {
        var img = new Image(); img.alt = '';
        img.onload = function () { ic.textContent = ''; ic.appendChild(img); };
        img.src = new URL('/favicon.ico', t[1]).href;
      } catch (e) {}
      var L = {'百度':'Baidu','Baidu':'百度','知乎':'Zhihu','Zhihu':'知乎'}, n0 = t[0];
      if (L[n0] && /baidu\.com|zhihu\.com/.test(t[1])) n0 = (lang === 'zh') === /[\u4e00-\u9fff]/.test(n0) ? n0 : L[n0];
      var nm = document.createElement('span'); nm.className = 'nm'; nm.textContent = n0;
      var x = document.createElement('button'); x.className = 'x'; x.type = 'button'; x.textContent = '\u00d7'; x.title = T.del;
      x.onclick = function (e) { e.preventDefault(); e.stopPropagation(); tiles.splice(i, 1); save(tiles); render(); };
      a.appendChild(ic); a.appendChild(nm); a.appendChild(x); box.appendChild(a);
    });
    if (tiles.length < 12) {
      var add = document.createElement('div'); add.className = 'tile add'; add.title = T.add;
      add.innerHTML = '<span class="ic">+</span>';
      var nm = document.createElement('span'); nm.className = 'nm'; nm.textContent = T.add; add.appendChild(nm);
      add.onclick = function () { $('addf').hidden = false; $('an').value = ''; $('au').value = ''; $('an').focus(); };
      box.appendChild(add);
    }
  }
  $('addf').onsubmit = function (e) {
    e.preventDefault();
    var u = toUrl($('au').value), n = $('an').value.trim();
    if (!u || !/^https?:/.test(u) || u.indexOf(E.q) === 0) { $('au').focus(); return; }
    if (!n) { try { n = new URL(u).hostname.replace(/^www\./, ''); } catch (err) { n = u; } }
    tiles.push([n, u]); save(tiles); $('addf').hidden = true; render();
  };
  $('acancel').onclick = function () { $('addf').hidden = true; };
  render();
})();
