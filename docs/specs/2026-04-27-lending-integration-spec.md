# Spec: Lending Protocol Integration Guide

**Versão:** 1.0  
**Status:** Draft  
**Data:** 27 de Abril de 2026  

## 1. Objetivo
Este documento serve como referência técnica para protocolos de Lending (Stellar ou EVM) que desejam utilizar o Zolvency como motor de underwriting para crédito sub-colateralizado ou juros diferenciados.

## 2. Fluxo de Integração (Underwriting)

### 2.1 Consulta de Score (Underwriting)
O protocolo deve consultar o `ZolvencyRegistry` para obter a visão agregada do usuário.

**Chamada Soroban:**
```rust
let reputation: Map<Symbol, u64> = registry_client.get_user_reputation(&user_address);
```

**Lógica Recomendada de LTV (Loan-to-Value):**
| Tier (SBT) | Multiplicador de LTV | Redução de Juros (Base) |
| :--- | :--- | :--- |
| **Novice** | 1.0x (Colateral Padrão) | 0% |
| **Pro** | 1.1x | -1% |
| **Architect** | 1.25x | -2.5% |
| **Legend** | 1.5x | -5% |
| **Singularity** | 2.0x (Máximo) | -10% |

---

## 3. Underwriting para RWA (Real World Assets)

Diferente de SBTs sociais, SBTs de ativos RWA carregam valor intrínseco. Protocolos de Lending podem utilizar o Zolvency para colateralização direta de ativos físicos.

### 3.1 Consulta de Saúde do Ativo (Asset Performance)
Ao avaliar um ativo RWA como garantia:
```rust
let asset_metadata = registry_client.get_token_metadata(&asset_contract_address);
// Verifique o performance_score dentro do current_state (bps)
if asset_metadata.current_state.performance_score < 8000 {
    panic!("Asset underperforming; higher collateral required");
}
```

### 3.2 Lógica de Liquidação Física
Se o empréstimo colateralizado por RWA entrar em default:
1. O protocolo de Lending executa a apreensão legal do ativo (off-chain).
2. O protocolo chama o Registry para registrar a liquidação física.
3. O SBT do ativo é marcado como `:Seized`, encerrando sua vida financeira on-chain.

---

## 4. Mecanismo de Proteção (Reputation Lock)

Ao abrir uma posição de dívida, o protocolo de Lending **deve** travar a reputação do usuário no Zolvency para evitar que ele use o mesmo score em múltiplos protocolos simultaneamente (Trust Arbitrage).

### 3.1 Executando o Lock
O protocolo chama o Registry informando a data estimada de finalização do empréstimo.
```rust
registry_client.lock_reputation(&lending_contract_address, &user_address, &unlock_timestamp);
```
*Nota: Enquanto estiver travada, a função `is_locked` retornará true, e outros protocolos devem negar novos créditos.*

## 4. Liquidação e Slashing (The "Stick")

Se o usuário entrar em default (inadimplência) e for liquidado, o protocolo de Lending tem o direito de "queimar" a confiança do usuário globalmente.

### 4.1 Aplicando o Slashing
O administrador do protocolo de Lending (ou o contrato de liquidação) envia a prova de inadimplência para o Registry.
```rust
registry_client.apply_slashing(&admin_address, &user_address);
```
*Resultado: O usuário entra na Blacklist Global. Todas as funções `get_user_reputation` retornarão zero para este endereço, e ele perderá acesso a todos os benefícios do ecossistema Zolvency.*

## 5. Integração Cross-chain (EVM)

Protocolos em redes EVM devem utilizar o `ZolvencyVerifier.sol` (alimentado via Axelar/LayerZero) para verificar os mesmos dados. O fluxo de Lock/Slashing em redes remotas será mediado pelos adaptadores de interoperabilidade.

---

## 6. Próximos Passos para Desenvolvedores
1. Implementar o `Zolvency-LTV-Hook` no seu contrato de Lending.
2. Testar o fluxo de Slashing em ambiente de testnet.
3. Consultar o [Cheatsheet](../guides/CHEATSHEET.md) para endereços de contrato.
