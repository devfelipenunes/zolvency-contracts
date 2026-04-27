# Product Requirements Document (PRD): Zolvency Protocol v6.0

**Versão:** 6.0 (The Universal Trust Hub)  
**Status:** Strategic Baseline  
**Data:** 24 de Abril de 2026  
**Autor:** Gemini CLI & Felipe Nunes  

---

## 1. Visão do Produto
O **Zolvency** é o protocolo de liquidação de confiança e integração RWA da rede Stellar. Ele atua como a ponte definitiva entre o fluxo de caixa Web2 e a liquidez Web3, transformando reputação, renda e ativos físicos em colaterais financeiros programáveis.

---

## 2. Verticais de Utilidade e Receita

### 2.1 Web2-to-Web3 Cashflow (Payroll & Invoicing)
- **Mecânica:** Transformar recebíveis do mundo real (Notas Fiscais, Holerites, PIX) em SBTs de Crédito Antecipável. O Zolvency valida a veracidade do documento via ZK-Email e libera o score para antecipação de liquidez em USDC.

### 2.2 RWA Provenance (Asset SBTs)
- **Mecânica:** Identidade digital para ativos físicos (Imóveis, Máquinas, Créditos de Carbono). O SBT rastreia o ciclo de vida e a performance do ativo, permitindo que investidores comprem "shares" de RWA com confiança auditada on-chain.

### 2.3 Institutional Gateway (Compliance Pass)
- **Mecânica:** Camada de conformidade para instituições. SBTs que atestam status de "Investidor Qualificado" e "KYC Bancário", permitindo a criação de Pools de DeFi Permissionadas (Gated-DeFi).

---

## 3. Pilares de Engenharia Econômica

### 3.1 Reputation & Asset Locks
- Travamento instantâneo de reputação ou direitos sobre ativos ao abrir linhas de crédito, eliminando o risco de gasto duplo (double-spending) de confiança entre chains.

### 3.2 Dynamic Underwriting (The Scoring Engine)
- Algoritmo que combina:
    - **Score de Atividade (GitHub/Social)**
    - **Score de Fluxo (Renda/Vendas Web2)**
    - **Score de Lastro (Stake/RWA)**

---

## 4. Modelos de Monetização de Alta Performance

1.  **Origination Spread:** Taxa sobre o volume de crédito antecipado via SBTs de recebíveis.
2.  **Asset Audit Fee:** Cobrança por atualização de performance de ativos RWA on-chain.
3.  **Institutional Access:** Assinatura B2B para protocolos que operam apenas com usuários "Zolvency Verified".
4.  **Verification-as-a-Service (VaaS):** Taxas por consulta de eligibilidade cross-chain.

---

## 5. Roadmap Estratégico

### Q2 2026: Infraestrutura Core
- Finalização do Registry com suporte a **Multi-Asset Metadata** e **Reputation Locks**.

### Q3 2026: O Salto Web2 (Recebíveis)
- Lançamento do **Bank-SBT** (Renda PIX) e **Invoice-SBT** (Notas Fiscais via ZK).
- Primeiro piloto de antecipação de recebíveis para Freelancers.

### Q4 2026: RWA & Institucional
- Lançamento do **Asset-SBT** para ativos físicos.
- Integração com o primeiro grande Player de RWA da Stellar.
