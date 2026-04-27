# Zolvency Economy: Trust Hub Tokenomics & Financial Simulations

## 1. O Motor de Receita: Origination-as-a-Service (OaaS)

O Zolvency transiciona de um modelo de "taxa de consulta" para um modelo de "spread de volume". Nossa receita é diretamente proporcional ao crédito facilitado via SBTs.

### 1.1 Spread de Originação (Web2 Receivables)
Ao emitir um SBT de Recebível (ex: Nota Fiscal de $10.000) para antecipação na Stellar:
- **Taxa de Originação:** 1.0% ($100).
- **Alocação de Fluxo:**
    - **60% ($60) Tesouraria Zolvency:** Reinvestimento em P&D e queima de $ZOLV.
    - **30% ($30) Nó Validador (Spoke):** Recompensa pela computação da prova ZK e custódia de dados.
    - **10% ($10) Fundo de Adjudicação:** Reserva para cobrir inadimplência catastrófica (Seguro Interno).

### 1.2 Simulação de Fluxo de Caixa (Cenário Q3-2026)
| Métrica | Valor |
| :--- | :--- |
| Volume Mensal de Originação (GTV) | $1.000.000 |
| Receita Bruta do Protocolo (1.0%) | $10.000 |
| Yield para Stakers de $ZOLV (Buyback) | $3.000 |
| Crescimento do Fundo de Adjudicação | $1.000 |

---

## 2. Dynamic Staking & Trust Tiers (O "Skin in the Game")

Diferente de protocolos que aceitam qualquer validador, o Zolvency exige colateral para emissão de confiança.

### 2.1 Asset-Bonding Stake (Para RWAs)
Emissores de Ativos Reais (ex: Real Estate tokenizado) devem depositar 0.5% do valor do ativo em USDC no cofre de segurança do Zolvency.
- **Função:** Se o emissor falsificar os dados do ativo (provado via auditoria externa), o stake é utilizado para reembolsar os investidores do mercado secundário.
- **Incentivo:** Enquanto o stake está travado, o emissor recebe 50% das **Audit Fees** geradas pelo seu ativo.

### 2.2 Reputation Tiers (Impacto no LTV)
O score social impacta diretamente o custo de capital:
| Tier | Score Min. | Multiplicador LTV | Desconto na Taxa (APR) |
| :--- | :--- | :--- | :--- |
| **Novice** | 0 | 1.0x | 0% |
| **Pro** | 500 | 1.1x | -0.5% |
| **Architect** | 1500 | 1.2x | -1.5% |
| **Legend** | 3000 | 1.4x | -3.0% |
| **Singularity** | 5000+ | 1.6x | -5.0% |

---

## 3. Estratégia de Deflação: O Token $ZOLV

O token $ZOLV (futuro) atua como a unidade de governança e captura de valor.
- **Fee Burn:** Uma porcentagem de todas as `VaaS Fees` (Verification-as-a-Service) é usada para comprar e queimar $ZOLV do mercado secundário.
- **Governance Power:** O peso do voto não é apenas por tokens, mas multiplicado pelo `ZolvencyTier` do votante. (O voto de um "Legend" vale mais que o de um "Novice").

---

## 4. Glossário de Novos Nós
- [[Asset-Bonding-Stake]]: Colateral exigido de emissores RWA para garantir veracidade.
- [[Audit-Fee]]: Taxa recorrente para manutenção e validação de metadados on-chain.
- [[VaaS-Verification-as-a-Service]]: Modelo de negócio de API para dApps externos.
- [[Zolv-Burn-Mechanism]]: Algoritmo de buyback and burn baseado em volume de crédito.
