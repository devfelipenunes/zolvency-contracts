const { Keypair } = require('@stellar/stellar-sdk');
const secret = 'SCLMARUQYABVBJ3O7K57FVSDVBTZ6OD4W6KTIGQKNXX3DUH4DUK7UMBO';
const kp = Keypair.fromSecret(secret);
console.log(kp.publicKey());
