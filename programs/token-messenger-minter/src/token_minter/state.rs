//! State

use {
    crate::token_minter::error::TokenMinterError,
    anchor_lang::prelude::*,
    anchor_spl::token::{Burn, Transfer},
};

#[account]
#[derive(Debug, InitSpace)]
pub struct TokenMinter {
    pub token_controller: Pubkey,
    pub pauser: Pubkey,
    pub paused: bool,
    pub bump: u8,
}

#[account]
#[derive(Debug, InitSpace)]
pub struct TokenPair {
    pub remote_domain: u32,
    pub remote_token: Pubkey,
    pub local_token: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(Debug, InitSpace)]
pub struct LocalToken {
    pub custody: Pubkey,
    pub mint: Pubkey,
    pub burn_limit_per_message: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub amount_sent: u64,
    pub amount_received: u64,
    pub bump: u8,
    pub custody_bump: u8,
}

impl TokenMinter {
    pub fn validate(&self) -> bool {
        self.token_controller != Pubkey::default() && self.pauser != Pubkey::default()
    }

    pub fn burn<'info>(
        &self,
        mint: AccountInfo<'info>,
        from: AccountInfo<'info>,
        authority: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        local_token: &mut LocalToken,
        amount: u64,
    ) -> Result<()> {
        require!(!self.paused, TokenMinterError::ProgramPaused);
        require_gte!(
            local_token.burn_limit_per_message,
            amount,
            TokenMinterError::BurnAmountExceeded
        );

        local_token.messages_sent = local_token.messages_sent.wrapping_add(1);
        local_token.amount_sent = local_token.amount_sent.wrapping_add(amount);

        let context: CpiContext<'_, '_, '_, '_, Burn<'_>> = CpiContext::new(
            token_program,
            Burn {
                mint,
                from,
                authority,
            },
        );

        anchor_spl::token::burn(context, amount)
    }

    pub fn transfer<'info>(
        &self,
        from: AccountInfo<'info>,
        to: AccountInfo<'info>,
        authority: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        local_token: &mut LocalToken,
        amount: u64,
    ) -> Result<()> {
        require!(!self.paused, TokenMinterError::ProgramPaused);

        local_token.messages_received = local_token.messages_received.wrapping_add(1);
        local_token.amount_received = local_token.amount_received.wrapping_add(amount);

        let authority_seeds: &[&[&[u8]]] = &[&[b"token_minter", &[self.bump]]];

        let context = CpiContext::new(
            token_program,
            Transfer {
                from,
                to,
                authority,
            },
        )
        .with_signer(authority_seeds);

        anchor_spl::token::transfer(context, amount)
    }

    pub fn close_token_account<'info>(
        &self,
        receiver: AccountInfo<'info>,
        token_account: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        authority: AccountInfo<'info>,
    ) -> Result<()> {
        require!(!self.paused, TokenMinterError::ProgramPaused);

        let authority_seeds: &[&[&[u8]]] = &[&[b"token_minter", &[self.bump]]];

        let cpi_accounts = anchor_spl::token::CloseAccount {
            account: token_account,
            destination: receiver,
            authority,
        };
        let cpi_context = anchor_lang::context::CpiContext::new(token_program, cpi_accounts);

        anchor_spl::token::close_account(cpi_context.with_signer(authority_seeds))
    }
}

impl TokenPair {
    pub fn validate(&self) -> bool {
        self.remote_token != Pubkey::default() && self.local_token != Pubkey::default()
    }
}

impl LocalToken {
    pub fn validate(&self) -> bool {
        self.custody != Pubkey::default() && self.mint != Pubkey::default()
    }
}