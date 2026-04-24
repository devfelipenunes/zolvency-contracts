# Zolvency Protocol: Contracts

Repositório oficial dos contratos inteligentes do Zolvency Protocol na rede Stellar (Soroban) e adaptadores de interoperabilidade.

## 📂 Estrutura de Documentação

A documentação completa do projeto foi organizada para facilitar a manutenção e o onboarding:

### 🏗️ [Arquitetura](./docs/architecture/)
- **[Arquitetura Técnica](./docs/architecture/ARCHITECTURE.md)**: Visão geral do modelo Hub & Spoke.
- **[Interoperabilidade](./docs/architecture/INTEROP.md)**: Detalhes sobre o sistema de mensagens cross-chain.

### 🚀 [Produto](./docs/product/)
- **[PRD - Product Requirements Document](./docs/product/PRD.md)**: Visão, objetivos e roadmap do protocolo.

### 📚 [Guias e Specs](./docs/)
- **[Guia de Interoperabilidade Axelar](./docs/guides/AXELAR_INTEROP_GUIDE.md)**: Passo a passo para integração com EVM.
- **[Especificações de Design](./docs/specs/)**: Detalhes técnicos de cada funcionalidade implementada.
- **[Planos de Implementação](./docs/plans/)**: Histórico de execução das tasks.

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
