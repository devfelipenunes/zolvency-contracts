# Guia de Interoperabilidade Zolvency: Stellar <-> EVM via Axelar

Este documento detalha a implementação técnica para a transferência de reputação (SBTs) entre a rede Stellar (Soroban) e redes compatíveis com EVM (como Ethereum Sepolia) utilizando a infraestrutura da Axelar.

## 1. Visão Geral da Arquitetura

O sistema utiliza o modelo **Push Automático**, onde uma ação no Stellar dispara uma mensagem cross-chain.

- **Source (Stellar):** `GithubIdentityContract` - Responsável por emitir a reputação e iniciar a ponte.
- **Transporte (Axelar):** Axelar GMP (General Message Passing) - Valida e entrega a mensagem entre as redes.
- **Destination (EVM):** `ZolvencyVerifier.sol` - Recebe, decodifica e armazena a reputação no destino.

## 2. Implementação Técnica

### 2.1 Codificação ABI no Rust (Stellar)
Como o Soroban e o EVM possuem formatos de dados diferentes, a lógica de codificação manual é encapsulada dentro de cada **Contrato Adaptador**. Isso mantém o contrato de Identidade focado apenas na lógica de reputação.

**Formato do Payload (EVM-Compatible):**
1. `externalId` (bytes32): Hash do identificador do GitHub (Keccak256).
2. `tier` (uint32): Nível de reputação (1-5).
3. `user` (address): Endereço da carteira do usuário no EVM (20 bytes).

```rust
// packages/stellar/contracts/adapters/axelar/src/lib.rs
fn encode_evm_payload(env: &Env, external_id: &String, tier: u8, user: &Bytes) -> Bytes {
    let mut payload = Bytes::new(env);
    // 1. externalId (32 bytes)
    let external_id_hash = env.crypto().keccak256(&external_id.clone().to_xdr(env));
    payload.append(&external_id_hash.into());
    
    // 2. tier (uint32 padded to 32 bytes)
    let mut tier_bytes = [0u8; 32];
    tier_bytes[31] = tier;
    payload.append(&Bytes::from_array(env, &tier_bytes));
    
    // 3. user address (20 bytes padded to 32 bytes)
    let mut user_bytes = [0u8; 32];
    user.copy_into_slice(&mut user_bytes[12..32]);
    payload.append(&Bytes::from_array(env, &user_bytes));
    
    payload
}
```

### 2.2 Uso do MessengerClient (Stellar Side)
O contrato de identidade utiliza uma interface tipada (`MessengerClient`) para interagir com os adaptadores, garantindo que o despacho cross-chain seja modular e expansível.

```rust
// github-identity/src/lib.rs
let messenger = MessengerClient::new(&env, &interop_config.adapter_address);
messenger.send(
    &caller,
    &cc.destination_chain,
    &cc.destination_address,
    &params.external_id,
    &(tier.to_number() as u32),
    &cc.user_destination_address,
);
```

### 2.3 Contrato Verificador (Solidity)
O contrato receptor verifica a origem para evitar ataques de falsificação de reputação.

```solidity
function _execute(
    bytes32 commandId,
    string calldata sourceChain,
    string calldata sourceAddress,
    bytes calldata payload
) internal override {
    require(keccak256(bytes(sourceChain)) == keccak256(bytes("stellar")), "INVALID_CHAIN");
    require(keccak256(bytes(sourceAddress)) == keccak256(bytes(sourceStellarAddress)), "INVALID_SOURCE");

    (bytes32 externalId, uint8 tier, address user) = abi.decode(payload, (bytes32, uint8, address));
    reputations[user] = Reputation({externalId: externalId, tier: tier});
}
```

## 3. Guia de Deploy e Teste

### Endereços Ativos (Testnet 2026)
- **Stellar Gateway:** `CB2JYOOZPHO43R57TC5PXV22QICKIDC5NKRF62BZG2J6JYFUIQPIAYY3`
- **Stellar Gas Service:** `CCLZOCGHHC6F6JCZHEUP53LDQHRBPPCNRYXOVFZFS3O63OGRC47CKCGV`
- **EVM Gateway:** `0xe432150cce91c13a887f7D836923d5597adD8E31`

### Comandos de Teste
Para testar a interoperabilidade, execute um `mint` no Stellar fornecendo os parâmetros `cross_chain`:

```bash
stellar contract invoke --id [STELAR_CONTRACT] --source deployer --network testnet -- \
  mint \
  --caller deployer \
  --signature ... \
  --params '{"username": "user", ...}' \
  --cross_chain '{"destination_chain": "ethereum-sepolia", "destination_address": "[EVM_VERIFIER]", "user_destination_address": "[USER_EVM_ADDR]"}'
```

## 4. Próximos Passos
1. **Sync Bidirecional:** Implementar a volta (EVM -> Stellar) caso o usuário queira atualizar sua reputação a partir de ações em protocolos DeFi no Ethereum.
2. **Multi-Chain Hub:** Registrar o `ZolvencyVerifier` em outras redes (Polygon, Base, Avalanche) para tornar a reputação universal.
