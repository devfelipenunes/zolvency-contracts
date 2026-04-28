.PHONY: build test fmt lint clean deploy-testnet

# Configurações
WASM_IDENTITY=packages/stellar/contracts/github-identity/target/wasm32-unknown-unknown/release/github_identity.wasm
WASM_REGISTRY=packages/stellar/contracts/zolvency-registry/target/wasm32-unknown-unknown/release/zolvency_registry.wasm

build:
	@echo "🔨 Building contracts..."
	RUSTFLAGS="-C target-feature=-sign-ext -C target-feature=-mutable-globals -C target-feature=-reference-types -C target-feature=-bulk-memory" stellar contract build

test:
	@echo "🧪 Running tests..."
	cargo test

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
	slither packages/evm/ --config-file slither.config.json

.PHONY: audit-rust
audit-rust:
	@echo "🔍 Checking Rust dependencies for vulnerabilities..."
	cargo audit

# Helper para rodar a automação completa
deploy-testnet: build
	@echo "🚀 Starting testnet automation..."
	./scripts/testnet_automation.sh
