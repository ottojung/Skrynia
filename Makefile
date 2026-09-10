PREFIX = /usr/local

install: build scripts/install.sh
	sh scripts/install.sh "$(PREFIX)"

uninstall: scripts/uninstall.sh
	sh scripts/uninstall.sh "$(PREFIX)"

build: dist/built

dist/built: scripts/build.sh
	sh scripts/build.sh "$@"

test:
	node tests/test.js

.PHONY: install uninstall build test
.SECONDARY:
