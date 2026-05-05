DOCKER_IMAGE_NAME=backend:latest
CORE_COVERAGE_IGNORE_REGEX=(src/bin/ws_test.rs|src/main.rs|src/test_utils.rs|src/ws/connection.rs|src/routes/auth.rs|src/routes/gifs.rs|src/services/message_service.rs|src/auth/middleware.rs|src/models/user.rs)

.PHONY: build docker-build up down logs shell coverage coverage-core

build:
	cargo build --release

docker-build:
	docker build -t $(DOCKER_IMAGE_NAME) .

up:
	docker-compose up --build -d

down:
	docker-compose down

logs:
	docker-compose logs -f

shell:
	docker-compose exec backend /bin/bash

coverage:
	EPITALK_SKIP_SLOW_TESTS=1 cargo llvm-cov --summary-only

coverage-core:
	EPITALK_SKIP_SLOW_TESTS=1 cargo llvm-cov --summary-only --ignore-filename-regex '$(CORE_COVERAGE_IGNORE_REGEX)'
