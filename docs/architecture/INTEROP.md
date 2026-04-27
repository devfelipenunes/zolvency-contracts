# Zolvency Interoperability Guide

Este documento explica como gerenciar o sistema de interoperabilidade modular da Zolvency.

## 1. Arquitetura Modular
O contrato `GithubIdentityContract` (Stellar) utiliza o **Adapter Pattern** para despachar mensagens de reputação. Cada adaptador é um contrato independente.

Protocolos suportados:
- **Axelar GMP:** Robusto, baseado em gateways. Ideal para "Push" automático.
- **Authority-Pull:** Quase custo zero no Stellar. Emite eventos que são assinados off-chain e puxados pelo usuário na EVM.
- **LayerZero V2:** Ultra-rápido, baseado em OApps.

## 2. Como trocar o protocolo ativo
A troca é feita apontando para o contrato do adaptador correspondente.

### Para ativar o Axelar:
```bash
# 1. Deploy do Adaptador Axelar
# 2. Configurar o Adaptador
stellar contract invoke --id <AXELAR_ADAPTER_ID> --source admin --network testnet -- \
  initialize --admin <ADMIN> --gateway <GW> --gas_service <GAS> --gas_token <TOKEN>

# 3. Ativar na Identidade
stellar contract invoke --id <IDENTITY_ID> --source admin --network testnet -- \
  set_active_protocol --admin <ADMIN> --protocol Axelar --adapter <AXELAR_ADAPTER_ID>
```

### Para ativar o Authority-Pull:
```bash
# 1. Deploy do Adaptador Authority-Pull
# 2. Ativar na Identidade
stellar contract invoke --id <IDENTITY_ID> --source admin --network testnet -- \
  set_active_protocol --admin <ADMIN> --protocol None --adapter <AUTHORITY_ADAPTER_ID>
```
*(Nota: Use `None` se o protocolo não exigir lógica Push complexa ou implemente uma nova variant no enum).*

## 3. Comparativo Técnico

| Característica | Axelar GMP | Authority-Pull | LayerZero V2 |
| :--- | :--- | :--- | :--- |
| **Modelo** | Push (Gateway) | Pull (Signatures) | Push (OApp) |
| **Custo Stellar** | ~15 XLM (Gás EVM) | ~0.1 XLM (Evento) | Variável |
| **Latência** | ~2-5 min | Instantâneo | < 1 min |
| **UX** | Automática | Requer 2 passos | Automática |


## 5. Próximos Passos Recomendados
- [x] Implementar o terceiro adaptador: **Authority-Pull** (Assinaturas off-chain) para custo quase zero no Stellar.
- [ ] Monitorar o **LayerZero Scan** e o **Axelarscan** para validar as provas de entrega.
- [ ] Implementar adaptador nativo **LayerZero V2** (OApp) para Stellar.
