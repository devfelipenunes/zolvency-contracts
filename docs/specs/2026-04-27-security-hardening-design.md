# Design Spec: Armor Up - Automated Security Hardening

**Data:** 2026-04-27  
**Status:** Approved  
**Autor:** Gemini CLI  

## 1. Objetivo
Implementar uma camada de auditoria automática e análise estática de segurança para o protocolo Zolvency. O objetivo é garantir que vulnerabilidades conhecidas (EVM e Soroban) sejam detectadas instantaneamente durante o desenvolvimento e bloqueadas no CI (Continuous Integration).

## 2. Motivação
Com o avanço para a v6.0 (Trust Hub) e a manipulação de reputação que reflete valores financeiros reais, o custo de um exploit aumentou significativamente. Ferramentas de análise estática como Slither (Solidity) e Cargo Audit (Rust) são essenciais para manter a integridade do protocolo "Armor Up".

## 3. Arquitetura do Sistema de Auditoria

### 3.1 Camada de CI (GitHub Actions)
Expandir o `.github/workflows/ci.yml` para incluir os seguintes jobs:
- **EVM Security (Slither):** Analisa contratos Solidity em `verifiers/evm/src`. Bloqueia o build em caso de vulnerabilidades de impacto "High".
- **Rust Security (Cargo Audit):** Verifica o `Cargo.lock` contra a base de dados de vulnerabilidades da RustSec.
- **Static Analysis (Clippy):** Roda o linter avançado do Rust com flags de segurança para detectar padrões perigosos no Soroban.

### 3.2 Camada Local (Makefile)
Adicionar comandos para facilitar a auditoria manual:
- `make audit`: Executa Slither e Cargo Audit.
- `make lint`: Executa Clippy e fmt.

## 4. Ferramentas e Configurações

### 4.1 Solidity (Slither)
- **Configuração:** `slither.config.json` na raiz.
- **Filtros:** Ignorar alertas de "Naming Conventions" em contratos de bibliotecas externas para reduzir ruído.

### 4.2 Rust (Cargo Audit & Clippy)
- **Clippy:** Utilizar `#[deny(clippy::all)]` e especificamente checar por possíveis panic cases e integer overflows não tratados em lógica financeira.

## 5. Plano de Verificação
1. Validar que o `make audit` falha se uma vulnerabilidade conhecida (ex: reentrância simples) for introduzida propositalmente em um contrato de teste.
2. Confirmar que o pipeline do GitHub Actions falha corretamente em Pull Requests com problemas de segurança.

## 6. Sincronização de Documentação
- Atualizar a `docs/product/RISK_MATRIX.md` para refletir a implementação da auditoria automática como mitigação de riscos técnicos.
