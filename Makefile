PREFIX = /usr/local

install: scripts/install.sh
	sh scripts/install.sh "$(PREFIX)"

uninstall: scripts/uninstall.sh
	sh scripts/uninstall.sh "$(PREFIX)"

test:
	node tests/test.js

.PHONY: install uninstall test
