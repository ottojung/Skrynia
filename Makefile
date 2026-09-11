
BUILDER_IMAGE = skrynia-builder:0.1.0

builder: builder/Dockerfile
	docker build -t $(BUILDER_IMAGE) builder/

test:
	node tests/test.js
	node tests/regression.js
	node tests/docker-invocation.js

.PHONY: builder test
