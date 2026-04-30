# Relatório de Revisão Técnica: Zolvency Protocol (v6.1-Security-Focus)

**Data:** 30 de Abril de 2026  
**Status:** CRITICAL VULNERABILITIES IDENTIFIED  
**Autor:** Gemini CLI (v3-Tech-Analyst)

---

## 1. Core Mechanism: Hub & Spoke Sovereign Identity
O sistema transicionou para uma arquitetura "Passkey-First", utilizando o `SoulID` como âncora de identidade on-chain no Stellar, com verificação de provas cross-chain para o EVM via Axelar e Authority Signatures.

## 2. Security & Failure Analysis (CRITICAL)

### 2.1 EVM Verifiers: Replay Attack (Severity: CRITICAL)
Ambos os contratos de verificação no EVM (`ZolvencyVerifierAuthority.sol` e `ZolvencyVerifierAxelar.sol`) não possuem proteção contra replay.
- **Vulnerabilidade:** Um atacante pode capturar uma prova válida (assinatura ou payload Axelar) e reenviá-la múltiplas vezes para o mesmo endereço ou para endereços diferentes (se o payload não estiver estritamente vinculado ao endereço do usuário de forma verificável).
- **Impacto:** Inflação artificial de reputação e Trust Arbitrage.
- **Recomendação:** Implementar um `mapping(bytes32 => bool) public processedHashes` e incluir um `nonce` único (proveniente do Stellar) ou o hash da mensagem original para garantir que cada prova seja usada apenas uma vez.

### 2.2 Stellar: Soul Recovery Collision (Severity: MEDIUM)
No contrato `zolvency-soul`, a função `recover_soul` remove o mapeamento da `old_passkey` ANTES de inserir a `new_passkey`.
- **Vulnerabilidade:** Se `old_passkey == new_passkey`, o mapeamento é removido e nunca re-adicionado (pois a inserção posterior sobrescreve ou falha na lógica de existência). Além disso, não há verificação se a `new_passkey` já pertence a outro `SoulID`.
- **Recomendação:** Adicionar `require(old_passkey != new_passkey)` e verificar se a nova passkey já está em uso.

### 2.3 Stellar: Registry Consistency (Severity: MEDIUM)
O `ZolvencyRegistry` ainda utiliza o termo `Address` em alguns contextos de travamento, embora tenha sido atualizado para suportar `SoulID`.
- **Vulnerabilidade:** Inconsistência de dados entre contratos que usam `Address` vs `SoulID` pode levar a falhas na agregação de score (`get_soul_reputation`).
- **Recomendação:** Padronizar todos os mapeamentos de reputação para `SoulID` e garantir que o `is_soul_locked` seja consultado por todos os Spokes antes da emissão.

### 2.4 Signature Malleability & Expiry (Severity: LOW)
O `ZolvencyVerifierAuthority.sol` não verifica a validade temporal da assinatura.
- **Recomendação:** Incluir um `deadline` (timestamp) no payload assinado para evitar o uso de provas obsoletas.

---

## 3. Trade-offs (Pros/Cons)

### Pros:
- **Sovereignty:** O uso de Passkeys e `SoulID` elimina a dependência de chaves privadas Stellar tradicionais para identidade, melhorando significativamente a UX.
- **Modularity:** A separação em Spokes permite a evolução independente de cada fonte de dados.

### Cons:
- **Relayer Dependency:** O sistema de mint e recovery depende de um `Relayer` para pagar as taxas. Se o relayer for censurado ou ficar offline, o usuário fica bloqueado (embora possa agir diretamente se possuir XLM, a interface de Passkey é abstraída).
- **Storage Costs:** O uso extensivo de `Persistent Storage` no Soroban exige uma estratégia de renovação de TTL ativa para evitar perda de dados.

---

## 4. Próximos Passos Recomendados
1. **Fix Replay Attack (EVM):** Urgente. Implementar sistema de nonces/hashes processados.
2. **Hardening `recover_soul` (Stellar):** Adicionar checks de segurança para evitar colisões de passkey.
3. **Registry Audit:** Revisar todos os spokes para garantir que utilizam `SoulID` corretamente na função `has_identity`.
