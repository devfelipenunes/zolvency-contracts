# Product Requirements Document (PRD): Zolvency Protocol v6.1
## The Universal Trust Hub: Transforming Cashflow into On-Chain Capital

**Versão:** 6.1 (Strategic Deep Baseline)  
**Status:** Approved for Implementation  
**Data:** 27 de Abril de 2026  
**Autor:** Gemini CLI (v4-Author-Auditor) & Felipe Nunes  

---

## 1. Visão Executiva e Tese de Mercado
O **Zolvency** resolve o "Problema do Colateral" na rede Stellar. Enquanto protocolos tradicionais de Lending (Aave, Compound) exigem sobrecolateralização (ex: deposite $150 para pegar $100), o Zolvency permite que o usuário utilize seu **fluxo de caixa futuro** e **reputação técnica** como garantia. 

Nossa tese é que a confiança verificada via **Zero-Knowledge Proofs (ZKP)** é o ativo mais líquido do século XXI. O Zolvency não é apenas um emissor de SBTs; é o oráculo de liquidação de risco para o ecossistema Stellar.

---

## 2. Especificações Técnicas das Verticais

### 2.1 Web2-to-Web3 Cashflow (Payroll & Invoicing)
Transformamos recebíveis (Notas Fiscais, Holerites) em tokens de crédito.
- **Mecânica de Verificação:** Integração com circuitos `zk-email` para validar o cabeçalho DKIM de e-mails corporativos/bancários sem expor dados sensíveis do usuário.
- **Protocolo de Oráculo:** O nó validador atesta a validade da prova ZK e o `ZolvencyRegistry` emite um score dinâmico.
- **KPI de Sucesso:** Volume de Originação de Crédito (VOV) > $500k no primeiro trimestre pós-lançamento.

### 2.2 RWA Provenance (Asset-Backed SBTs)
Identidade imutável para ativos físicos (Real World Assets).
- **Esquema de Dados:** Metadados padronizados via `get_metadata` v6.0, incluindo hash de documentos de propriedade, histórico de manutenção e última avaliação de mercado.
- **Lógica de Performance:** SBTs que degradam o score se o relatório de auditoria trimestral não for enviado, forçando transparência do emissor do RWA.

### 2.3 Institutional Gateway (Compliance & Gated-DeFi)
Camada de firewall para protocolos permissionados.
- **KYC-ZKP:** O usuário prova que possui KYC em uma exchange parceira (ex: Mercado Bitcoin, Coinbase) sem revelar sua identidade real ao dApp final.
- **Regras de Acesso:** White-listing automático em pools de liquidez baseado no `ZolvencyTier`.

---

## 3. Pilares de Engenharia de Risco (Armor Up)

### 3.1 Reputation Lock (Anti-Arbitragem)
Evita o "Ataque de Confiança Recursiva".
- **Teoria Pura:** Se um usuário tem 100 pontos de reputação e pega $50 em empréstimo no Protocolo A, sua reputação disponível deve cair para 50.
- **Implementação:** O contrato de Lending chama `lock_reputation(env, caller, user, unlock_timestamp)`. O Hub bloqueia novas consultas de score até que a dívida seja liquidada ou o timestamp expire.

### 3.2 Reputation Decay (Proof-of-Freshness)
A reputação é perecível.
- **Mecânica:** Decaimento linear de 1% ao dia após o `Business TTL` (90 dias).
- **Trade-off:** Aumenta a frequência de transações (e taxas de rede), mas garante que o colateral de reputação não seja baseado em dados obsoletos (ex: um dev que parou de programar há 2 anos).

### 3.3 Zolvency Credit Multiplier (Risk Aggregator)
O Hub evolui de uma listagem para um **Cérebro de Underwriting**.
- **Mecânica:** Combinação ponderada de múltiplos SBTs (ex: GitHub + Tax + Machine-Health) para gerar um multiplicador de LTV único e dinâmico.
- [[Trust-LTV-Multiplier]], [[Risk-Aggregation]]

### 3.4 Slashing Global (Blacklisting)
A "Pena de Morte" financeira no ecossistema.
- **Gatilho:** Liquidação forçada em um protocolo parceiro envia uma chamada de `apply_slashing` ao Registry.
- **Efeito:** O endereço do usuário é marcado em `DataKey::Blacklist`. Todas as consultas de reputação retornam `Map::new()`, invalidando instantaneamente qualquer outro benefício ou linha de crédito ativa.

---

## 4. Análise de Red Teaming (Vetores de Ataque)

1.  **Ataque de Sibil (Sybil Attack):**
    - *Vetor:* Um usuário cria 10 identidades GitHub falsas para farmar score.
    - *Mitigação:* O Hub mantém um mapeamento `ExternalID -> TokenID`. Uma vez que um ID externo é vinculado a uma carteira, qualquer tentativa de re-vincular invalida o token anterior (`Sybil Resistance` ativa).
2.  **Collusion (Conluio de Oráculos):**
    - *Vetor:* Um Spoke malicioso emite scores altos para endereços controlados por ele.
    - *Mitigação:* Implementação de Staking/Bonding obrigatório para novos Spokes. Se o Spoke for pego mentindo, seu stake é confiscado pelo Fundo de Adjudicação.

---

## 5. Roadmap v6.x (The Execution Path)

| Fase | Marco | Entregável Técnico |
| :--- | :--- | :--- |
| **Q2-2026** | **Hardening** | Auditoria Slither/Clippy 100% verde + Two-Step Admin. |
| **Q3-2026** | **Cashflow** | Circuito ZK-Email funcional no Spoke de Recebíveis. |
| **Q4-2026** | **RWA Hub** | Integração com oráculo de preço de ativos físicos (Chainlink/Stellar-Price). |
| **Q1-2027** | **DAO Transition** | Governança transferida para detentores de $ZOLV stake. |

---

## 6. Glossário de Novos Nós
- [[ZK-Email-DKIM]]: Verificação de origem de e-mail via criptografia de chave pública.
- [[Trust-LTV-Multiplier]]: Algoritmo de ajuste de margem de garantia baseado em score social.
- [[Reputation-Arbitrage]]: Ato de explorar dessincronizações de reputação entre protocolos.
- [[Zolvency-Adjudication-Fund]]: Reserva técnica para cobertura de fraudes em SBTs de recebíveis.
