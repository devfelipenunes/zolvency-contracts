# Tornar Passkey Opcional - Plano de Implementação

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transformar a Passkey (secp256r1) em um campo opcional no contrato `github-identity`, permitindo o mint de identidades sem a necessidade de hardware security, enquanto mantém a funcionalidade para quem desejar usá-la.

**Architecture:** Utilização do tipo `Option<T>` do Soroban/Rust para os campos de passkey e assinatura nas structs de entrada e armazenamento. A lógica de mint validará a assinatura apenas se ambos os campos (chave e assinatura) forem fornecidos.

**Tech Stack:** Rust, Soroban SDK, JavaScript (Stellar SDK para validação).

---

### Task 1: Atualizar Estruturas de Dados e Interface

**Files:**
- Modify: `packages/stellar/contracts/github-identity/src/types.rs`
- Modify: `packages/stellar/contracts/github-identity/src/interface.rs`

- [ ] **Step 1: Alterar `MintParams` e `GithubData` em `types.rs`**

```rust
// Em types.rs
#[contracttype]
#[derive(Clone, Debug)]
pub struct MintParams {
    pub username: String,
    pub external_id: String,
    pub passkey: Option<BytesN<65>>, // Alterado para Option
    pub passkey_signature: Option<BytesN<64>>, // Alterado para Option
    pub contributions: u32,
    pub proof_data: Bytes,
    pub nonce: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GithubData {
    pub username: String,
    pub external_id: String,
    pub contributions: u32,
    pub tier: Tier,
    pub minted_at: u64,
    pub updated_at: u64,
    pub expires_at: u64,
    pub proof_data: Bytes,
    pub passkey: Option<BytesN<65>>, // Alterado para Option
}
```

- [ ] **Step 2: Alterar a interface em `interface.rs`**

```rust
// Em interface.rs
pub trait ZolvencyTokenTrait {
    // ... outras funções
    fn get_owner_passkey(env: Env, token_id: u64) -> Option<BytesN<65>>; // Retorno alterado para Option
}
```

- [ ] **Step 3: Verificar compilação básica**

Execute: `cargo check` (dentro da pasta do contrato)
Expected: Deve falhar em `lib.rs` e `test.rs` devido ao mismatch de tipos, confirmando que as mudanças na interface foram detectadas.

- [ ] **Step 4: Commit**

```bash
git add packages/stellar/contracts/github-identity/src/types.rs packages/stellar/contracts/github-identity/src/interface.rs
git commit -m "contract: make passkey optional in types and interface"
```

---

### Task 2: Implementar Lógica Condicional de Mint

**Files:**
- Modify: `packages/stellar/contracts/github-identity/src/lib.rs`

- [ ] **Step 1: Atualizar a implementação de `get_owner_passkey`**

```rust
// Em lib.rs
    fn get_owner_passkey(env: Env, token_id: u64) -> Option<BytesN<65>> {
        storage::get_token_data(&env, token_id)
            .map(|d| d.passkey)
            .unwrap_or(None)
    }
```

- [ ] **Step 2: Atualizar lógica de validação no `mint`**

```rust
// Em lib.rs dentro da função mint
        #[cfg(not(test))]
        {
            match (params.passkey.clone(), params.passkey_signature.clone()) {
                (Some(pk), Some(sig)) => {
                    let mut msg_bytes = [0u8; 64];
                    let ext_id = params.external_id.clone();
                    ext_id.copy_into_slice(&mut msg_bytes[..ext_id.len() as usize]);
                    let msg_hash = env.crypto().sha256(&Bytes::from_slice(&env, &msg_bytes[..ext_id.len() as usize]));
                    env.crypto().secp256r1_verify(&pk, &msg_hash, &sig);
                },
                (None, None) => {
                    // Pula validação se ambos forem None
                },
                _ => {
                    // Retorna erro se apenas um for fornecido
                    return Err(Error::InvalidSignature);
                }
            }
        }
```

