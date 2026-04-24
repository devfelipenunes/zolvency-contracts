# Product Requirements Document (PRD): Zolvency Protocol

**Versão:** 1.0  
**Status:** Draft  
**Data:** 24 de Abril de 2026  
**Autor:** Gemini CLI & Felipe Nunes  

---

## 1. Visão Geral do Produto
O **Zolvency** é a camada de crédito descentralizada da rede Stellar. Ele permite a criação de Soulbound Tokens (SBTs) que representam a reputação verificável de um usuário (GitHub, Renda Bancária, Histórico On-chain), funcionando como um "score de crédito web3".

### 1.1 Problema
Atualmente, protocolos de empréstimo (Lending) em DeFi são sobre-colateralizados (over-collateralized) porque não há uma forma confiável de avaliar o risco do tomador. Isso torna o capital ineficiente.

### 1.2 Solução
O Zolvency fornece infraestrutura para que protocolos consultem a "saúde financeira" e a "reputação técnica" de um usuário através de identidades digitais soberanas, permitindo taxas de juros menores e maiores multiplicadores de LTV (Loan-to-Value).

---

## 2. Objetivos Estratégicos
- **Modularidade:** Permitir que qualquer fonte de dados (ZK-Email, Reclaim, KYC) se torne um "Spoke" de reputação.
- **Interoperabilidade:** Exportar a reputação gerada na Stellar para ecossistemas EVM (Ethereum, L2s).
- **Segurança Adaptável:** Oferecer segurança de hardware (Passkeys) sem comprometer o onboarding rápido.

---

## 3. Público-Alvo
1. **Usuários Finais:** Desenvolvedores e investidores que desejam usar sua reputação off-chain para obter melhores condições em DeFi.
2. **Protocolos DeFi:** Plataformas de lending que precisam de novos parâmetros de análise de risco.
3. **Emissores de Identidade:** Projetos que possuem dados de usuários e querem monetizar ou descentralizar essa informação.

---

## 4. Requisitos Funcionais (Core Features)

### 4.1 Hub de Reputação (Registry)
- **Centralização de Spikes:** Um contrato central que lista todos os emissores de SBT confiáveis.
- **API Unificada:** Uma única função para consultar todo o score do usuário em diferentes tipos de tokens.

### 4.2 Identidade GitHub (SBT)
- **Mint via ZK-Proof:** Emissão de tokens baseada em provas de atividade real no GitHub.
- **Tiering Automático:** Classificação do usuário (Novice, Pro, Architect, etc.) baseada em contribuições.
- **Resistência a Sybil:** Impedir que uma única conta do GitHub gere múltiplos tokens para endereços diferentes.

### 4.3 Segurança e Passkey (Opt-in)
- **Vínculo Opcional:** O usuário pode escolher vincular seu token a uma Passkey (WebAuthn) para garantir que apenas seu dispositivo físico possa autorizar atualizações.
- **Fluxo sem Atrito:** Possibilidade de mintar identidades sem passkey para testes e onboarding rápido.

### 4.4 Multi-Protocol Push (Cross-chain)
- **Sincronização Automática:** Ao atualizar a reputação na Stellar, o dado é enviado via Axelar ou LayerZero para contratos veroficadores na EVM.
- **Modelo Pull:** Suporte para verificação off-chain via assinaturas de autoridade.

---

## 5. Requisitos Não-Funcionais
- **Latência:** Atualizações cross-chain devem ser finalizadas em menos de 3 minutos.
- **Custo:** O custo de mint na Stellar deve ser inferior a $0.05 USD.
- **Escalabilidade:** O Registry deve suportar até 100 tipos diferentes de emissores de reputação.

---

## 6. Arquitetura Técnica (Resumo)
- **Backend:** Soroban Smart Contracts (Rust).
- **Frontend/SDK:** TypeScript SDK para integração fácil com dApps.
- **Oráculos:** ZK-Email para validação de DKIM e Reclaim Protocol para provas de APIs.

---

## 7. Roadmap de Produto

### Fase 1: Fundação (Atual)
- [x] Hub Registry implementado.
- [x] SBT de GitHub funcional.
- [x] Interoperabilidade básica via Axelar.
- [x] Passkeys opcionais implementadas.

### Fase 2: Expansão de Dados (Q3 2026)
- [ ] **Bank-SBT:** Integração com extratos bancários via ZK-Email.
- [ ] **On-chain History Spoke:** Análise de histórico de trocas e liquidez na Stellar DEX.
- [ ] **Zolvency Score:** Algoritmo ponderado para gerar um número de score (0-1000).

### Fase 3: Ecossistema (Q4 2026)
- [ ] Lançamento do SDK Oficial.
- [ ] Parceria com o primeiro protocolo de Lending para LTV diferenciado.
- [ ] Governança (Zolvency DAO) para curadoria de emissores no Registry.

---

## 8. Critérios de Sucesso
- 5.000 identidades mintadas nos primeiros 3 meses.
- Integração com pelo menos 2 protocolos de Lending na testnet.
- Taxa de erro em transações cross-chain inferior a 1%.
