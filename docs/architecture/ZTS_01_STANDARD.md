# Zolvency Technical Standard 01 (ZTS-01)
## Standard Interface and Behavior for Reputation Spokes

**Versão:** 1.0  
**Status:** Strategic Baseline  
**Data:** 27 de Abril de 2026  
**Autor:** Zolvency Core & Gemini CLI (v4-Author-Auditor)

---

## 1. Abstract
O ZTS-01 define a interface e o comportamento obrigatório para qualquer contrato inteligente que deseje atuar como um **Spoke** (emissor de reputação) dentro do ecossistema Zolvency na rede Stellar. A conformidade com este padrão garante que o `ZolvencyRegistry` possa agregar o score do usuário e que protocolos de Lending possam realizar underwriting cross-asset de forma uniforme.

---

## 2. The ZolvencyTokenTrait Interface

Todo Spoke compatível DEVE implementar a seguinte trait Soroban:

```rust
pub trait ZolvencyTokenTrait {
    // Retorna o identificador do tipo (ex: "github", "bank", "real-estate")
    fn get_token_type(env: Env) -> Symbol;

    // Retorna a fonte de dados primária (ex: "zk-email-dkim", "iot-sensor")
    fn get_source(env: Env) -> String;

    // Retorna metadados estruturados (ZTS-02 Standard)
    fn get_metadata(env: Env) -> TokenMetadata;

    // Validação de negócio (Check de expiração e status legal)
    fn is_valid(env: Env, token_id: u64) -> bool;

    // Retorna o timestamp UNIX de expiração
    fn get_expiry(env: Env, token_id: u64) -> u64;

    // Retorna a chave pública da Passkey vinculada (opcional)
    fn get_owner_passkey(env: Env, token_id: u64) -> Option<BytesN<65>>;
}
```

---

## 3. Comportamentos Obrigatórios

### 3.1 Sybil Resistance (Resistência a Sibil)
Um Spoke DEVE garantir que um identificador externo único (ex: GitHub ID ou Registro de Imóvel) não possa ser usado para emitir múltiplos tokens ativos para carteiras diferentes simultaneamente.
- **Implementação Recomendada:** Mapeamento `hash(external_id) -> token_id`. Se um novo `mint` for solicitado para um `external_id` já existente, o token anterior DEVE ser invalidado ou transferido.

### 3.2 Proof of Freshness (Decay)
Reputação é dinâmica. Um Spoke DEVE implementar uma lógica de expiração (`Business TTL`).
- Recomenda-se um período padrão de **90 dias**.
- Após este período, `is_valid` deve retornar `false` ou o score deve sofrer decaimento linear até que uma nova prova seja apresentada.

### 3.3 Registry Integration
Ao ser inicializado, um Spoke DEVE registrar seu endereço no `ZolvencyRegistry` central para ser incluído na agregação global de score.

---

## 4. Estrutura de Metadados (ZTS-02)

Os metadados retornados por `get_metadata` devem seguir o padrão:
- `name`: Nome legível do tipo de reputação.
- `symbol`: Sigla curta (ex: ZGH para Zolvency GitHub).
- `version`: Versão do esquema de dados.
- `data_source`: Link ou identificador da fonte de auditoria.

---

## 5. Conclusão
O cumprimento do ZTS-01 permite a expansão orgânica do Hub. Desenvolvedores terceiros podem criar "SBTs de Performance Solar", "SBTs de Histórico de Aluguel" ou qualquer outra métrica de confiança, integrando-se instantaneamente à liquidez do ecossistema Zolvency.

---
*[[ZTS-01]], [[Spoke-Compliance]], [[Reputation-Standard]]*
