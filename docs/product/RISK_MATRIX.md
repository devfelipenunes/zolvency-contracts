# Zolvency Risk Matrix: Security and Mitigations

| Risco | Impacto | Probabilidade | Mitigação |
| :--- | :--- | :--- | :--- |
| **Oráculo Malicioso** | Massivo | Baixa | Curadoria rigorosa, auditoria automática (Slither/Cargo Audit) e depósito de caução (Bond) para novos Spokes. |
| **Exploit de Slashing** | Alto | Média | Auditoria automática (Slither/Cargo Audit) e período de carência (Timelock) antes da negativação global. |
| **Latência Cross-chain** | Média | Alta | **Reputation Lock** na rede de origem antes de autorizar crédito remoto. |
| **Centralização de Admin** | Alto | Média | **Two-Step Admin Transfer** implementado; migração para Multi-sig imediata. |
| **Vazamento de Dados** | Baixo | Baixa | Uso exclusivo de Zero-Knowledge Proofs; nenhum dado pessoal toca o ledger. |
| **Fraude Física (RWA)** | Massivo | Média | **Asset-Bonding Stake** (colateral do emissor) e auditoria de Multi-sig para ativos de alto valor. |
| **Disputa Judicial** | Média | Média | Integração com **Oráculos Judiciais** para marcar tokens como `:Lien` ou `:Seized` automaticamente. |
| **Quebra de Curva secp256r1** | Crítico | Quase Zero | Suporte para rotação de chaves e novos algoritmos de assinatura via upgrade do contrato. |
