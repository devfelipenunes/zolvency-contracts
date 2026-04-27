# Zolvency Operations Cheatsheet

Guia rápido de comandos para desenvolvimento, deploy e interação com os contratos do Zolvency Protocol.

## 🛠️ Comandos Globais (Makefile)

Utilize o `Makefile` na raiz para operações padronizadas:
```bash
make build    # Compila todos os contratos (release)
make test     # Executa todos os testes unitários
make fmt      # Formata o código Rust
make lint     # Executa Clippy (linter)
make clean    # Limpa artefatos de build
```

---

## 🚀 Deploy e Inicialização (Testnet)

### 1. Deploy do Registry (Hub)
```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/zolvency_registry.wasm \
  --source admin \
  --network testnet
```

### 2. Inicializar Registry
```bash
stellar contract invoke --id <REGISTRY_ID> --source admin --network testnet -- \
  initialize --admin <ADMIN_ADDR> --signer <SIGNER_ADDR>
```

### 3. Deploy do Spoke (GitHub Identity)
```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/github_identity.wasm \
  --source admin \
  --network testnet
```

### 4. Inicializar GitHub Identity
```bash
stellar contract invoke --id <GITHUB_ID> --source admin --network testnet -- \
  initialize \
  --admin <ADMIN_ADDR> \
  --registry <REGISTRY_ID> \
  --fee_token <XLM_ADDR> \
  --access_control <AC_ADDR> \
  --treasury <TREASURY_ADDR> \
  --mint_fee 0 \
  --zk_verifier null  # Opcional: Endereço do contrato verificador ZK
```

---

## 💎 Operações de Usuário (Mint)

### Mint de Identidade (Sem Passkey)
```bash
stellar contract invoke --id <GITHUB_ID> --source user --network testnet -- \
  mint \
  --caller <USER_ADDR> \
  --signature <DUMMY_SIG_64_BYTES> \
  --params '{
    "username": "dev_user",
    "external_id": "gh_12345",
    "passkey": null,
    "passkey_signature": null,
    "contributions": 1500,
    "proof_data": "",
    "nonce": 0
  }'
```

---

## 🔗 Interoperabilidade (Axelar)

### Configurar Adaptador na Identidade
```bash
stellar contract invoke --id <GITHUB_ID> --source admin --network testnet -- \
  set_axelar_config \
  --admin <ADMIN_ADDR> \
  --gateway <GATEWAY_ADDR> \
  --gas_service <GAS_SERVICE_ADDR> \
  --gas_token <GAS_TOKEN_ADDR>
```

### Ativar Protocolo
```bash
stellar contract invoke --id <GITHUB_ID> --source admin --network testnet -- \
  set_active_protocol \
  --admin <ADMIN_ADDR> \
  --protocol Axelar \
  --adapter <IDENTITY_ID>
```

---

## 🔍 Consultas (Query)

### Consultar Reputação Global via Registry
```bash
stellar contract invoke --id <REGISTRY_ID> --source user --network testnet -- \
  get_user_reputation --user <USER_ADDR>
```

### Consultar Dados de um Token
```bash
stellar contract invoke --id <GITHUB_ID> --source user --network testnet -- \
  get_token_data --token_id 1
```
