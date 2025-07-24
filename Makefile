PLUGINS_DIR := .plugins
PLUGINS_DIR_SRC := $(PLUGINS_DIR)/src

BUILDER_VERSION := v1.0.9

CONFIGS_DIR := configs
IMAGES_DIR := images

IMAGES := $(patsubst $(CONFIGS_DIR)/%.pkrvars.hcl,$(IMAGES_DIR)/%.img,$(CONFIGS))

.PHONY: dirs clean init image provision-bin provision

dirs:
	@mkdir -p $(PLUGINS_DIR) $(PLUGINS_DIR_SRC) $(CONFIGS_DIR) $(IMAGES_DIR)

clean:
	@rm -rf $(PLUGINS_DIR)

init:
	@PACKER_PLUGIN_PATH=$(PLUGINS_DIR) packer init packer

image: init | dirs
	echo "Building image..."
	sudo PACKER_PLUGIN_PATH=$(PLUGINS_DIR) DONT_SETUP_QEMU=1 packer build -var=image_dir=$(IMAGES_DIR) packer; \

provision-bin:
	cd provision && cargo build

provision: provision-bin
	sudo ./provision/target/debug/provision ./configs/workspace.json ./configs/instances
