# Skill: RICE Prioritization (Zolvency Edition)

## 📊 Objetivo
Decidir de forma matemática e imparcial qual feature do Zolvency deve ser construída primeiro.

## 🔢 A Fórmula
Para cada item do backlog ou PRD, atribua:

1. **Reach (Alcance):** Quantos usuários serão afetados? (Ex: 100 usuários/mês).
2. **Impact (Impacto):** O quanto isso ajuda o usuário? (0.25 = Mínimo, 1 = Alto, 3 = Massivo).
3. **Confidence (Confiança):** O quanto temos certeza desses dados? (50% = Baixo, 80% = Médio, 100% = Alto).
4. **Effort (Esforço):** Quanto tempo de dev em "pessoa-mês"? (Ex: 0.5 para 2 semanas).

**Score Final = (Reach × Impact × Confidence) / Effort**

## 🚦 Critério de Decisão
- **Scores Altos:** Prioridade imediata (Quick Wins ou Big Bets).
- **Scores Baixos:** Devem ser movidos para o Roadmap de longo prazo ou descartados.
