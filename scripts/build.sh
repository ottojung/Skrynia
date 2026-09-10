#! /bin/sh
set -e
set -x

DISTDIR="$1"
shift || true

mkdir -p "$DISTDIR"
touch "$DISTDIR/built"
