use anchor_lang::prelude::*;
/* use quarry_mine::{Quarry,Rewarder,Miner}; */
use anchor_spl::token::TokenAccount;
use anchor_spl::token::Token;
use serum_swap::Side;
use serum_swap::ExchangeRate;
declare_id!("9C8SwwbnvtzZM95yuPePtiPvwCmu6fKTGE5CcqK2S1un");

#[program]
pub mod test1 {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        Ok(())
    }
   /*  pub fn sey_hello(ctx:Context<AccountH>,nonce:u8)-> ProgramResult{
        let owner=*ctx.accounts.owner.key;
        msg!("hello karima owner {:?}",owner);
        let program_id=*ctx.accounts.program_id.key;
        msg!("hello karima program_id {:?}",program_id);
        let program_id_info=ctx.accounts.program_id.clone( );
        let cpi_accounts=StructAccount{
            owner: ctx.accounts.owner.to_account_info(),
        };
        let program_address=ctx.accounts.program_address.clone();
        let account_program_address=ctx.accounts.account_program_address.clone();
        let program_principal=ctx.accounts.program_principal.clone();
        let expected_allocated_key =Pubkey::create_program_address(&[&account_program_address.key.to_bytes()[..32], &[nonce]], program_principal.key)?;
    if *program_address.key != expected_allocated_key {
    // allocated key does not match the derived address
    return Err(ProgramError::InvalidArgument);
    }
    let seeds=&[&account_program_address.key.to_bytes()[..32], &[nonce]];
    let signer_seeds = &[&seeds[..]];
    //let signer_seeds =[&[&[&account_program_address.key.to_bytes()[..32], &[nonce]]]];
    msg!(" cpi_accounts");    
    let cpi_ctx = CpiContext::new_with_signer(program_id_info, cpi_accounts,signer_seeds);
        msg!(" cpi_ctx");
        serum_swap::cpi::hello(cpi_ctx);

        Ok(())
    } */
    pub fn swap(ctx:Context<SwapInfo>)->ProgramResult{
        let market1=ctx.accounts.market.market.to_account_info();
        let open_orders=ctx.accounts.market.open_orders.to_account_info();
        let request_queue=ctx.accounts.market.request_queue.to_account_info();
        let event_queue=ctx.accounts.market.event_queue.to_account_info();
        let bids=ctx.accounts.market.bids.to_account_info();
        let asks=ctx.accounts.market.asks.to_account_info();
        let order_payer_token_account=ctx.accounts.market.order_payer_token_account.to_account_info();
        let coin_vault=ctx.accounts.market.coin_vault.to_account_info();
        let pc_vault=ctx.accounts.market.pc_vault.to_account_info();
        let vault_signer=ctx.accounts.market.vault_signer.to_account_info();
        let coin_wallet=ctx.accounts.market.coin_wallet.to_account_info();
        let program_id_swap_info=ctx.accounts.program_id_swap.clone( );

        let market=serum_swap::cpi::accounts::MarketAccounts{
            market:market1,
            open_orders,
            request_queue,
            event_queue,
            bids,
            asks,
            order_payer_token_account,
            coin_vault,
            pc_vault,
            vault_signer,
            coin_wallet
        };
         let cpi_accounts=serum_swap::cpi::accounts::Swap{
            market,
            authority:ctx.accounts.authority.to_account_info(),
            pc_wallet:ctx.accounts.pc_wallet.to_account_info(),
            dex_program:ctx.accounts.dex_program.to_account_info(),
            token_program:ctx.accounts.token_program.to_account_info(),
            rent:ctx.accounts.rent.to_account_info()
        }; 
        msg!("4");
             let rate:u64 = 1;
            let from_decimals:u8 = 2;
            let quote_decimals:u8 = 2;
            let strict:bool=false;
            let amount:u64=200;
            let min_exchange_rate=serum_swap::ExchangeRate{ rate, from_decimals, quote_decimals, strict };
            let side=serum_swap::Side::Bid; 
            msg!("5");
            
            let side=serum_swap::Side::Bid;
        let cpi_ctx = CpiContext::new(program_id_swap_info, cpi_accounts);
        msg!("6");
        serum_swap::cpi::swap(cpi_ctx,side,amount,min_exchange_rate);
       
     Ok(())
    } 
  /*   pub fn stake(ctx:Context<StakeUser>,amount:u64,_nonce:u64)->ProgramResult{
        let _authority=ctx.accounts.authority.clone();
        let quarry_program=ctx.accounts.quarry_program.clone( );
 
         let cpi_accounts=quarry_mine::cpi::accounts::UserStake{
            authority:ctx.accounts.authority.to_account_info(),
            miner:ctx.accounts.miner.to_account_info(),
            quarry:ctx.accounts.quarry.to_account_info(),
            miner_vault:ctx.accounts.miner_vault.to_account_info(),
            token_account:ctx.accounts.token_account.to_account_info(),
            token_program:ctx.accounts.token_program.to_account_info(),
            rewarder:ctx.accounts.rewarder.to_account_info(),
            unused_clock:ctx.accounts.unused_clock.to_account_info(),
        }; 
        let cpi_ctx = CpiContext::new(quarry_program, cpi_accounts);
        quarry_mine::cpi::stake_tokens(cpi_ctx,amount);
       
     Ok(())
    } */
   /*  pub fn withdrow(ctx:Context<StakeUser>,amount:u64)->ProgramResult{
        let _authority=ctx.accounts.authority.clone();
        let quarry_program=ctx.accounts.quarry_program.clone( );
 
         let cpi_accounts=quarry_mine::cpi::accounts::UserStake{
            authority:ctx.accounts.authority.to_account_info(),
            miner:ctx.accounts.miner.to_account_info(),
            quarry:ctx.accounts.quarry.to_account_info(),
            miner_vault:ctx.accounts.miner_vault.to_account_info(),
            token_account:ctx.accounts.token_account.to_account_info(),
            token_program:ctx.accounts.token_program.to_account_info(),
            rewarder:ctx.accounts.rewarder.to_account_info(),
            unused_clock:ctx.accounts.unused_clock.to_account_info(),
        }; 
        let cpi_ctx = CpiContext::new(quarry_program, cpi_accounts);
        quarry_mine::cpi::withdraw_tokens(cpi_ctx,amount);
       
     Ok(())
    }
 */
    /* pub fn claim(ctx:Context<ClaimUser>)->ProgramResult{
        msg!("1");
        let quarry_program=ctx.accounts.quarry_program.clone( );
        msg!("2");
        let stake=quarry_mine::cpi::accounts::UserClaim{
            authority:ctx.accounts.authority.to_account_info(),
            miner:ctx.accounts.miner.to_account_info(),
            quarry:ctx.accounts.quarry.to_account_info(),
            unused_miner_vault:ctx.accounts.unused_miner_vault.to_account_info(),
            unused_token_account:ctx.accounts.unused_token_account.to_account_info(),
            token_program:ctx.accounts.token_program.to_account_info(),
            rewarder:ctx.accounts.rewarder.to_account_info(),
            unused_clock:ctx.accounts.unused_clock.to_account_info(),
        };
        msg!("3");
        let cpi_accounts=quarry_mine::cpi::accounts::ClaimRewards{
            mint_wrapper:ctx.accounts.mint_wrapper.to_account_info(),
            mint_wrapper_program:ctx.accounts.mint_wrapper_program.to_account_info(),
            minter:ctx.accounts.minter.to_account_info(),
            rewards_token_mint:ctx.accounts.rewards_token_mint.to_account_info(),
            rewards_token_account:ctx.accounts.rewards_token_account.to_account_info(),
            claim_fee_token_account:ctx.accounts.claim_fee_token_account.to_account_info(),
            stake
        };
        msg!("4444");
        let cpi_ctx = CpiContext::new(quarry_program, cpi_accounts);
        msg!("5555");
        quarry_mine::cpi::claim_rewards(cpi_ctx);
        msg!("6");
        
     Ok(())
    } */
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct SwapInfo <'info>{
    pub market: MarketAccounts<'info>,
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub pc_wallet: AccountInfo<'info>,
    // Programs.
    pub dex_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    // Sysvars.
    pub rent: AccountInfo<'info>,
     //quarryProgram id 
     pub program_id_swap:AccountInfo<'info>,
    
}

