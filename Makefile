PREFIX = /usr/local
BUILDER_IMAGE = skrynia-builder:0.1.0

install: scripts/install.sh builder/Dockerfile
	sh scripts/install.sh "$(PREFIX)"

uninstall: scripts/uninstall.sh
	sh scripts/uninstall.sh "$(PREFIX)"

builder: builder/Dockerfile
	docker build -t $(BUILDER_IMAGE) builder/

test:
	node tests/test.js

.PHONY: install uninstall builder test
