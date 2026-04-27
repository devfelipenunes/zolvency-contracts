# Zolvency Economy: Trust Hub Tokenomics

## 1. Fluxo de Valor e Receita

O Zolvency transiciona de um modelo de "taxa de consulta" para um modelo de "spread de volume".

### 1.1 Spread de Originação (Web2 -> Web3)
Ao emitir um SBT de Recebível (Nota Fiscal/Holerite) para antecipação de crédito:
- **Taxa de Originação:** 0.5% a 1.5% do valor total da nota fiscal.
- **Divisão:**
    - 60% para o Protocolo Zolvency.
    - 30% para o Nó Validador (Spoke) que processou o ZK-Proof.
    - 10% para o Fundo de Adjudicação (Seguro contra fraude).

### 1.2 Asset Performance Fees (RWA)
Para ativos RWA (Imóveis, Máquinas):
- **Audit Fee:** Taxa fixa de 10 USDC cobrada a cada atualização trimestral de metadados do ativo (performance/manutenção).
- **Secondary Market Royalty:** 0.1% de cada troca de titularidade do ativo, cobrada para atualizar o histórico no SBT do ativo.

## 2. Dynamic Staking (Trust Tiers)
O sistema de stake agora escala com o valor do ativo identificado.
- **Micro-Stake:** Para SBTs de reputação social.
- **Asset-Bonding Stake:** Investidores de RWA precisam depositar USDC para garantir a veracidade dos dados do ativo que estão listando via Zolvency.

## 3. Modelo B2B: Compliance API
Instituições pagam por chamadas de alto volume via API:
- **Tier Free:** Até 100 consultas/mês.
- **Tier Pro:** $500/mês para até 10.000 consultas + suporte a Custom Spokes.
- **Tier Enterprise:** Sob consulta (para Bancos e Real Estate Issuers).
