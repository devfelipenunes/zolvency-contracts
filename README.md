# Zolvency Protocol: Smart Contracts

Repositório oficial dos contratos inteligentes do Zolvency Protocol na rede Stellar (Soroban) e adaptadores de interoperabilidade EVM (Solidity).

## 📂 Estrutura de Engenharia

A documentação técnica foca na arquitetura, padrões de interface e fluxos de segurança dos contratos:

### 🏗️ Arquitetura e Padrões
- **[Arquitetura Técnica](./docs/architecture/ARCHITECTURE.md)**: Visão detalhada do modelo Hub & Spoke e gestão de estado no Soroban.
- **[Padrão Técnico ZTS-01](./docs/architecture/ZTS_01_STANDARD.md)**: Especificação da interface obrigatória para novos Spokes de reputação.
- **[Ciclo de Vida RWA](./docs/architecture/RWA_LIFECYCLE.md)**: Implementação de estados dinâmicos para ativos físicos on-chain.
- **[Fluxo ZK-Email](./docs/architecture/ZK_EMAIL_FLOW.md)**: Protocolo de verificação de provas ZK para fluxos de caixa Web2.
- **[Interoperabilidade](./docs/architecture/INTEROP.md)**: Arquitetura de mensageria cross-chain e Adapter Pattern.

### 🛠️ Implementação e Integração
- **[Guia de Integração para Lending](./docs/specs/2026-04-27-lending-integration-spec.md)**: Documentação de API para protocolos de crédito consumirem o Hub.
- **[Comandos Úteis (Cheatsheet)](./docs/guides/CHEATSHEET.md)**: Guia rápido para desenvolvimento, deploy e interação via Soroban CLI.
- **[Guia de Interoperabilidade Axelar](./docs/guides/AXELAR_INTEROP_GUIDE.md)**: Detalhes técnicos da ponte Stellar <-> EVM.
- **[Especificações de Design](./docs/specs/)**: Histórico de decisões técnicas e design de funcionalidades.

## 🤖 Automação e IA
Este repositório utiliza uma [Skill de Desenvolvimento Interna](./docs/internal/SKILL.md) para garantir que agentes de IA mantenham os padrões de segurança "Armor Up" e a integridade da documentação técnica.

## ⚡ Quick Start (Makefile)

Utilize o `Makefile` para gerenciar o ciclo de vida do desenvolvimento:

```bash
# Compilar todos os contratos (Stellar WASM & EVM Solidity)
make build

# Executar a suíte completa de testes unitários e integração
make test

# Executar auditoria de segurança (Slither, Cargo Audit, Clippy)
make audit

# Formatação e Linting
make fmt && make lint
```

## 🛠️ Requisitos
- **Rust/Cargo**: Toolchain `stable` com target `wasm32-unknown-unknown`.
- **Stellar CLI**: Para deploy e interação com a rede Soroban.
- **Foundry**: Necessário para compilação e testes dos contratos EVM.
- **Slither**: Para análise estática de segurança em Solidity.

---
*Este repositório é focado estritamente na lógica de contratos inteligentes e infraestrutura de segurança do protocolo Zolvency.*
