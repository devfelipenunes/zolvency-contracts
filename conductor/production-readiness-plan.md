# Plano de Prontidão para Produção: Auditoria e Refinamento

Este plano detalha as ações finais de "limpeza de código" e "endurecimento de segurança" para levar os contratos ao estado final de produção.

## 1. Limpeza Estética e Eficiência (Stellar)
- **Ação:** Remover todos os comentários obsoletos (ex: `// Tenta buscar o token_id`), logs de debug comentados e explicações óbvias que consomem espaço no WASM.
- **Ação:** Simplificar mensagens de erro em contratos Solidity (mensagens curtas economizam gás na implantação e execução).

## 2. Padronização de Nonces e Segurança (Spokes)
- **Ação (GitHub):** Garantir que o `increment_nonce` ocorra de forma atômica e após a exportação bem-sucedida, alinhando com o `UberIncomeContract`.
- **Ação (GitHub):** Implementar a função `get_nonce` no `GithubIdentityContract` para permitir que o front-end consulte o valor atual antes do mint.

## 3. Reforço de Segurança Cross-Chain (EVM)
- **Ação (Axelar):** Validar se o `sourceChain` é exatamente "stellar" (ou a string configurada) no método `_execute`, prevenindo ataques de personificação entre cadeias.
- **Ação (Authority):** Adicionar `require` de nonce válido e incremento obrigatório para evitar replay de assinatura.

## 4. Otimização de Storage (Hub & Spokes)
- **Ação:** Verificar se todos os contratos utilizam `instance()` para configurações globais (lidas em cada transação) e `persistent()` apenas para dados de usuários/tokens (esparsos).
- **Ação:** Garantir que o Registry respeite o `is_soul_locked` antes de permitir exportações.

## 5. Verificação e Testes
- Executar `make test` em todos os pacotes.
- Verificar o tamanho do binário WASM gerado (`soroban contract optimize`) para garantir que está dentro dos limites da rede.

---
**Objetivo Final:** Código limpo, sem redundâncias, com proteção contra replays em todas as frentes e pronto para o deploy oficial.
