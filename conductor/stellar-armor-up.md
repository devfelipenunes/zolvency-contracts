# Stellar Contracts - "Armor Up" Refactoring Plan

## Objective
Refatorar e otimizar os contratos do Zolvency na rede Stellar (`zolvency-registry` e `github-identity`), focando em escalabilidade no armazenamento, gestão consistente de aluguel (TTL/Rent) e padronização de segurança e erros.

## Key Files & Context
- `contracts/zolvency-registry/src/lib.rs`
- `contracts/github-identity/src/storage.rs`
- `contracts/github-identity/src/lib.rs`
- Possivelmente outros contratos (ex: `uber-income`) que seguem o mesmo padrão.

## Implementation Steps

### 1. Escalabilidade do Hub (`Nexus`)
*   **Refatorar o Armazenamento de Tokens:** Substituir o uso de `Vec<Address>` no `DataKey::Tokens`. Atualmente, cada novo token registrado exige a leitura e reescrita de toda a lista. Vamos alterar para um sistema de `TokenCount` (contador) mapeado com índices individuais (`DataKey::Token(u32)`), ou manter um array pequeno apenas para leitura rápida, otimizando as chamadas.
*   **Tratamento de Erros Profissional:** Remover os `panic!("Not admin")` e introduzir um `enum Error` focado em baixo consumo de *gas*.
*   **Gerenciamento de TTL (Rent):** Adicionar chamadas de `extend_ttl` para as chaves `Admin`, `Signer` e configurações persistentes sempre que forem lidas ou escritas, evitando que o Registry congele por falta de pagamento do aluguel do Soroban.

### 2. Padronização de Storage (Spokes - ex: `github-identity`)
*   **Auditoria de TTL:** Garantir que todos os dados críticos no `storage.rs` recebam o devido `extend_ttl(ONE_YEAR, ONE_YEAR)`.
*   **Limpeza de Código:** Verificar a padronização dos modificadores e das constantes `DAY_IN_LEDGERS` para garantir cálculos de tempo corretos entre diferentes contratos da plataforma.

### 3. Fortalecimento da Interoperabilidade (Cross-Chain)
*   **Garantia de Identidade:** Validar se o payload emitido pelo `messenger.rs` está enviando o `nonce` correto (se aplicável ao design final) ou garantindo que a emissão (push) contém metadados suficientes para o EVM prevenir Replay Attacks.

## Verification & Testing
1.  Compilar os contratos via `soroban contract build`.
2.  Atualizar e rodar os testes em `zolvency-registry/src/test.rs` para assegurar que a nova lógica de `TokenCount` / armazenamento não quebra a busca (`get_user_reputation`).
3.  Verificar os custos de Storage Rent nas emissões locais.
