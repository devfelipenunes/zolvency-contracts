# Zolvency Protocol: Contracts

Repositório oficial dos contratos inteligentes do Zolvency Protocol na rede Stellar (Soroban) e adaptadores de interoperabilidade.

## 📂 Estrutura de Documentação

A documentação completa do projeto foi organizada para facilitar a manutenção e o onboarding:

### 🏗️ [Arquitetura](./docs/architecture/)
- **[Arquitetura Técnica](./docs/architecture/ARCHITECTURE.md)**: Visão geral do modelo Hub & Spoke.
- **[Interoperabilidade](./docs/architecture/INTEROP.md)**: Detalhes sobre o sistema de mensagens cross-chain.

### 🚀 [Produto](./docs/product/)
- **[PRD - Product Requirements Document](./docs/product/PRD.md)**: Visão v6.0 do Trust Hub e integração RWA.
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
