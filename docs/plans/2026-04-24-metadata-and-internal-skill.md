# Padronização de Metadados e Skill Interna - Plano de Implementação

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Adicionar metadados padronizados aos contratos de reputação e criar uma Skill interna para guiar o desenvolvimento e manutenção do protocolo por agentes de IA.

**Architecture:** Extensão da `ZolvencyTokenTrait` com a função `get_metadata`, retorno de struct `TokenMetadata` e criação de documentação técnica estruturada como Skill em `docs/internal/SKILL.md`.

**Tech Stack:** Rust, Soroban SDK, Markdown (Skill format).

---

### Task 1: Definir Estrutura de Metadados

**Files:**
- Modify: `contracts/github-identity/src/types.rs`
- Modify: `contracts/github-identity/src/interface.rs`

- [x] **Step 1: Adicionar `TokenMetadata` em `types.rs`**

```rust
// Em types.rs
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub version: String,
    pub data_source: String,
}
```

- [x] **Step 2: Atualizar a interface em `interface.rs`**

```rust
// Em interface.rs
pub trait ZolvencyTokenTrait {
    // ... existentes
    fn get_metadata(env: Env) -> TokenMetadata; // Nova função
}
```

- [x] **Step 3: Commit**

```bash
git add contracts/github-identity/src/types.rs contracts/github-identity/src/interface.rs
git commit -m "contract: add TokenMetadata structure to interface"
```

---

### Task 2: Implementar Metadados no GitHub Identity

**Files:**
- Modify: `contracts/github-identity/src/lib.rs`

- [ ] **Step 1: Implementar `get_metadata`**

```rust
// Em lib.rs dentro de impl ZolvencyTokenTrait
    fn get_metadata(env: Env) -> TokenMetadata {
        TokenMetadata {
            name: String::from_str(&env, "Zolvency GitHub Identity"),
            symbol: String::from_str(&env, "ZOLV-GH"),
            version: String::from_str(&env, "1.1.0"), // Versão atualizada com Passkey Opcional
            data_source: String::from_str(&env, "zk-email / github-api"),
        }
    }
```

- [ ] **Step 2: Validar compilação**

Execute: `make build`
Expected: Sucesso.

- [ ] **Step 3: Commit**

```bash
git add contracts/github-identity/src/lib.rs
git commit -m "contract: implement get_metadata in github-identity"
```

---

### Task 3: Criar Skill de Desenvolvimento Interno

**Files:**
- Create: `docs/internal/SKILL.md`

- [ ] **Step 1: Redigir a Skill Interna**

Crie o arquivo com diretrizes sobre:
- Arquitetura Hub & Spoke.
- Padrão de Interoperabilidade Modular.
- Requisitos de Segurança (Passkeys Opcionais).
- Convenções de Código (Rust Workspace).

- [ ] **Step 2: Commit**

```bash
git add docs/internal/SKILL.md
git commit -m "docs: create internal developer skill for AI agents"
```

---

### Task 4: Atualizar Portal de Entrada (README)

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Adicionar referência à Skill no README**

```markdown
## 🤖 Para Agentes de IA
Este repositório inclui uma [Skill de Desenvolvimento Interna](./docs/internal/SKILL.md). Ao trabalhar neste projeto, carregue esta skill para garantir adesão aos padrões arquiteturais e de segurança do Zolvency.
```

- [ ] **Step 2: Commit Final**

```bash
git add README.md
git commit -m "docs: add AI Agent guidance to README"
```
