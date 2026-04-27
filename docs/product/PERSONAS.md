# Zolvency Personas: User and Partner Journeys

## 1. O Usuário Final: "Felipe, o Desenvolvedor"
- **Perfil:** Dev Senior, possui histórico no GitHub, mas não quer vender seus Bitcoins para pegar um empréstimo de curto prazo.
- **Jornada:**
    1. Felipe minta seu **GitHub SBT** no Zolvency.
    2. Ele ative o **Tier Premium** via Passkey para provar que é ele mesmo.
    3. Ao solicitar um empréstimo no "Stellar Lend", o protocolo consulta o Zolvency Registry.
    4. O Zolvency trava o score dele (Lock).
    5. Felipe recebe o crédito com juros 5% menores.

## 2. O Parceiro de Ecossistema: "LendProtocol (EVM)"
- **Perfil:** Protocolo de Lending na rede Sepolia (Ethereum) que quer atrair usuários da Stellar.
- **Jornada:**
    1. O LendProtocol integra o **Zolvency SDK**.
    2. Eles usam o **Axelar Adapter** para verificar se o usuário da Stellar tem um SBT válido.
    3. Eles confiam no score porque sabem que, se o usuário não pagar, a reputação dele será queimada na rede de origem.

## 3. O Agente de IA: "Zolv-Bot"
- **Perfil:** Agente de IA que gerencia o portfólio de um usuário de forma autônoma.
- **Jornada:**
    1. O Zolv-Bot lê o `TokenMetadata` no Hub para entender quais ativos o usuário pode colateralizar.
    2. O bot usa a Passkey (via autorização prévia) para atualizar as provas de vida (commits no GitHub) do usuário, evitando o **Reputation Decay**.
