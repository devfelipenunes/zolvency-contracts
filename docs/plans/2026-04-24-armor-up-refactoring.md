# Armor Up: Correções de Vulnerabilidades e Robustez - Plano de Implementação

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Corrigir fragilidades de governança (Two-Step Admin), implementar Reputation Lock/Slashing no Registry, integrar validação de provas ZK (Interface), adicionar gestão de estado (TTL Renewal) e tornar os scripts de deploy idempotentes.

**Architecture:** Adição de Two-Step Admin e funções de Lock/Slashing no `Nexus`. No `GithubIdentityContract`, integração de um verificador ZK externo opcional durante o `mint` e adição de função pública de renovação de TTL. Refatoração do script bash de deploy.

**Tech Stack:** Rust, Soroban SDK, Bash.

---

### Task 1: Governança em Duas Etapas (Two-Step Admin Transfer)

**Files:**
- Modify: `contracts/zolvency-registry/src/lib.rs`

- [ ] **Step 1: Adicionar novos DataKeys e atualizar inicialização**

```rust
// Em contracts/zolvency-registry/src/lib.rs
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    PendingAdmin, // Novo
    Signer,
    Tokens,
    Locks(Address), // Adicionado da Task 2 para consistência
    Blacklist(Address), // Adicionado da Task 2 para consistência
}
```

- [ ] **Step 2: Implementar `transfer_admin` e `accept_admin`**

```rust
// Dentro do impl Nexus
    pub fn transfer_admin(env: Env, admin: Address, new_admin: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Not admin");
        }
        env.storage().persistent().set(&DataKey::PendingAdmin, &new_admin);
    }

    pub fn accept_admin(env: Env, new_admin: Address) {
        new_admin.require_auth();
        let pending_admin: Address = env.storage().persistent().get(&DataKey::PendingAdmin).unwrap();
        if new_admin != pending_admin {
            panic!("Not pending admin");
        }
        env.storage().persistent().set(&DataKey::Admin, &new_admin);
        env.storage().persistent().remove(&DataKey::PendingAdmin);
    }
```

- [ ] **Step 3: Verificar compilação**

Run: `cargo check` em `contracts/zolvency-registry`.

- [ ] **Step 4: Commit**

```bash
git add contracts/zolvency-registry/src/lib.rs
git commit -m "feat(registry): implement two-step admin transfer for safer governance"
```

---

### Task 2: Reputation Lock & Slashing Mechanics

**Files:**
- Modify: `contracts/zolvency-registry/src/lib.rs`

- [ ] **Step 1: Implementar funções de Lock e Slashing**

```rust
// Dentro do impl Nexus
    pub fn lock_reputation(env: Env, caller: Address, user: Address, unlock_timestamp: u64) {
        // Nota: Em produção, 'caller' seria verificado contra uma lista de protocolos autorizados.
        caller.require_auth(); 
        
        let key = DataKey::Locks(user.clone());
        env.storage().persistent().set(&key, &unlock_timestamp);
    }

    pub fn is_locked(env: Env, user: Address) -> bool {
        let key = DataKey::Locks(user);
        if let Some(unlock_timestamp) = env.storage().persistent().get::<_, u64>(&key) {
            env.ledger().timestamp() < unlock_timestamp
        } else {
            false
        }
    }

    pub fn apply_slashing(env: Env, admin: Address, user: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Not admin");
        }
        let key = DataKey::Blacklist(user.clone());
        env.storage().persistent().set(&key, &true);
    }

    pub fn is_blacklisted(env: Env, user: Address) -> bool {
        let key = DataKey::Blacklist(user);
        env.storage().persistent().get(&key).unwrap_or(false)
    }
```

- [ ] **Step 2: Modificar `get_user_reputation` para respeitar o Blacklist**

```rust
// No início de get_user_reputation:
        if Self::is_blacklisted(env.clone(), user.clone()) {
            return Map::new(&env); // Retorna vazio se estiver na blacklist
        }
```

- [ ] **Step 3: Verificar compilação**

Run: `cargo check` em `contracts/zolvency-registry`.

- [ ] **Step 4: Commit**

```bash
git add contracts/zolvency-registry/src/lib.rs
git commit -m "feat(registry): implement reputation lock and slashing mechanics"
```

---

### Task 3: Gestão de Estado (TTL Renewal)

**Files:**
- Modify: `contracts/github-identity/src/lib.rs`
- Modify: `contracts/github-identity/src/storage.rs`

- [ ] **Step 1: Adicionar função em `storage.rs` para renovar TTL**

```rust
// Em contracts/github-identity/src/storage.rs
pub fn extend_token_ttl(env: &Env, token_id: u64) -> Result<(), Error> {
    let key = (Symbol::new(env, "TOK"), token_id);
    if !env.storage().persistent().has(&key) {
        return Err(Error::TokenNotFound);
    }
    env.storage().persistent().extend_ttl(&key, ONE_YEAR, ONE_YEAR);
    Ok(())
}
```

