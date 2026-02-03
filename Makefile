# ══════════════════════════════════════════════════════════════════════════════
# Fogo Locker
# ══════════════════════════════════════════════════════════════════════════════

PROGRAM_ID_localnet := 76Hqr9nixY17jLcLekWAnmnAQLdSYS9DuU3XErVfETi3
PROGRAM_ID_testnet  := LockvXm2nWht6EvHf44AmCuS3eMKRiWTuks2x27XRRo
PROGRAM_ID_staging  := 76Hqr9nixY17jLcLekWAnmnAQLdSYS9DuU3XErVfETi3
PROGRAM_ID_mainnet  := LockvXm2nWht6EvHf44AmCuS3eMKRiWTuks2x27XRRo

PROGRAM_SO := target/deploy/locker.so

CLUSTER ?= localnet
RPC_localnet := http://localhost:8899
RPC_testnet  := https://testnet.fogo.io
RPC_mainnet  := https://mainnet.fogo.io

KEYPAIR_DIR := .keys
PROGRAM_ID  := $(PROGRAM_ID_$(CLUSTER))

.DEFAULT_GOAL := help

# ══════════════════════════════════════════════════════════════════════════════
# Build
# ══════════════════════════════════════════════════════════════════════════════

build: ## Build on-chain program (CLUSTER=localnet|testnet|mainnet)
	$(MAKE) build/$(CLUSTER)

build/localnet: ## Build for localnet
	anchor build -p locker --no-idl -- --features localnet

build/testnet: ## Build for testnet
	anchor build -p locker
	@cp target/idl/locker.json ./sdk/artifacts/

build/mainnet: ## Build for mainnet (no feature flag)
	anchor build -p locker
	@cp target/idl/locker.json ./sdk/artifacts/

# ══════════════════════════════════════════════════════════════════════════════
# Test
# ══════════════════════════════════════════════════════════════════════════════

test: build/localnet ## Run all tests
	anchor test --skip-build

test/unit: ## Run unit tests only
	cargo test --lib -p locker

# ══════════════════════════════════════════════════════════════════════════════
# Code Quality
# ══════════════════════════════════════════════════════════════════════════════

fmt: ## Format Rust code
	cargo fmt --all

fmt/check: ## Check Rust formatting
	cargo fmt --all --check

lint: ## Run clippy
	cargo clippy --all-targets -- -D warnings

audit: ## Run security audit
	cargo audit

# ══════════════════════════════════════════════════════════════════════════════
# Deploy
# ══════════════════════════════════════════════════════════════════════════════

deploy: _check-cluster ## Deploy program (CLUSTER=localnet|testnet|mainnet)
ifeq ($(CLUSTER),mainnet)
	@printf "\033[31mDeploy to MAINNET? [y/N] \033[0m" && read ans && [ $${ans:-N} = y ]
endif
	$(MAKE) build/$(CLUSTER)
	anchor deploy -p locker --provider.cluster $(RPC_$(CLUSTER))

upgrade: _check-cluster ## Upgrade program (CLUSTER=localnet|testnet|mainnet)
ifeq ($(CLUSTER),mainnet)
	@printf "\033[31mUpgrade on MAINNET? [y/N] \033[0m" && read ans && [ $${ans:-N} = y ]
endif
	$(MAKE) build/$(CLUSTER)
	anchor upgrade -p locker --provider.cluster $(RPC_$(CLUSTER)) $(PROGRAM_SO)

# ══════════════════════════════════════════════════════════════════════════════
# Utilities
# ══════════════════════════════════════════════════════════════════════════════

help: ## Show this help
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z\/_-]+:.*?## / {printf "  \033[32m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

clean: ## Remove build artifacts
	cargo clean
	rm -rf target/deploy

size: build ## Show program size
	@ls -lh $(PROGRAM_SO) | awk '{printf "Program size: %s\n", $$5}'

show: _check-cluster ## Show program info on cluster
	@solana program show $(PROGRAM_ID) --url $(RPC_$(CLUSTER)) 2>/dev/null || echo "Program not deployed on $(CLUSTER)"

_check-keypair:
	@test -f $(KEYPAIR_DIR)/$(PROGRAM_ID).json || (echo "Error: $(KEYPAIR_DIR)/$(PROGRAM_ID).json not found" && exit 1)

_check-cluster:
	@test -n "$(RPC_$(CLUSTER))" || (echo "Error: Invalid CLUSTER '$(CLUSTER)'" && exit 1)
