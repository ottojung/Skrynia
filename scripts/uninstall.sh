#! /bin/sh
set -e
set -x

PREFIX="$1"
shift || true

systemctl stop skrynia 2>/dev/null || true
systemctl disable skrynia 2>/dev/null || true
rm -f /etc/systemd/system/skrynia.service
systemctl daemon-reload 2>/dev/null || true

rm -f "$PREFIX/bin/skrynia-server"
rm -f "$PREFIX/bin/skrynia"
rm -rf "$PREFIX/lib/skrynia"
rm -rf "$PREFIX/share/skrynia"