#[derive(Accounts, Clone)]
pub struct MarketAccounts<'info> {
    #[account(mut)]
    pub market: AccountInfo<'info>,
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub request_queue: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    #[account(mut)]
    pub bids: AccountInfo<'info>,
    #[account(mut)]
    pub asks: AccountInfo<'info>,
    // For bids, this is the base currency. For asks, the quote.
    #[account(mut)]
    pub order_payer_token_account: AccountInfo<'info>,
    // Also known as the "base" currency. For a given A/B market,
    // this is the vault for the A mint.
    #[account(mut)]
    pub coin_vault: AccountInfo<'info>,
    // Also known as the "quote" currency. For a given A/B market,
    // this is the vault for the B mint.
    #[account(mut)]
    pub pc_vault: AccountInfo<'info>,
    // PDA owner of the DEX's token accounts for base + quote currencies.
    pub vault_signer: AccountInfo<'info>,
    // User wallets.
    #[account(mut)]
    pub coin_wallet: AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct AccountH <'info>{
    pub owner: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub program_address:AccountInfo<'info>,
    pub token_program1: Program<'info, Token>,
    pub account_program_address:AccountInfo<'info>,
    pub token_program2: Program<'info, Token>,
    pub program_principal:AccountInfo<'info>,
    pub token_program3: Program<'info, Token>,
    pub program_id:AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct AccountSeeds <'info>{
    pub program_address:AccountInfo<'info>,
    pub token_program1: Program<'info, Token>,
    pub account_program_address:AccountInfo<'info>,
    pub token_program2: Program<'info, Token>,
    pub program_principal:AccountInfo<'info>,
  
}
/* 
#[derive(Accounts)]
pub struct StakeUser<'info>{
    pub authority: Signer<'info>,
    /// Miner.
    #[account(mut)]
    pub miner: Account<'info, Miner>,
    /// Quarry to claim from.
    #[account(mut)]
    pub quarry: Account<'info, Quarry>,
    /// Vault of the miner.
    #[account(mut)]
    pub miner_vault: Account<'info, TokenAccount>,
    /// User's staked token account
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    /// Token program
    pub token_program: Program<'info, Token>,
    /// Rewarder
    pub rewarder: Account<'info, Rewarder>,
    /// Unused variable that held the clock. Placeholder.
    pub unused_clock: UncheckedAccount<'info>,
    //quarryProgram id 
    pub quarry_program:AccountInfo<'info>,
}
 */
