# Zolvency Protocol: Contracts

Repositório oficial dos **contratos core** do Zolvency Protocol na rede Stellar (Soroban): **soul**, **nexus** e **zpay**.

> 🔀 Contratos auxiliares, verifiers cross-chain (EVM/Cosmos/Solana) e adaptadores de interoperabilidade Axelar foram extraídos para o repositório [`devfelipenunes/zolvency-interop`](https://github.com/devfelipenunes/zolvency-interop).

## 📂 Estrutura

```
contracts/
├── soul       # Reputação soulbound (identidade raiz)
├── nexus      # Hub que agrega reputação dos spokes
└── zpay       # Pagamentos / gateway ZPay
```

## 📚 Documentação

### 🏗️ [Arquitetura](./docs/architecture/)

- **[Arquitetura Técnica](./docs/architecture/ARCHITECTURE.md)**: Visão geral do modelo Hub & Spoke.
- **[Padrão Técnico ZTS-01](./docs/architecture/ZTS_01_STANDARD.md)**: O padrão de interface para novos Spokes de reputação.
- **[Ciclo de Vida RWA](./docs/architecture/RWA_LIFECYCLE.md)**: Como gerenciar ativos físicos dinâmicos on-chain.
- **[Guia de Integração para Lending](./docs/specs/2026-04-27-lending-integration-spec.md)**: Como protocolos de crédito usam o Zolvency.

### 🚀 [Produto](./docs/product/)

- **[PRD - Product Requirements Document](./docs/product/PRD.md)**: Visão v6.1 do Trust Hub e integração RWA.
- **[Matriz de Riscos](./docs/product/RISK_MATRIX.md)**: Segurança e planos de mitigação.

### 📚 [Guias e Specs](./docs/)

- **[Comandos Úteis (Cheatsheet)](./docs/guides/CHEATSHEET.md)**: Guia rápido de Soroban CLI e deploy.
- **[Especificações de Design](./docs/specs/)**: Detalhes técnicos de cada funcionalidade implementada.

> 📄 Documentos de visão de produto (Manifesto, Economia, Personas, GTM, Horizontes), planos históricos de execução e o fluxo ZK-Email dos spokes Web2 foram movidos para [`devfelipenunes/zolvency-interop`](https://github.com/devfelipenunes/zolvency-interop).

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
- Spokes que emitem credenciais (ex: `github-identity`, `uber-income`, `income-bank`, `binance-kyc` — vivem em [`devfelipenunes/zolvency-interop`](https://github.com/devfelipenunes/zolvency-interop)) validam Soul no `mint` consultando o contrato Soul (ex: `balance(user) > 0`).
- O `nexus` agrega reputação consultando entrypoints padronizados dos spokes.

## 🧪 E2E (Soul-Centric Flow)

O script [scripts/e2e.sh](scripts/e2e.sh) executa um fluxo de ponta a ponta em testnet:

1. Mint da Soul
2. Criação de um Mandate no Nexus
3. Pagamento via ZPay
4. Escrow via ZPay
5. Revogação do mandate

Pré-requisitos:

- Um arquivo `.env` com `ADMIN_SECRET`
- IDs de contrato (`NEXUS_ID`, `SOUL_ID`, `ZPAY_ID`) configurados no próprio `.env`
- As identidades de teste (`user_e2e`, `agent_e2e`, `vendor_e2e`) são geradas automaticamente pelo script via `stellar keys generate`

Execução:

```bash
bash scripts/e2e.sh
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

## 🔀 Repositórios relacionados

- [`devfelipenunes/zolvency-interop`](https://github.com/devfelipenunes/zolvency-interop): adapters Axelar, verifiers cross-chain (EVM/Cosmos/Solana) e spokes auxiliares (flow, gig, github, proof-vault, direct-sovereign).

---

Para mais detalhes sobre a visão do produto, veja o [PRD](./docs/product/PRD.md).
