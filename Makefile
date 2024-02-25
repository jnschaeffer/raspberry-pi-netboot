PLUGINS_DIR := .plugins
PLUGINS_DIR_SRC := $(PLUGINS_DIR)/src

BUILDER_VERSION := v1.0.9

CONFIGS_DIR := configs
IMAGES_DIR := images

CONFIGS := $(wildcard $(CONFIGS_DIR)/*.pkrvars.hcl)
IMAGES := $(patsubst $(CONFIGS_DIR)/%.pkrvars.hcl,$(IMAGES_DIR)/%.img,$(CONFIGS))

.PHONY: build clean dirs images

dirs:
	@mkdir -p $(PLUGINS_DIR) $(PLUGINS_DIR_SRC) $(CONFIGS_DIR) $(IMAGES_DIR)

$(IMAGES): $(CONFIGS) *.pkr.hcl | dirs
	@for config in $(CONFIGS); do \
		echo "Building image for $$config..."; \
		sudo PACKER_PLUGIN_PATH=$(PLUGINS_DIR) DONT_SETUP_QEMU=1 packer build -var=image_dir=$(IMAGES_DIR) -var-file="$$config" .; \
	done

images: $(IMAGES)

build: $(PLUGINS_DIR)/packer-builder-arm
	@sudo PACKER_PLUGIN_PATH=$(PLUGINS_DIR) ./build.sh

clean:
	@rm -rf $(PLUGINS_DIR)

$(PLUGINS_DIR)/packer-builder-arm: | dirs
	cd $(PLUGINS_DIR_SRC) && \
	git clone git@github.com:mkaczanowski/packer-builder-arm.git && \
	cd packer-builder-arm && \
	go build -o $(PLUGINS_DIR)
