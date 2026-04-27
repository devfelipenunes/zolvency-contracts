---
name: zolvency-internal-dev
description: Padrões arquiteturais, de segurança e de codificação para o protocolo Zolvency. Use ao desenvolver ou manter contratos Soroban e adaptadores de interoperabilidade.
---

# Zolvency Internal Developer Skill

Este guia define os padrões arquiteturais, de segurança e de codificação para o desenvolvimento do protocolo Zolvency. Agentes de IA e desenvolvedores devem seguir estas diretrizes rigorosamente para garantir a integridade e interoperabilidade do ecossistema.

## 1. Arquitetura Hub & Spoke

O protocolo Zolvency é organizado em uma topologia Hub & Spoke para garantir escalabilidade, segurança centralizada e uma interface unificada para protocolos DeFi.

- **Hub (Zolvency Registry):**
    - Atua como o "Registry" oficial de todos os SBTs (Soulbound Tokens).
    - Armazena a chave pública autorizada (`authorized_signer`) para validação global.
    - Fornece a API `get_user_reputation` que agrega dados de todos os Spokes.
- **Spoke (SBT Contracts):**
    - Implementam a `ZolvencyTokenTrait`.
    - Cada contrato representa uma fonte de reputação específica (GitHub, Bancos, Histórico On-chain).
    - Validam provas (ZK proofs ou assinaturas) antes de emitir tokens.

## 2. Padrão de Interoperabilidade Modular

A Zolvency utiliza o **Adapter Pattern** para exportar reputação para outras redes sem poluir o código principal dos contratos de identidade.

- **Desacoplamento:** O contrato `GithubIdentity` não conhece os detalhes do Axelar ou LayerZero. Ele apenas chama um `Adapter`.
- **Adaptadores:** Contratos independentes que implementam o envio de mensagens cross-chain.
- **Protocolos Suportados:**
    - **Axelar GMP:** Para automação "Push" completa via Gateways.
    - **Authority-Pull:** Para emissão de eventos e assinaturas off-chain (baixo custo no Stellar).
    - **LayerZero V2:** Para mensageria ultra-rápida entre OApps.

## 3. Requisitos de Segurança (Passkeys Opcionais)

O protocolo oferece suporte a segurança baseada em hardware via WebAuthn/Passkeys (secp256r1).

- **Opt-in Security:** O uso de Passkey é estritamente **opcional**.
- **Lógica de Validação:**
    - Se `passkey` E `passkey_signature` forem fornecidos (`Some`), a validação criptográfica é obrigatória.
    - Se ambos forem `None`, a validação é ignorada, facilitando o onboarding.
    - **Falha de Integridade:** Se apenas um dos campos for fornecido, a transação deve falhar com `InvalidSignature`.

## 4. Convenções de Código (Rust Workspace)

O repositório é gerenciado como um Rust Workspace para manter a modularidade.

- **Estrutura de Pastas:**
    - `packages/stellar/contracts/`: Contratos Soroban.
    - `packages/evm/`: Contratos Solidity (Foundry).
- **Tipagem:** Utilize `Option<T>` em vez de placeholders de bytes vazios para campos opcionais.
- **Build:** Utilize o `Makefile` na raiz para garantir compilações determinísticas: `make build`.

## 5. Guia de Metadados (TokenMetadata)

Todos os contratos Spoke devem expor metadados padronizados para facilitar a integração com indexadores e UIs.

### Estrutura
```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub version: String,
    pub data_source: String,
}
```

### Implementação Obrigatória
Cada contrato deve implementar a função `get_metadata(env: Env) -> TokenMetadata` na `ZolvencyTokenTrait`.

---

**Nota para Agentes:** Ao iniciar uma tarefa de codificação:
1. Verifique se o contrato segue a `ZolvencyTokenTrait` v6.0.
2. Os testes unitários devem cobrir o cenário de "Passkey opcional".
3. **MANDATO DE SINCRONIZAÇÃO E DENSIDADE:** É obrigatório atualizar os documentos correspondentes (`ARCHITECTURE.md`, `PRD.md`, `ECONOMY.md`) sempre que houver mudanças. A documentação DEVE seguir o **Protocolo de Hiper-Densidade**: incluir teoria pura, detalhes de implementação, crítica de engenharia e análise de red teaming. Nunca aceite documentação "em tópicos" rasa; busque profundidade técnica máxima.
