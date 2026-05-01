# Armor Up: Automated Security Hardening - Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implementar auditoria automática de segurança (Slither, Cargo Audit, Clippy) e integrá-la ao CI e Makefile.

**Architecture:** Abordagem de "Defense in Depth" com análise estática no Solidity e Rust, bloqueando falhas de segurança no GitHub Actions.

**Tech Stack:** Slither, Cargo Audit, Cargo Clippy, GitHub Actions, Makefile.

---

### Task 1: Configurar Slither para Solidity

**Files:**
- Create: `slither.config.json`
- Modify: `Makefile`

- [ ] **Step 1: Criar arquivo de configuração do Slither**

```json
{
  "detectors_to_exclude": "naming-convention,solc-version,low-level-calls",
  "filter_paths": "lib",
  "solc_remaps": [
    "@axelar-network/axelar-gmp-sdk-solidity/=lib/axelar-gmp-sdk-solidity/",
    "@layerzerolabs/oapp-evm/=lib/layerzero-v2/packages/layerzero-v2/evm/oapp/",
    "@openzeppelin/contracts/=lib/openzeppelin-contracts/contracts/",
    "forge-std/=lib/forge-std/src/"
  ]
}
```

- [ ] **Step 2: Adicionar comando de auditoria EVM ao Makefile**

```makefile
.PHONY: audit-evm
audit-evm:
	@echo "🔍 Running Slither security audit..."
	slither verifiers/evm/ --config-file slither.config.json
```

- [ ] **Step 3: Commit**

```bash
git add slither.config.json Makefile
git commit -m "chore: setup slither configuration for evm security audit"
```

---

### Task 2: Configurar Auditoria de Dependências Rust

**Files:**
- Modify: `Makefile`

- [ ] **Step 1: Adicionar comando de auditoria Rust ao Makefile**

```makefile
.PHONY: audit-rust
audit-rust:
	@echo "🔍 Checking Rust dependencies for vulnerabilities..."
	cargo audit
```

- [ ] **Step 2: Commit**

```bash
git add Makefile
git commit -m "chore: add cargo audit to makefile for dependency security"
```

---

### Task 3: Configurar Clippy para Soroban

**Files:**
- Modify: `Makefile`

- [ ] **Step 1: Adicionar comando de lint/clippy ao Makefile**

```makefile
.PHONY: lint
lint:
	@echo "🧹 Running Clippy static analysis..."
	cargo clippy --all-targets --all-features -- -D warnings
```

- [ ] **Step 2: Atualizar o alvo `audit` para rodar tudo**

```makefile
.PHONY: audit
audit: audit-rust audit-evm
```

- [ ] **Step 3: Commit**

```bash
git add Makefile
git commit -m "chore: unify security and linting commands in makefile"
```

---

### Task 4: Atualizar GitHub Actions (CI)

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Adicionar jobs de segurança ao workflow**

```yaml
  security:
    name: Security Hardening
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
          components: clippy
      
      - name: Install cargo-audit
        run: cargo install cargo-audit

      - name: Run Cargo Audit
        run: cargo audit

      - name: Run Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.10'

      - name: Install Slither
        run: |
          pip3 install slither-analyzer
          
      - name: Run Slither
        run: slither verifiers/evm/ --config-file slither.config.json
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: integrate slither, cargo-audit and clippy into pipeline"
```

---

### Task 5: Sincronização de Documentação

**Files:**
- Modify: `docs/product/RISK_MATRIX.md`

- [ ] **Step 1: Atualizar mitigação de riscos técnicos**

```markdown
| **Oráculo Malicioso** | Massivo | Baixa | Curadoria rigorosa e auditoria automática (Slither/Cargo Audit). |
```

- [ ] **Step 2: Commit**

```bash
git add docs/product/RISK_MATRIX.md
git commit -m "docs: reflect automated security in risk matrix"
```
