
use ed25519_dalek::{Keypair, Signer};
use rand::rngs::OsRng;

fn main() {
    let mut csprng = OsRng{};
    let keypair: Keypair = Keypair::generate(&mut csprng);
    
    println!("Public Key: {:02x?}", keypair.public.to_bytes());
    
    // We need to match the XDR encoding of Soroban
    // Symbol "USD" in XDR
    // i128 10_000_000 in XDR
    // u64 1000 in XDR
    
    // For simplicity, I'll just use a random keypair and test the failure case first.
}
