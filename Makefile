.PHONY: build test fmt lint clean deploy-testnet test-scripts

# Configurações
WASM_IDENTITY=contracts/github/target/wasm32-unknown-unknown/release/github_identity.wasm
WASM_REGISTRY=contracts/nexus/target/wasm32-unknown-unknown/release/nexus.wasm

build:
	@echo "🔨 Building contracts..."
	RUSTFLAGS="-C target-feature=-sign-ext -C target-feature=-mutable-globals -C target-feature=-reference-types -C target-feature=-bulk-memory" stellar contract build

test:
	@echo "🧪 Running tests..."
	cargo test

test-scripts:
	@echo "🧪 Running testnet validation scripts..."
	cd .. && bash contracts/scripts/perfect_e2e_simulation.sh
	cd .. && bash contracts/scripts/zpay_negative_cases.sh
	cd .. && bash contracts/scripts/nexus_authority_cases.sh
	cd .. && bash contracts/scripts/soul_negative_cases.sh

fmt:
	@echo "🎨 Formatting code..."
	cargo fmt --all

.PHONY: lint
lint:
	@echo "🧹 Running Clippy static analysis..."
	cargo clippy --all-targets --all-features -- -D warnings

clean:
	@echo "🧹 Cleaning targets..."
	cargo clean

.PHONY: audit
audit: audit-rust audit-evm

.PHONY: audit-evm
audit-evm:
	@echo "🔍 Running Slither security audit..."
	slither verifiers/evm/ --config-file slither.config.json

.PHONY: audit-rust
audit-rust:
	@echo "🔍 Checking Rust dependencies for vulnerabilities..."
	cargo audit

# Helper para rodar a automação completa
deploy-testnet: build
	@echo "🚀 Starting testnet automation..."
	./scripts/testnet_automation.sh
