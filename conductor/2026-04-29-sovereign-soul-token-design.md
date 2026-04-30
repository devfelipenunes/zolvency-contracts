# Design Spec: Novo Soul Token Soberano (Passkey-First)

**Data:** 2026-04-29  
**Status:** Draft  
**Autor:** Gemini CLI  

## 1. Objetivo
Reestruturar o contrato `zolvency-soul` para ser um sistema de Identidade Soberana (Self-Sovereign Identity) focado em **Passkeys**, eliminando a dependência de Endereços Stellar (`Address`) como identificadores primários. A arquitetura incluirá um mecanismo de recuperação on-chain totalmente soberano.

## 2. Arquitetura Proposta

### 2.1. Identificação "Passkey-First"
O identificador primário do usuário não será mais uma carteira Stellar, mas sim um **`SoulID`** (um inteiro único incremental).
A chave pública da Passkey (`BytesN<65>`) será a "porta de entrada" mapeada para esse `SoulID`.

### 2.2. Segurança do "Código de Resgate" (Refinamento Crítico)
Enviar um código/senha em texto plano na transação on-chain para fazer a recuperação é perigoso (bots podem interceptar a transação e roubar a identidade). 
**Solução Criptográfica:** O "Código de Resgate" será, na verdade, uma **Chave Privada Secundária** gerada pelo SDK no momento do Mint (ex: uma chave secp256r1 ou ed25519). 
- O usuário guarda essa chave privada (pode ser representada como uma frase de recuperação ou código QR).
- O contrato salva apenas a **Chave Pública de Recuperação** (`recovery_pubkey`).
- Para recuperar, o usuário assina a nova Passkey com o Código de Resgate (Chave Privada). O contrato verifica a assinatura e efetua a troca. O segredo nunca é exposto!

## 3. Alterações no Contrato (`zolvency-soul`)

### 3.1. Tipos de Dados (`DataKey` e Structs)

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Relayer,
    TotalSouls,
    SoulById(u32),                 // Mapeia SoulID para SoulData
    SoulByPasskey(BytesN<65>),     // Mapeia Passkey PubKey para SoulID
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SoulData {
    pub id: u32,
    pub passkey: BytesN<65>,           // Chave pública da Passkey atual
    pub recovery_pubkey: BytesN<65>,   // Chave pública do Código de Resgate
    pub minted_at: u64,
}
```

### 3.2. Funções Principais

- **`mint(env, passkey: BytesN<65>, recovery_pubkey: BytesN<65>) -> u32`**
  - Autenticado pelo `Relayer` (para cobrir as taxas do usuário que está no SDK).
  - Cria um novo `SoulID`, salva `SoulData` e faz o mapeamento `SoulByPasskey`.

- **`get_soul_by_passkey(env, passkey: BytesN<65>) -> Option<SoulData>`**
  - Retorna os dados da alma com base na Passkey (ideal para o SDK identificar o usuário automaticamente).

- **`recover_soul(env, old_passkey: BytesN<65>, new_passkey: BytesN<65>, recovery_signature: BytesN<64>)`**
  - Autenticado pelo `Relayer`.
  - Busca o `SoulID` a partir da `old_passkey`.
  - O payload da assinatura será o hash da `new_passkey` + `old_passkey`.
  - O contrato verifica a `recovery_signature` usando a `recovery_pubkey` guardada no `SoulData`.
  - Se válido, atualiza o `SoulByPasskey` (deleta o antigo, insere o novo) e atualiza o `SoulData.passkey`.

## 4. Plano de Execução (Próximos Passos)
1. **Refatorar `lib.rs`**: Substituir a lógica baseada em `Address` pela nova lógica de `SoulID` e `Passkey`.
2. **Implementar Criptografia**: Adicionar a validação `secp256r1_verify` na função de recuperação.
3. **Atualizar `test.rs`**: Simular o fluxo completo: mint via Passkey, busca por Passkey e o cenário de recuperação utilizando um par de chaves de resgate mockado.