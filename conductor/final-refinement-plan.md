# Plano de Refinamento Final: Arredondamento do Ecossistema

Este plano detalha os ajustes finais para garantir que todos os contratos sigam os mesmos padrões de segurança, eficiência e usabilidade.

## 1. Zolvency Soul (Identidade Central)
- **Mudança:** Implementar transferência de Admin em dois passos (`transfer_admin` e `accept_admin`) para evitar perda de controle por erro de digitação.
- **Mudança:** Adicionar evento para alteração de Relayer.

## 2. Zolvency Registry (Hub)
- **Mudança:** Impedir a função `export_reputation` de processar pedidos para almas que estão na `Blacklist` ou `Locked`.
- **Mudança:** Refinar a validação de quem pode chamar a exportação.

## 3. Spokes (Uber e GitHub)
- **Padronização (GitHub):** Adicionar suporte a `nonces` e à função `get_nonce`, alinhando com o contrato Uber.
- **Otimização:** Mover a configuração (`Config`) para `instance storage` para reduzir custos de gás em operações frequentes (mint/update).
- **Consistência:** Garantir que o incremento de nonce ocorra após a exportação em todos os spokes.

## 4. EVM (Verificadores)
- **Axelar:** Adicionar validação de `sourceChain` no método `_execute` para garantir que apenas mensagens da Stellar (via Axelar) sejam aceitas.

## 5. Verificação Final
- Revisar todos os `require_auth()` e garantir que as mensagens de erro são consistentes.