- [ ] **Step 2: Expor função pública em `lib.rs`**

```rust
// Em contracts/github-identity/src/lib.rs, dentro de impl GithubIdentityContract
    pub fn renew_token_ttl(env: Env, token_id: u64) -> Result<(), Error> {
        // Qualquer um pode chamar, sem require_auth, pois estão pagando a taxa da rede
        storage::extend_token_ttl(&env, token_id)
    }
```

- [ ] **Step 3: Commit**

```bash
git add contracts/github-identity/src/lib.rs contracts/github-identity/src/storage.rs
git commit -m "feat(identity): add public function to renew token TTL state"
```

---

### Task 4: Integração de Verificador ZK Externo (Interface)

**Files:**
- Modify: `contracts/github-identity/src/types.rs`
- Modify: `contracts/github-identity/src/lib.rs`

- [ ] **Step 1: Atualizar Config em `types.rs`**

```rust
// Em contracts/github-identity/src/types.rs, adicionar à struct Config
pub struct Config {
    pub admin: soroban_sdk::Address,
    pub registry: soroban_sdk::Address,
    pub fee_token: soroban_sdk::Address,
    pub access_control: soroban_sdk::Address,
    pub treasury: soroban_sdk::Address,
    pub mint_fee: i128,
    pub zk_verifier: Option<soroban_sdk::Address>, // Novo
}
```

- [ ] **Step 2: Atualizar `initialize` e adicionar `set_zk_verifier` em `lib.rs`**

```rust
// Em contracts/github-identity/src/lib.rs
    pub fn initialize(
        env: Env,
        admin: Address,
        registry: Address,
        fee_token: Address,
        access_control: Address,
        treasury: Address,
        mint_fee: i128,
    ) -> Result<(), Error> {
        // ...
        let config = types::Config {
            admin,
            registry,
            fee_token,
            access_control,
            treasury,
            mint_fee,
            zk_verifier: None, // Inicializa como None
        };
        // ...
    }

    pub fn set_zk_verifier(env: Env, admin: Address, verifier: Option<Address>) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;
        let mut config = storage::get_config(&env)?;
        config.zk_verifier = verifier;
        storage::set_config(&env, &config);
        Ok(())
    }
```

- [ ] **Step 3: Invocar verificação ZK no `mint`**

```rust
// Em contracts/github-identity/src/lib.rs, na função mint
        let config = storage::get_config(&env)?;
        if let Some(verifier) = config.zk_verifier {
            let is_valid: bool = env.invoke_contract(
                &verifier,
                &Symbol::new(&env, "verify_proof"),
                Vec::from_array(&env, [params.proof_data.clone().into_val(&env)]),
            );
            if !is_valid {
                return Err(Error::Unauthorized); // Usando erro existente para simplificar
            }
        }
```

- [ ] **Step 4: Commit**

```bash
git add contracts/github-identity/src/
git commit -m "feat(identity): integrate external ZK verifier hook in mint process"
```

---

### Task 5: Scripts Idempotentes (Deploy Inteligente)

**Files:**
- Modify: `scripts/testnet_automation.sh`

- [ ] **Step 1: Refatorar script para verificar existência de IDs**

```bash
# Em scripts/testnet_automation.sh

# 1. Deploy Zolvency Registry
if [ -z "$ZOLVENCY_REGISTRY_ID" ]; then
    echo "📦 Deploying Zolvency Registry..."
    REGISTRY_WASM="contracts/zolvency-registry/target/wasm32-unknown-unknown/release/zolvency_registry.wasm"
    REGISTRY_ID=$($STELLAR_CLI contract deploy --wasm "$REGISTRY_WASM" --source "$DEPLOYER_SECRET" --network testnet)
    echo "✅ Registry deployed: $REGISTRY_ID"
else
    echo "🔄 Using existing Registry: $ZOLVENCY_REGISTRY_ID"
    REGISTRY_ID=$ZOLVENCY_REGISTRY_ID
fi

# 2. Deploy Github Identity
if [ -z "$GITHUB_IDENTITY_ID" ]; then
    echo "🆔 Deploying Github Identity..."
    IDENTITY_WASM="contracts/github-identity/target/wasm32-unknown-unknown/release/github_identity.wasm"
    IDENTITY_ID=$($STELLAR_CLI contract deploy --wasm "$IDENTITY_WASM" --source "$DEPLOYER_SECRET" --network testnet)
    echo "✅ Identity deployed: $IDENTITY_ID"
else
    echo "🔄 Using existing Identity: $GITHUB_IDENTITY_ID"
    IDENTITY_ID=$GITHUB_IDENTITY_ID
fi
```

- [ ] **Step 2: Commit Final**

```bash
git add scripts/testnet_automation.sh
git commit -m "chore: make deploy script idempotent based on env vars"
```
