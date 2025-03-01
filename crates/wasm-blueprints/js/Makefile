.DEFAULT_GOAL = help
TARGET = module.wasm
.PHONY: help test
DOCKER_IMAGE = yozefu-wasm-blueprints-js

build: ## Try to build the wasm module. If it fails, it tries to build with Docker
	make -S $(TARGET) || make -S build-from-docker

build-from-docker: ## Build the wasm module with Docker
	@echo " 🐋 Trying to build wasm module with docker"
	docker build -t $(DOCKER_IMAGE) .
	docker create --name $(DOCKER_IMAGE) $(DOCKER_IMAGE)
	docker cp $(DOCKER_IMAGE):/tmp/build/dist/plugin.wasm $(TARGET)
	docker rm -v $(DOCKER_IMAGE)

$(TARGET): ## Build the wasm module
	npm install
	npm run build
	
test: $(TARGET) ## Run the tests
	cat "tests/parameters.json" | extism call --wasi --stdin $(TARGET) parse_parameters
	cat "tests/match.json" | extism call --stdin $(TARGET) matches | grep -q '"match":true'
	cat "tests/no-match.json" | extism call --stdin $(TARGET) matches | grep -q '"match":false'

clean: ## Clean the wasm file and the target directory
	rm -rf dist

help: ## Show this help
	@echo "Variables:"
	@make -pnf $(MAKEFILE_LIST) | awk '/^# (makefile |command)/{getline; print}' | grep -v "^MAKEFILE_LIST" | sort | uniq | awk 'BEGIN {FS = ":?= "}; {printf "  \033[36m%-30s\033[0m %s\n", $$1, $$2}'
	@echo "\nTargets:"
	@grep -E '^[/%a-zA-Z0-9_-]+: .*?## .*$$' $(MAKEFILE_LIST) | sort | awk  'BEGIN {FS = ": .*?## "}; {printf "  \033[36m%-30s\033[0m %s\n", $$1, $$2}'
