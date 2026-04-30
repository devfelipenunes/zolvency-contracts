# Zolvency Protocol: Contracts

Repositório oficial dos contratos inteligentes do Zolvency Protocol na rede Stellar (Soroban) e adaptadores de interoperabilidade.

## 📂 Estrutura de Documentação

A documentação completa do projeto foi organizada para facilitar a manutenção e o onboarding:

### 🏗️ [Arquitetura](./docs/architecture/)
- **[Arquitetura Técnica](./docs/architecture/ARCHITECTURE.md)**: Visão geral do modelo Hub & Spoke.
- **[Padrão Técnico ZTS-01](./docs/architecture/ZTS_01_STANDARD.md)**: O padrão de interface para novos Spokes de reputação.
- **[Ciclo de Vida RWA](./docs/architecture/RWA_LIFECYCLE.md)**: Como gerenciar ativos físicos dinâmicos on-chain.
- **[Fluxo ZK-Email](./docs/architecture/ZK_EMAIL_FLOW.md)**: Detalhamento visual da verificação de fluxo de caixa Web2.
- **[Interoperabilidade](./docs/architecture/INTEROP.md)**: Detalhes sobre o sistema de mensagens cross-chain.
- **[Guia de Integração para Lending](./docs/specs/2026-04-27-lending-integration-spec.md)**: Como protocolos de crédito usam o Zolvency.

### 🚀 [Produto](./docs/product/)
- **[Manifesto Sovereign Trust](./docs/product/MANIFESTO.md)**: A visão estratégica de ir além da solvência e dominar o mercado RWA.
- **[PRD - Product Requirements Document](./docs/product/PRD.md)**: Visão v6.1 do Trust Hub e integração RWA.
- **[Horizontes Futuros](./docs/product/FUTURE_HORIZONS.md)**: 10 direções estratégicas de alta rentabilidade (Web2-to-Web3).
- **[Modelo Econômico](./docs/product/ECONOMY.md)**: Tokenomics de spread e taxas B2B.
- **[Personas e Jornadas](./docs/product/PERSONAS.md)**: Mapa de usuários, RWA Issuers e IAs.
- **[Matriz de Riscos](./docs/product/RISK_MATRIX.md)**: Segurança e planos de mitigação.
- **[Estratégia de Mercado (GTM)](./docs/product/GTM_STRATEGY.md)**: Roadmap de adoção e crescimento.

### 📚 [Guias e Specs](./docs/)
- **[Comandos Úteis (Cheatsheet)](./docs/guides/CHEATSHEET.md)**: Guia rápido de Soroban CLI e deploy.
- **[Guia de Interoperabilidade Axelar](./docs/guides/AXELAR_INTEROP_GUIDE.md)**: Passo a passo para integração com EVM.
- **[Especificações de Design](./docs/specs/)**: Detalhes técnicos de cada funcionalidade implementada.
- **[Planos de Implementação](./docs/plans/)**: Histórico de execução das tasks.

## 🤖 Para Agentes de IA
Este repositório inclui uma [Skill de Desenvolvimento Interna](./docs/internal/SKILL.md). Ao trabalhar neste projeto, carregue esta skill para garantir adesão aos padrões arquiteturais e de segurança do Zolvency.

## ⚡ Quick Start

Se você acabou de clonar o repositório, utilize o `Makefile` para preparar o ambiente:

```bash
# Instalar dependências e compilar contratos
make build

# Executar a suíte completa de testes
make test

# Verificar lint e formatação
make lint && make fmt
```

## 🔐 Invariante: “Sem Soul, sem credencial”

O protocolo assume uma identidade raiz (`zolvency-soul`) como pré-requisito para emissão de credenciais (Spokes). Na prática:

- O usuário primeiro “loga” e recebe uma Soul (mint via `relayer` autorizado).
- Spokes que emitem credenciais (ex: `github-identity`, `uber-income`, `income-bank`, `binance-kyc`) validam Soul no `mint` consultando o contrato Soul (ex: `balance(user) > 0`).
- O `zolvency-registry` agrega reputação consultando entrypoints padronizados dos spokes.

## 🧪 E2E (Soul-Centric Flow)

O script [scripts/test_e2e_flow.sh](scripts/test_e2e_flow.sh) executa um fluxo de ponta a ponta em testnet:

1) Mint da Soul
2) Checagem de `balance`
3) Mint de um spoke (GitHub)
4) Mint de um spoke (Uber Income)

Pré-requisitos:
- Um arquivo `.env` com `DEPLOYER_SECRET` e `ADMIN_PUBLIC`
- IDs de contrato (`SOUL_ID`, `GITHUB_ID`, `UBER_ID`) configurados no próprio script

Execução:

```bash
bash scripts/test_e2e_flow.sh
```

## 🛠️ Como começar

### Pré-requisitos
- Rust & Cargo
- Soroban CLI
- Node.js (para scripts de validação)

### Build e Testes
```bash
# Build de todos os contratos
cargo build --target wasm32-unknown-unknown --release

# Executar testes unitários
cargo test
```

---
Para mais detalhes sobre a visão do produto, veja o [PRD](./docs/product/PRD.md).