/* 
#[derive(Accounts)]
pub struct ClaimUser<'info>{
    /// Mint wrapper.
    #[account(mut)]
    pub mint_wrapper: Box<Account<'info, quarry_mint_wrapper::MintWrapper>>,
    /// Mint wrapper program.
    pub mint_wrapper_program: Program<'info, quarry_mint_wrapper::program::QuarryMintWrapper>,
    /// [quarry_mint_wrapper::Minter] information.
    #[account(mut)]
    pub minter: Box<Account<'info, quarry_mint_wrapper::Minter>>,

    /// Mint of the rewards token.
   #[account(mut)]
   pub rewards_token_mint: Account<'info, Mint>, 

   /// Account to claim rewards for.
    #[account(mut)]
    pub rewards_token_account: Box<Account<'info, TokenAccount>>,

    /// Account to send claim fees to.
    #[account(mut)]
    pub claim_fee_token_account: Box<Account<'info, TokenAccount>>, 
    /// Miner authority (i.e. the user).
    pub authority: Signer<'info>,

    /// Miner.
    #[account(mut)]
    pub miner: Account<'info, Miner>,
     /// Quarry to claim from.
     #[account(mut)]
     pub quarry: Account<'info, Quarry>,
 
     /// Placeholder for the miner vault.
     #[account(mut)]
     pub unused_miner_vault: UncheckedAccount<'info>,
  /// Placeholder for the user's staked token account.
  #[account(mut)]
  pub unused_token_account: UncheckedAccount<'info>,

  /// Token program
  pub token_program: Program<'info, Token>,
     /// Rewarder
     pub rewarder: Account<'info, Rewarder>,

     /// Unused variable that held the clock. Placeholder.
     pub unused_clock: UncheckedAccount<'info>,

 //quarryProgram id 
 pub quarry_program:AccountInfo<'info>,

} */