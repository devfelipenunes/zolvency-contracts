# Product Requirements Document (PRD): Zolvency Protocol v4.0

**Versão:** 4.0 (The Trust Settlement Layer)  
**Status:** Approved Baseline  
**Data:** 24 de Abril de 2026  
**Autor:** Gemini CLI & Felipe Nunes  

---

## 1. Visão do Produto
O **Zolvency** é o protocolo de infraestrutura de liquidação de confiança da rede Stellar. Ele transforma o comportamento humano e empresarial em **recurso econômico líquido**, permitindo que a reputação atue como colateral dinâmico, segurável e verificável cross-chain.

---

## 2. Pilares Estratégicos e Mecânicas de Jogo

### 2.1 Reputation Lock (Anti-Arbitrage)
- **Mecânica:** Travamento instantâneo do score no Hub Registry ao abrir uma posição de crédito. Bloqueia a "fuga de reputação" durante a latência de sincronização cross-chain.

### 2.2 Reputation Decay (Prova de Vida)
- **Mecânica:** SBTs possuem um fator de decaimento logarítmico. A ausência de novas provas (ex: 90 dias sem commit ou 30 dias sem renda) reduz o score automaticamente em 10% ao mês.

### 2.3 Staked Reputation (Skin in the Game)
- **Mecânica:** Usuários podem fazer stake de ativos (XLM, USDC) sobre seu SBT. 
- **Tier Ultra-Premium:** Apenas para usuários com Passkey + Staked Collateral. Este tier oferece os maiores multiplicadores de LTV do mercado.

### 2.4 Slashing Algorítmico e Adjudicação
- **Mecânica:** O "blacklisting" global é acionado por provas de liquidação on-chain. Contestações são resolvidas via Adjudicators autorizados (DAOs ou Oráculos de Justiça).

---

## 3. Requisitos Funcionais

### 3.1 Hub Registry (O Cérebro)
- **Lock & Decay Engine**: Gerencia o estado de travamento e o cálculo de expiração dinâmica.
- **Unified Query API**: Retorna o score líquido (Score Base - Decay + Stake Bonus).

### 3.2 Spoke Contracts (As Fontes)
- **GitHub SBT**: Validação técnica e autoridade de código.
- **Bank-SBT**: Focado em fluxo de caixa (PIX/Open Banking).
- **Merchant-SBT (PJ)**: Validação de volume de vendas para pequenas empresas através de provas de APIs de pagamento.

### 3.3 Interoperabilidade
- **Zolvency Verification API (ZVA)**: Protocolo de consulta para contratos EVM validarem locks ativos e tiers de confiança.

---

## 4. Modelos de Monetização (Revenue Streams)

1.  **VaaS (Verification-as-a-Service)**: Taxa por consulta de eligibilidade (paga pelo protocolo de Lending).
2.  **Staking Yield Spread**: Uma pequena fração do rendimento de ativos em stake sobre reputação retorna ao tesouro do protocolo.
3.  **Sync & Adjudication Fees**: Taxas por sincronização cross-chain e resolução de disputas de inadimplência.
4.  **Verification Badges**: Taxas de auditoria para novos Spokes entrarem no "Registry Curated List".

---

## 5. Roadmap Estratégico

### Q2 2026: Infraestrutura Core
- [ ] Implementação do **Reputation Lock** e **Decay Logic** no Registry.
- [ ] Lançamento do Dashboard de Metadados para Agentes de IA.

### Q3 2026: Piloto Financeiro (MEI/Devs)
- [ ] **Bank-SBT** (PIX Proofs).
- [ ] Parceria com protocolo de Lending para "Empréstimo por Reputação".

### Q4 2026: Social & Governance
- [ ] **Staking Module**: Lançamento da camada de seguro sobre reputação.
- [ ] Migração para governança descentralizada (Zolvency DAO).

---

## 6. Métricas de Sucesso
- **Capital Efficiency**: Redução média de 20% na exigência de colateral real para usuários Zolvency Premium.
- **Reputation Freshness**: Intervalo médio de atualização de SBTs inferior a 45 dias.
- **Corporate Adoption**: Primeiro Spoke de faturamento PJ (CNPJ) operacional.
