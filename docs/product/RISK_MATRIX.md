# Zolvency Risk Matrix: Security and Mitigations

| Risco | Impacto | Probabilidade | Mitigação |
| :--- | :--- | :--- | :--- |
| **Oráculo Malicioso** | Massivo | Baixa | Curadoria rigorosa e depósito de caução (Bond) para novos Spokes. |
| **Exploit de Slashing** | Alto | Média | Implementação de período de carência (Timelock) antes da negativação global. |
| **Latência Cross-chain** | Média | Alta | **Reputation Lock** na rede de origem antes de autorizar crédito remoto. |
| **Centralização de Admin** | Alto | Média | **Two-Step Admin Transfer** implementado; migração para Multi-sig imediata. |
| **Vazamento de Dados** | Baixo | Baixa | Uso exclusivo de Zero-Knowledge Proofs; nenhum dado pessoal toca o ledger. |
| **Quebra de Curva secp256r1** | Crítico | Quase Zero | Suporte para rotação de chaves e novos algoritmos de assinatura via upgrade do contrato. |