- [ ] **Step 3: Atualizar a persistência do `GithubData` no `mint`**

```rust
// Em lib.rs no final do mint
        let github_data = GithubData {
            // ... outros campos
            passkey: params.passkey, // params.passkey agora é Option
        };
```

- [ ] **Step 4: Verificar compilação**

Execute: `cargo check`
Expected: Deve passar em `lib.rs` (pode ainda falhar em `test.rs`).

- [ ] **Step 5: Commit**

```bash
git add packages/stellar/contracts/github-identity/src/lib.rs
git commit -m "contract: implement conditional passkey validation in mint"
```

---

### Task 3: Atualizar Testes Unitários

**Files:**
- Modify: `packages/stellar/contracts/github-identity/src/test.rs`

- [ ] **Step 1: Atualizar stubs de teste para usar `Some(...)`**

```rust
// Em test.rs
fn stub_passkey(env: &Env) -> Option<BytesN<65>> {
    Some(BytesN::from_array(env, &[1u8; 65]))
}

fn stub_passkey_signature(env: &Env) -> Option<BytesN<64>> {
    Some(BytesN::from_array(env, &[0u8; 64]))
}
```

- [ ] **Step 2: Adicionar novo teste `test_mint_without_passkey`**

```rust
#[test]
fn test_mint_without_passkey() {
    let ctx = setup_test();
    let user = Address::generate(&ctx.env);
    ctx.env.mock_all_auths();

    let params = MintParams {
        username: String::from_str(&ctx.env, "no_passkey"),
        external_id: String::from_str(&ctx.env, "gh_999"),
        passkey: None,
        passkey_signature: None,
        contributions: 100,
        proof_data: Bytes::new(&ctx.env),
        nonce: 0,
    };

    let token_id = ctx.client.mint(&user, &stub_passkey_signature(&ctx.env).unwrap(), &params, &None, &None).unwrap();
    assert_eq!(ctx.client.get_owner_passkey(&token_id), None);
}
```

- [ ] **Step 3: Executar testes unitários**

Run: `cargo test`
Expected: Todos os testes devem passar.

- [ ] **Step 4: Commit**

```bash
git add packages/stellar/contracts/github-identity/src/test.rs
git commit -m "test: update unit tests to support optional passkey"
```

---

### Task 4: Atualizar Integração e Scripts

**Files:**
- Modify: `packages/stellar/contracts/zolvency-registry/src/test.rs`
- Modify: `scripts/validate_final.js`

- [ ] **Step 1: Ajustar teste de integração no Registry**

```rust
// Em packages/stellar/contracts/zolvency-registry/src/test.rs
    // ... dentro do teste
    let params = github_contract::MintParams {
        username: String::from_str(&env, "devfelipenunes"),
        external_id: String::from_str(&env, "gh_123"),
        passkey: None, // Alterado para None
        passkey_signature: None, // Alterado para None
        contributions: 1500u32,
        proof_data: Bytes::new(&env),
        nonce: 0u64,
    };

    github_client.mint(&user, &signature, &params, &None, &None);
```

- [ ] **Step 2: Ajustar script `validate_final.js`**

```javascript
// Em scripts/validate_final.js
        const mintParams = {
            username: "final_validator",
            external_id: "gh_final_001",
            passkey: null, // SDK traduz null para Option::None
            passkey_signature: null, // Adicionar este campo se faltar
            contributions: 3000,
            proof_data: Buffer.alloc(0),
            nonce: 0n
        };
```

- [ ] **Step 3: Executar teste de integração**

Run: `cargo test` (na raiz ou dentro de zolvency-registry)
Expected: Testes de integração passando.

- [ ] **Step 4: Commit Final**

```bash
git add packages/stellar/contracts/zolvency-registry/src/test.rs scripts/validate_final.js
git commit -m "refactor: update registry tests and validation script"
```
