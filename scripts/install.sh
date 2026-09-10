#! /bin/sh
set -e
set -x

PREFIX="$1"
shift || true

# Build the local builder image
docker build -t skrynia-builder:0.1.0 ./builder/

# Install server, admin CLI, and client library
mkdir -p -- "$PREFIX/lib/skrynia"
cp -r -T -- ./src/ "$PREFIX/lib/skrynia/src"

# Install configuration
mkdir -p -- "$PREFIX/share/skrynia"
cp -T -- ./config/skrynia.conf "$PREFIX/share/skrynia/skrynia.conf"

# Install wrapper scripts
mkdir -p -- "$PREFIX/bin"
sed "s#SKRYNIAPREFIX#$PREFIX#g" -- ./scripts/run-server.sh > "$PREFIX/bin/skrynia-server"
chmod +x -- "$PREFIX/bin/skrynia-server"
sed "s#SKRYNIAPREFIX#$PREFIX#g" -- ./scripts/run-admin.sh > "$PREFIX/bin/skrynia"
chmod +x -- "$PREFIX/bin/skrynia"

# Install client library for apps
mkdir -p -- "$PREFIX/share/skrynia/client"
cp -T -- ./src/client.js "$PREFIX/share/skrynia/client/skrynia.js"

# Create skrynia user if it does not exist
id skrynia >/dev/null 2>&1 || useradd --system --no-create-home --shell /usr/sbin/nologin skrynia

# Create state directories with correct ownership.
# Set group to skrynia with setgid so new files/dirs inherit it.
# Mode 0775 allows the skrynia service (running as skrynia user) to write
# directories and files that admin CLI creates as root.
mkdir -p -- /var/lib/skrynia/releases
mkdir -p -- /var/lib/skrynia/storage
mkdir -p -- /var/lib/skrynia/state
chown -R skrynia:skrynia /var/lib/skrynia 2>/dev/null || true
chmod 0775 /var/lib/skrynia 2>/dev/null || true
chmod -R g+rws /var/lib/skrynia 2>/dev/null || true

# Install systemd service
mkdir -p -- /etc/systemd/system/
cp -T -- ./scripts/skrynia.service /etc/systemd/system/skrynia.service
systemctl daemon-reload
systemctl enable skrynia
systemctl restart skrynia
