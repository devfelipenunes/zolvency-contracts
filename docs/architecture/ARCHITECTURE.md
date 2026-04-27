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

## 5. Glossário de Novos Nós
- [[Soroban-TTL-Strategy]]: Manual de gestão de custos de armazenamento no Soroban.
- [[Cross-Chain-Nonce-Sync]]: Mecanismo de prevenção de replay em mensageria modular.
- [[Hub-Spoke-Separation]]: Vantagens competitivas de arquiteturas modulares em DeFi.
- [[ZK-Verifier-Hook]]: Interface para delegação de computação pesada para off-chain provvers.
