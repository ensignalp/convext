#!/usr/bin/env bash
set -euo pipefail

echo "[*] Convext kaldırılıyor..."

if command -v systemctl >/dev/null 2>&1; then
    systemctl --user disable --now convext.service >/dev/null 2>&1 || true
fi

pkill -x "convext" >/dev/null 2>&1 || true
pkill -x "citrus-daemon" >/dev/null 2>&1 || true

rm -f "${HOME}/.local/bin/convext"
rm -f "${HOME}/.local/bin/citrus-daemon"
rm -f "${HOME}/.local/share/nautilus-python/extensions/convext_menu.py"
rm -f "${HOME}/.local/share/nautilus-python/extensions/citrus_selection_bridge.py"
rm -f "${HOME}/.config/systemd/user/convext.service"
rm -f "${HOME}/.config/autostart/convext.desktop"
rm -f "${HOME}/.config/autostart/convext-daemon.desktop"

if command -v nautilus >/dev/null 2>&1; then
    nautilus -q >/dev/null 2>&1 || true
fi

echo "[✓] Convext sistemden tamamen kaldırıldı."
