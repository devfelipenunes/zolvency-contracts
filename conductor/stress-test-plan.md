# Plano de Stress Testing e Blindagem Final

Este plano descreve os cenários extremos que serão testados para garantir que o ecossistema Zolvency seja resiliente em Mainnet.

## 1. Stress no Registry (Stellar)
- **Cenário:** Registro de 50 tokens (acima do limite de 20).
- **Verificação:** 
    - Garantir que `get_soul_reputation` (sem lista) não dê panic e respeite o limite de 20.
    - Garantir que um token registrado na posição 21 possa ser consultado se passado explicitamente na lista.
    - Medir o consumo de instruções CPU para 20 chamadas cross-contract.

## 2. Teste de Limite de Payload (Soroban Storage)
- **Cenário:** Tentar um `mint` no Uber Spoke com um `proof_data` de 40KB, 60KB e 100KB.
- **Verificação:** 
    - Identificar o ponto exato de falha do Soroban.
    - Garantir que a falha seja um erro limpo e não corrompa o storage.

## 3. Simulação de Ataque de Replay (EVM)
- **Cenário:** Gerar uma assinatura válida para o contrato na rede A (ChainID 1). Tentar usar a mesma assinatura no contrato na rede B (ChainID 2).
- **Verificação:** 
    - O contrato deve rejeitar com `INVALID_AUTH` devido à diferença de Domain Hash.
    - Testar reuso da mesma assinatura no mesmo contrato (deve falhar pelo Nonce).

## 4. Atomicidade do Nonce em Falhas (Spokes)
- **Cenário:** Simular uma falha na chamada `export_reputation` (ex: soul bloqueada).
- **Verificação:** 
    - O `mint` no Spoke deve reverter totalmente.
    - O nonce do usuário **não** deve ser incrementado, permitindo nova tentativa após resolver o bloqueio.

## 5. Benchmarking de Custos
- **Ação:** Coletar os valores de `Instructions`, `ReadBytes` e `WriteBytes` para um fluxo completo (Soul Mint -> Uber Mint -> Registry Export).
- **Objetivo:** Projetar o custo em Lumens (XLM) para os usuários finais.

---
**Status:** Pronto para execução após saída do Plan Mode.
