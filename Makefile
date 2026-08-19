.PHONY: build test fmt lint clean deploy e2e test-security audit

# Configurações
WASM_DIR=target/wasm32v1-none/release

build:
	@echo "🔨 Building contracts..."
	stellar contract build --optimize

test:
	@echo "🧪 Running Rust unit tests..."
	cargo test --workspace

e2e:
	@echo "🚀 Running End-to-End simulation on Testnet..."
	./scripts/e2e.sh

deploy: build
	@echo "🚀 Deploying core system to Testnet..."
	./scripts/deploy.sh

test-security:
	@echo "🧪 Running security and negative test cases..."
	./scripts/security/zpay_negative_cases.sh
	./scripts/security/nexus_authority_cases.sh
	./scripts/security/soul_negative_cases.sh

fmt:
	@echo "🎨 Formatting code..."
	cargo fmt --all

lint:
	@echo "🧹 Running Clippy static analysis..."
	cargo clippy --all-targets --all-features -- -D warnings

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

audit:
	@echo "🔍 Checking Rust dependencies for vulnerabilities..."
	cargo audit || echo "Cargo audit not found or failed"
