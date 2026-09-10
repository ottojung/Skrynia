#! /bin/sh
set -e

PREFIX="SKRYNIAPREFIX"
CONF="$PREFIX/share/skrynia/skrynia.conf"

if [ -f "$CONF" ]; then
    . "$CONF"
fi

export SKRYNIA_PREFIX="$PREFIX"
exec node "$PREFIX/lib/skrynia/src/server.js" "$@"
