#!/bin/sh
set -eu
# Reproducible cgroup-limited Rust preview; does not touch production data.
engine=${SKRYNIA_CONTAINER_ENGINE:-podman}
image=${SKRYNIA_CONTAINER_IMAGE:-localhost/skrynia-rust-memory-preview:latest}
port=${SKRYNIA_CONTAINER_PORT:-39877}
root=$(mktemp -d)
container=skrynia-memory-$$
cleanup() {
  "$engine" rm -f "$container" >/dev/null 2>&1 || :
  if [ "$engine" = podman ]; then
    "$engine" unshare chown -R 0:0 "$root" >/dev/null 2>&1 || :
  fi
  rm -rf "$root" "$root.events"
}
trap cleanup EXIT HUP INT TERM
if [ "${SKRYNIA_CONTAINER_SKIP_BUILD:-0}" != 1 ]; then
  TMPDIR=${TMPDIR:-/tmp} "$engine" build -q -f Dockerfile.rust -t "$image" . >/dev/null
fi
if [ "$engine" = podman ]; then
  "$engine" unshare chown -R 1000:1000 "$root"
  set --
else
  # Rootful Docker maps the runner's UID directly to the bind mount.
  set -- --user "$(id -u):$(id -g)"
fi
"$engine" run -d --name "$container" --memory=96m --memory-swap=96m "$@" \
  -v "$root:/var/lib/skrynia" -p "127.0.0.1:$port:17380" \
  -e SKRYNIA_URL="http://127.0.0.1:$port/crash" \
  -e SKRYNIA_MAX_BUFFERED_UPLOAD_BYTES=25165824 \
  "$image" >/dev/null
ready=0
attempt=0
while [ "$attempt" -lt 80 ]; do
  if curl -fsS "http://127.0.0.1:$port/crash/health" >/dev/null 2>&1; then ready=1; break; fi
  sleep 0.15
  attempt=$((attempt+1))
done
if [ "$ready" -ne 1 ]; then "$engine" logs "$container"; exit 1; fi
# Assert the requested cgroup limit exists, not just the command-line flag.
limit=$("$engine" exec "$container" sh -c   'cat /sys/fs/cgroup/memory.max 2>/dev/null || cat /sys/fs/cgroup/memory/memory.limit_in_bytes')
if [ "$limit" != 100663296 ]; then
  if [ "${SKRYNIA_CONTAINER_SKIP_CGROUP_ASSERT:-0}" != 1 ]; then
    printf 'Memory limit not enforced by container runtime: %s (expected 100663296)\n' "$limit" >&2
    exit 1
  fi
  printf 'WARNING: cgroup not enforced; this is an HTTP smoke test ONLY\n' >&2
fi
test "$("$engine" exec "$container" id -u)" = 1000
node tests/rust-container-pressure.js "$root" "$port" "$container"
# Require the kernel to report zero OOM kills during this test.
if "$engine" exec "$container" cat /sys/fs/cgroup/memory.events > "$root.events" 2>/dev/null; then
  if grep -Eq '^oom_kill [1-9]' "$root.events"; then
    cat "$root.events" >&2
    exit 1
  fi
fi
"$engine" stop -t 5 "$container" >/dev/null
if [ "$engine" = podman ]; then
  "$engine" unshare chown -R 0:0 "$root"
fi
# Validate on-disk interoperability after releasing Rust's writer lock.
SKRYNIA_CONTAINER_EXPECTED_ROOT="$root" node tests/rust-container-reopen.js
printf '%s\n' 'Memory-limited container and Node rollback: OK'
