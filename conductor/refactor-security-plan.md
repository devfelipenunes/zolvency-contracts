# Plano de Refatoração: Segurança e Otimização de Contratos

Este plano descreve as alterações necessárias para corrigir as vulnerabilidades identificadas e otimizar o desempenho do ecossistema Zolvency.

## 1. Contratos EVM (Hardening)

### ZolvencyVerifierAuthority.sol
- **Problema:** Vulnerabilidade a Replay Attacks (Cross-chain, Cross-contract e Re-uso de assinatura).
- **Mudanças:**
    - Adicionar um mapeamento de `nonces` por usuário.
    - Incluir `block.chainid` e `address(this)` no hash assinado.
    - Atualizar a função `verifyAndSetReputation` para validar e incrementar o nonce.

### ZolvencyVerifierAxelar.sol
- **Problema:** Uso excessivo de `string` para endereços internos.
- **Mudanças:**
    - (Opcional/Menor prioridade) Manter compatibilidade com Axelar (que usa strings), mas garantir validações rigorosas.

## 2. Contratos Stellar (Otimização e Padronização)

### ZolvencyRegistry (Hub)
- **Problema:** Risco de DoS por limite de gás na varredura total de tokens.
- **Mudanças:**
    - Marcar `get_soul_reputation` (sem tokens) como depreciada ou limitar o `count`.
    - Garantir que a exportação cross-chain use tipos de dados consistentes.

### Spokes (Uber, GitHub, etc.)
- **Problema:** Falta de um nonce compartilhado para exportação cross-chain consistente.
- **Mudanças:**
    - Garantir que o `nonce` usado na exportação cross-chain seja consistente entre o Spoke e o Registry.

## 3. Verificação e Testes
- Atualizar scripts de teste em `packages/stellar/contracts/*/src/test.rs`.
- Criar novos testes de replay para o contrato EVM.

---
**Nota:** A funcionalidade de Relayer será mantida conforme solicitado.
