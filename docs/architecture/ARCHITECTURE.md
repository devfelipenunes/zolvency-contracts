# Technical Architecture: Zolvency Protocol v6.1
## The Hub & Spoke Infrastructure for Programmable Trust

**Versão:** 6.1 (Technical Deep Dive)  
**Status:** Baseline for Implementation  
**Data:** 27 de Abril de 2026  
**Autor:** Gemini CLI (v4-Author-Auditor) & Felipe Nunes  

---

## 1. Overview da Topologia Hub & Spoke

O Zolvency foi desenhado sob o princípio da **Separação de Preocupações (Separation of Concerns)**. Diferente de identidades monolíticas, nossa rede é composta por um centro de governança (Hub) e múltiplos emissores de prova (Spokes).

### 1.1 O Hub: Zolvency Registry
O `ZolvencyRegistry` é o cérebro do protocolo. Suas responsabilidades são:
- **Indexação Global:** Mapeia quais contratos são emissores legítimos (ex: `GithubIdentity`, `BankIdentity`).
- **Agregação de Score:** Fornece a função `get_user_reputation` que itera sobre todos os Spokes registrados para compor o perfil do usuário.
- **Gestão de Estado de Risco:** Mantém as listas de `Locks` e `Blacklist` (Slashing).

### 1.2 Os Spokes: Zolvency Tokens (SBTs)
Cada Spoke é um contrato Soroban especializado em uma fonte de dados Web2 ou Web3.
- **Isolamento de Risco:** Um bug no circuito ZK de um Bank-SBT não afeta o funcionamento do GitHub-SBT.
- **Padronização:** Todos expõem a `ZolvencyTokenTrait` para garantir intercompatibilidade.

### 1.3 A Raiz de Identidade: Zolvency Soul
Além do Hub & Spoke, o protocolo utiliza um contrato raiz de identidade (`zolvency-soul`). Ele representa o “login”/presença mínima do usuário no ecossistema.

**Invariante operacional:** *sem Soul, sem credencial*.

- O `mint` de credenciais em Spokes deve negar a emissão quando `balance(user) == 0` no contrato Soul.
- O endereço do contrato Soul é parte da configuração do Spoke (via `initialize` ou setter de admin, dependendo do contrato).
- A Soul é não-transferível e deve ser “permanente” do ponto de vista de UX: os contratos aplicam estratégia de TTL para evitar expiração por inatividade técnica.

---

## 2. Deep Dive: Mecanismos de Proteção (Armor Up)

### 2.1 O Ciclo de Vida de um Reputation Lock
O `Reputation Lock` é a nossa defesa contra a **Trust Arbitrage** (o ato de usar o mesmo score para pegar empréstimos em dois protocolos simultaneamente).

**Fluxo de Dados:**
1. **Solicitação:** Um dApp de Lending chama `lock_reputation` no Registry ao liberar um crédito.
2. **Validação de Auth:** O Registry verifica se o `caller` (o dApp) é um protocolo autorizado via `AccessControl`.
3. **Persistência:** O estado é salvo como `DataKey::Locks(Address)`. O valor armazenado é o `unlock_timestamp`.
4. **Bloqueio:** Qualquer tentativa subsequente de consulta de score (para underwriting) pelo `get_user_reputation` resultará em um erro ou score reduzido enquanto o tempo não expirar.

### 2.2 Governança Segura: Two-Step Admin Transfer
Para evitar a perda de controle do Hub por erro humano (ex: transferir para um endereço errado), implementamos o padrão de aceitação:
- `transfer_admin(new_admin)`: Marca o novo admin como `Pending`. O admin atual ainda mantém o poder.
- `accept_admin()`: O novo admin deve chamar esta função para confirmar que possui a chave privada e assumir o controle.

---

## 3. Estratégias de Armazenamento no Soroban

O gerenciamento de estado no Soroban é caro e requer manutenção de TTL (Time To Live). O Zolvency utiliza três níveis de persistência:

1.  **Persistent Storage:** Usado para o Registry (Admin, Signers, Blacklist). Requer renovação periódica de TTL.
2.  **Temporary Storage:** Usado para nonces e estados de transição rápida que não precisam durar anos.
3.  **Instance Storage:** Usado para configurações do contrato (Fees, Verifiers) que são lidas em quase todas as chamadas.

### Renovação de Estado (TTL Management)
Implementamos a função `renew_token_ttl(token_id)` que permite que qualquer usuário pague uma taxa pequena em XLM para estender a vida de seu SBT por mais 1 ano (ONE_YEAR constant), garantindo que a reputação não desapareça por inatividade técnica do ledger.

No caso da Soul, o contrato também aplica renovação de TTL no armazenamento `persistent` do usuário (quando presente) e estende TTL de `instance`/code para reduzir risco de expiração do próprio contrato.

---

## 4. Segurança de Interoperabilidade (The Adapter Pattern)

Nossa ponte cross-chain é agnóstica a protocolo. 

```rust
// A interface que todos os adaptadores devem seguir
pub trait MessengerTrait {
    fn estimate_fee(env: Env, destination_chain: String) -> i128;
    fn send(...) -> Result<(), Error>;
}
```

### Análise de Red Teaming: O Ataque do "Replay de Mensagem"
- **Risco:** Um hacker captura um pacote Axelar de uma atualização de reputação e tenta reenviá-lo para dobrar o score no EVM.
- **Mitigação:** Cada pacote contém um `nonce` único atrelado ao `token_id` na rede de origem. O contrato `ZolvencyVerifier.sol` (EVM) mantém um mapping `processed_nonces` e rejeita qualquer ID já utilizado.

---

## 6. The Interoperable Trust Layer

The Zolvency architecture is designed for **Cross-Chain Portability**. By utilizing a "Hub & Spoke" model on Stellar/Soroban, we create a centralized trust anchor that can export reputation attestations to any external network.

### 6.1 Dual-Token Identity Binding
- **Identity Hub (Soul ID):** The root of trust on Stellar.
- **ZK Reputation Spokes (SBTs):** Specialized contracts that verify external data (GitHub, Bank, etc.) via **zkTLS**.
- **The Binding:** Proofs are cryptographically tied to the Soul ID using ZK context, preventing identity theft across chains.

### 6.2 Cross-Chain Attestation Flow
1. **Verification:** A ZK proof is verified on-chain by a Soroban Spoke contract.
2. **Attestation:** The contract emits an event or updates a state that is picked up by a Cross-Chain Adapter (Axelar/LayerZero).
3. **Consumption:** A dApp on an external chain (e.g., Ethereum) receives the verified "Trust Score" and executes business logic (e.g., releasing a loan).

### 6.3 Agentic Trust Extensions
Zolvency supports **AI Agent Identities**. A human user can delegate a "Sub-Soul" to an agent, restricting its capabilities via ZK proofs of policy compliance. This allows agents to act on behalf of humans across multiple chains with cryptographic safety.
