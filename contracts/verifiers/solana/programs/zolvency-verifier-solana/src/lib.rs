use anchor_lang::prelude::*;
use solana_axelar_gateway::cpi::accounts::ValidateMessage;
use solana_axelar_gateway::program::SolanaAxelarGateway;
use borsh::{BorshDeserialize, BorshSerialize};

declare_id!("FM344TprtFfP39Q4Td4ZXpamaCLfhDc4Qa61ygpGcou8");

#[program]
pub mod zolvency_verifier_solana {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, admin: Pubkey) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = admin;
        Ok(())
    }

    /// Executa o comando recebido via Axelar GMP.
    /// Esta função é chamada pelo Relayer do Axelar após a validação na rede.
    pub fn execute(
        ctx: Context<Execute>,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload: Vec<u8>,
    ) -> Result<()> {
        // 1. Validar a mensagem via Axelar Gateway (CPI)
        let gateway_program = ctx.accounts.gateway_program.to_account_info();
        let validate_accounts = ValidateMessage {
            gateway_config: ctx.accounts.gateway_config.to_account_info(),
            message_payload: ctx.accounts.message_payload.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(gateway_program, validate_accounts);
        solana_axelar_gateway::cpi::validate_message(
            cpi_ctx,
            source_chain.clone(),
            message_id,
            source_address.clone(),
            anchor_lang::solana_program::keccak::hash(&payload).to_bytes(),
        )?;

        // 2. Decodificar o payload
        
        // REPUTATION = 1
        if payload[0] == 1 {
            let mut data = &payload[1..];
            let soul_id = u32::deserialize(&mut data).map_err(|_| ErrorCode::InvalidPayload)?;
            let _user_address = <[u8; 32]>::deserialize(&mut data).map_err(|_| ErrorCode::InvalidPayload)?;
            let _external_id_hash = <[u8; 32]>::deserialize(&mut data).map_err(|_| ErrorCode::InvalidPayload)?;
            let _tier = u32::deserialize(&mut data).map_err(|_| ErrorCode::InvalidPayload)?;
            let _nonce = u64::deserialize(&mut data).map_err(|_| ErrorCode::InvalidPayload)?;
            let _token_type_hash = <[u8; 32]>::deserialize(&mut data).map_err(|_| ErrorCode::InvalidPayload)?;

            msg!("Reputation Updated on Solana for Soul {}", soul_id);
            // TODO: Implementar armazenamento de reputação se necessário na Solana
        }
        // WILL_AUTH = 2
        else if payload[0] == 2 {
            let mut data = &payload[1..];
            let soul_id = u32::deserialize(&mut data).map_err(|_| ErrorCode::InvalidPayload)?;
            let will_address = <[u8; 32]>::deserialize(&mut data).map_err(|_| ErrorCode::InvalidPayload)?;
            let permissions = u64::deserialize(&mut data).map_err(|_| ErrorCode::InvalidPayload)?;
            let expiry = u64::deserialize(&mut data).map_err(|_| ErrorCode::InvalidPayload)?;

            // 3. Salvar a permissão no PDA do Will
            let will_permission = &mut ctx.accounts.will_permission;
            will_permission.soul_id = soul_id;
            will_permission.will_address = will_address;
            will_permission.expiry = expiry;
            will_permission.is_active = true;

            msg!("Will Authorized on Solana: Soul {} -> Will {:?}", soul_id, will_address);
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + 32)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(source_chain: String, message_id: String, source_address: String, payload: Vec<u8>)]
pub struct Execute<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + 4 + 32 + 8 + 1,
        seeds = [b"will", payload[1..5].as_ref()], // Usa soul_id como semente (simplificado)
        bump
    )]
    pub will_permission: Account<'info, WillPermission>,

    /// Axelar Accounts
    pub gateway_program: Program<'info, SolanaAxelarGateway>,
    /// [CHECK] Validado via CPI no Gateway
    pub gateway_config: UncheckedAccount<'info>,
    /// [CHECK] Validado via CPI no Gateway
    pub message_payload: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct Config {
    pub admin: Pubkey,
}

#[account]
pub struct WillPermission {
    pub soul_id: u32,
    pub will_address: [u8; 32],
    pub expiry: u64,
    pub is_active: bool,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid payload received from Axelar")]
    InvalidPayload,
}
