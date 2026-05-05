// contracts/contracts/zpay/src/lib.rs
#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

pub mod nexus_interface;

#[contract]
pub struct ZPayContract;

#[contractimpl]
impl ZPayContract {
    pub fn hello(_env: Env) -> u32 {
        1
    }
}

#[cfg(test)]
mod test;
