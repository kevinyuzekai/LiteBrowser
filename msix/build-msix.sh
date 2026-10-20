#!/bin/bash
# Plume Browser — 生成 Microsoft Store 用 MSIX（x64，未签名；商店发布时由微软签名）
# 用法：
#   ./build-msix.sh                         # 用 identity.env 里的值
#   ./build-msix.sh <IdentityName> <Publisher> <PublisherDisplayName> [Version]   # 命令行覆盖
#   DISPLAY_NAME="Plume Browser" ./build-msix.sh ...                                 # 覆盖显示名称
# 需要 Windows 版 Plume.exe：默认取 ../../Plume/src-tauri/target/x86_64-pc-windows-msvc/release/Plume.exe，
# 也可用 EXE=/path/Plume.exe 指定。
set -euo pipefail
cd "$(dirname "$0")"
_DN="${DISPLAY_NAME:-}"
source ./identity.env
[[ -n "$_DN" ]] && DISPLAY_NAME="$_DN"
IDENTITY_NAME="${1:-$IDENTITY_NAME}"; PUBLISHER="${2:-$PUBLISHER}"
PUBLISHER_DISPLAY_NAME="${3:-$PUBLISHER_DISPLAY_NAME}"; VERSION="${4:-$VERSION}"
EXE="${EXE:-/workspace/Plume/src-tauri/target/x86_64-pc-windows-msvc/release/Plume.exe}"
[[ "$PUBLISHER" == CN=* ]] || { echo "错误: Publisher 必须以 CN= 开头"; exit 1; }
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.0$ ]] || { echo "错误: Version 必须形如 0.1.0.0（第 4 段为 0）"; exit 1; }
[[ -f "$EXE" ]] || { echo "错误: 找不到 $EXE"; exit 1; }
[[ "$IDENTITY_NAME" == *Placeholder* ]] && echo "警告: 占位身份，只能测试，不能上传商店。"

LAYOUT=build/layout; rm -rf build && mkdir -p "$LAYOUT/assets"
cp "$EXE" "$LAYOUT/Plume.exe"
cp assets/*.png "$LAYOUT/assets/"
esc() { printf '%s' "$1" | sed -e 's/&/\&amp;/g' -e 's/</\&lt;/g' -e 's/>/\&gt;/g' -e 's/"/\&quot;/g'; }
IN="$(esc "$IDENTITY_NAME")" PU="$(esc "$PUBLISHER")" PD="$(esc "$PUBLISHER_DISPLAY_NAME")" VE="$VERSION" DN="$(esc "$DISPLAY_NAME")" \
python3 -c '
import os,sys; t=open("AppxManifest.template.xml",encoding="utf-8").read()
for k,v in [("@IDENTITY_NAME@","IN"),("@PUBLISHER_DISPLAY_NAME@","PD"),("@PUBLISHER@","PU"),("@VERSION@","VE"),("@DISPLAY_NAME@","DN")]: t=t.replace(k,os.environ[v])
open(sys.argv[1],"w",encoding="utf-8").write(t)' "$LAYOUT/AppxManifest.xml"
SAFE="$(printf '%s' "$IDENTITY_NAME" | tr -c 'A-Za-z0-9._-' '_')"
PKG="$PWD/Plume_${VERSION}_x64_${SAFE}.msix"; rm -f "$PKG"
LD_LIBRARY_PATH="$PWD/tools" tools/makemsix pack -d "$LAYOUT" -p "$PKG"
ls -la "$PKG"; sha256sum "$PKG" | tee "$PKG.sha256"
echo "完成: $PKG"
