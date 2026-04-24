# Design Spec: Tornar Passkey Opcional

**Data:** 2026-04-24  
**Status:** Approved  
**Autor:** Gemini CLI  

## 1. Objetivo
Transformar a funcionalidade de Passkey (secp256r1) em um recurso opcional no contrato `github-identity`. Atualmente, o `mint` exige obrigatoriamente uma chave pública de passkey e uma assinatura válida, o que dificulta testes e integração rápida.

## 2. Motivação
- Facilitar o onboarding de usuários que ainda não possuem suporte a Passkeys.
- Simplificar scripts de automação e validação que falham devido a placeholders de tamanho incorreto.
- Seguir as melhores práticas de Rust/Soroban utilizando o tipo `Option`.

## 3. Alterações Propostas

### 3.1. Contrato: `github-identity`

#### `types.rs`
Alterar as structs para suportar valores opcionais:
- **`MintParams`**:
  - `passkey`: `BytesN<65>` -> `Option<BytesN<65>>`
  - `passkey_signature`: `BytesN<64>` -> `Option<BytesN<64>>`
- **`GithubData`**:
  - `passkey`: `BytesN<65>` -> `Option<BytesN<65>>`

#### `interface.rs`
- **`get_owner_passkey`**: Alterar o retorno de `BytesN<65>` para `Option<BytesN<65>>`.

#### `lib.rs` (Lógica de `mint`)
- Implementar lógica condicional:
  - Se `passkey` E `passkey_signature` forem `Some`, executa `env.crypto().secp256r1_verify(...)`.
  - Se ambos forem `None`, pula a validação.
  - Se apenas UM for fornecido, retorna `Error::InvalidSignature` (garante integridade).
- Ajustar a persistência no `GithubData`.

### 3.2. Integração e Testes
- **`test.rs`**: Atualizar stubs e casos de teste. Adicionar um teste específico para "mint sem passkey".
- **`zolvency-registry/src/test.rs`**: Atualizar a chamada do `mint` no teste de integração para usar `None` nos campos de passkey.
- **`scripts/validate_final.js`**: Alterar os parâmetros de passkey para `null`.

## 4. Documentação
- Atualizar `ARCHITECTURE.md` para descrever a Passkey como um recurso de segurança opcional ("Opt-in hardware security").

## 5. Plano de Verificação
1. Executar testes unitários do `github-identity` (`cargo test`).
2. Executar testes de integração do `zolvency-registry`.
3. Verificar se o contrato compila com sucesso via Soroban CLI.
