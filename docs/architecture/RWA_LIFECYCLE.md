# RWA Lifecycle Infrastructure: Technical Specification
## Managing the Performance and Provenance of Physical Assets on Soroban

**Versão:** 1.0 (Technical Deep Dive)  
**Status:** Baseline for Implementation  
**Data:** 27 de Abril de 2026  
**Autor:** Zolvency Core & Gemini CLI (v4-Author-Auditor)

---

## 1. O Esquema de Dados RWA (v7.0 Standard)

Para suportar ativos físicos que "respiram", o Zolvency expande o `TokenMetadata` para incluir um campo de **Estado Dinâmico**.

```rust
#[contracttype]
pub struct RWAState {
    pub last_valuation: i128,      // Valor em USDC
    pub performance_score: u32,    // 0-10000 (bps) baseado em yield/manutenção
    pub maintenance_hash: BytesN<32>, // Hash do log de manutenção off-chain
    pub legal_status: Symbol,      // :Clear, :Lien, :Dispute
}

#[contracttype]
pub struct RWAMetadata {
    pub asset_class: Symbol,       // :RealEstate, :Machinery, :Invoice
    pub physical_id: String,       // Chassis, Registro de Imóvel, etc.
    pub current_state: RWAState,
    pub verifier_address: Address, // O Spoke/Oráculo responsável
}
```

---

## 2. Orquestração de Oráculos de Performance

Diferente de preços de tokens (que vêm da Chainlink), a performance RWA exige oráculos de **Veracidade Física**.

### 2.1 Fluxo de Atualização (The Performance Heartbeat)
1. **Coleta de Dados:** Sensores de IoT ou Auditores Credenciados geram um relatório de performance (ex: "A usina solar gerou 500MWh este mês").
2. **Attestation:** O Auditor assina o payload off-chain usando uma chave autorizada no Registry.
3. **Commit:** O proprietário do ativo ou um bot automático chama o método `update_asset_performance` no Spoke correspondente.
4. **Validation:** O Spoke verifica a assinatura e recalcula o **Asset-Score** do SBT.
5. **Ripple Effect:** O novo score é propagado para o Registry, que pode ajustar automaticamente o LTV (Loan-to-Value) em protocolos de Lending parceiros.

---

## 3. Mecânica de Slashing e Liquidação Física

Quando um ativo RWA entra em inadimplência no mundo físico, o Zolvency atua como a **Camada de Enforcement Digital**.

### 3.1 Prova de Liquidação (PoL)
Se um imóvel é retomado por falta de pagamento:
- O protocolo de Lending envia uma **Prova de Liquidação** (assinatura legal ou decisão de oráculo judicial) para o `ZolvencyRegistry`.
- O Hub marca o SBT do ativo como `:Seized`.
- **Efeito:** O ativo perde toda a sua capacidade de gerar novos créditos e sua reputação cai para zero, impedindo o proprietário original de reutilizá-lo como colateral em qualquer lugar do ecossistema Stellar.

---

## 4. Análise de Red Teaming: O Ataque do "Oráculo Auditor"

- **Ameaça:** Um auditor recebe suborno para reportar que uma máquina quebrada está funcionando perfeitamente, mantendo o score alto.
- **Mitigação 1 (Skin in the Game):** Auditores devem manter um `Asset-Bonding Stake` no Hub.
- **Mitigação 2 (Multi-Sig Attestation):** Ativos de alto valor (ex: > $1M) exigem assinaturas de dois auditores independentes para atualizar o estado de performance.

---

## 5. Glossário de Novos Nós
- [[Dynamic-Asset-State]]: Capacidade de metadados evoluírem sem mudar o ID do token.
- [[Physical-ID-Hashing]]: Técnica para vincular objetos físicos únicos a endereços on-chain.
- [[Performance-Heartbeat]]: Transação periódica que atesta a saúde de um ativo RWA.
- [[Oráculo-Judicial]]: Agente que valida eventos de liquidação legal on-chain.
