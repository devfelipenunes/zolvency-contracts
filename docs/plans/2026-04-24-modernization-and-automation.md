# Modernização de Infraestrutura e Automação - Plano de Implementação

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implementar automação via Makefile, pipeline de CI/CD via GitHub Actions, refatorar scripts para remover hardcoding e adicionar capacidade de upgrade aos contratos.

**Architecture:** Centralização de comandos no Makefile, isolamento de segredos/IDs no `.env`, automação de testes no GitHub Actions e implementação do padrão de upgrade nativo do Soroban.

**Tech Stack:** Rust, Soroban, GitHub Actions, Makefile, JavaScript.

---

### Task 1: Pipeline de CI (GitHub Actions)

**Files:**
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Criar o arquivo de workflow**

```yaml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  rust-checks:
    name: Rust Checks (Lint & Test)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
          targets: wasm32-unknown-unknown
          components: rustfmt, clippy

      - name: Rust Cache
        uses: Swatinem/rust-cache@v2

      - name: Check Formatting
        run: cargo fmt --all -- --check

      - name: Lint with Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Run Tests
        run: cargo test --all-features
```

- [ ] **Step 2: Validar sintaxe localmente (se possível)**

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add github actions pipeline for rust checks and tests"
```

---

### Task 2: Centralização de Comandos (Makefile)

**Files:**
- Create: `Makefile`

- [ ] **Step 1: Criar o Makefile com comandos essenciais**

```makefile
.PHONY: build test fmt lint clean deploy-testnet

# Configurações
WASM_IDENTITY=contracts/github-identity/target/wasm32-unknown-unknown/release/github_identity.wasm
WASM_REGISTRY=contracts/zolvency-registry/target/wasm32-unknown-unknown/release/zolvency_registry.wasm

build:
	@echo "🔨 Building contracts..."
	cargo build --target wasm32-unknown-unknown --release

test:
	@echo "🧪 Running tests..."
	cargo test

fmt:
	@echo "🎨 Formatting code..."
	cargo fmt --all

lint:
	@echo "🔍 Running linter..."
	cargo clippy --all-targets --all-features -- -D warnings

clean:
	@echo "🧹 Cleaning targets..."
	cargo clean

# Helper para rodar a automação completa
deploy-testnet: build
	@echo "🚀 Starting testnet automation..."
	./scripts/testnet_automation.sh
```

- [ ] **Step 2: Testar comando `make build`**

Run: `make build`
Expected: Sucesso na compilação.

- [ ] **Step 3: Commit**

```bash
git add Makefile
git commit -m "chore: add Makefile for unified command management"
```

---

### Task 3: Refatoração de Scripts (Remover Hardcoding)

**Files:**
- Modify: `scripts/validate_final.js`
- Modify: `scripts/testnet_automation.sh`
- Modify: `.env.example`

- [ ] **Step 1: Atualizar `.env.example`**

```text
# Adicionar placeholders para os novos IDs de contrato
GITHUB_IDENTITY_ID=
ZOLVENCY_REGISTRY_ID=
ADMIN_SECRET=
DEPLOYER_SECRET=
# ... resto existente
```

- [ ] **Step 2: Refatorar `validate_final.js` para usar `process.env`**

```javascript
// Em scripts/validate_final.js
const identityId = process.env.GITHUB_IDENTITY_ID;
const registryId = process.env.ZOLVENCY_REGISTRY_ID;

if (!identityId || !registryId) {
    console.error("❌ Erro: GITHUB_IDENTITY_ID ou ZOLVENCY_REGISTRY_ID não definidos no .env");
    process.exit(1);
}
```

- [ ] **Step 3: Commit**

```bash
git add scripts/ .env.example
git commit -m "refactor: remove hardcoded contract IDs from scripts"
```

---

### Task 4: Suporte a Upgrades (github-identity)

**Files:**
- Modify: `contracts/github-identity/src/lib.rs`

- [ ] **Step 1: Adicionar função de upgrade**

```rust
// Em lib.rs dentro de impl GithubIdentityContract
    pub fn upgrade(env: Env, admin: Address, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;

        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }
```

- [ ] **Step 2: Verificar compilação**

Run: `make build`
Expected: Sucesso.

- [ ] **Step 3: Commit Final**

```bash
git add contracts/github-identity/src/lib.rs
git commit -m "feat: add upgrade functionality to github-identity contract"
```
