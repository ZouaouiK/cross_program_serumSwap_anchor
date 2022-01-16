//! Program to perform instantly settled token swaps on the Serum DEX.
//!
//! Before using any instruction here, a user must first create an open orders
//! account on all markets being used. This only needs to be done once, either
//! via the system program create account instruction in the same transaction
//! as the user's first trade or via the explicit `init_account` and
//! `close_account` instructions provided here, which can be included in
//! transactions.

use anchor_lang::prelude::*;
use anchor_spl::dex;
use anchor_spl::dex::serum_dex::instruction::SelfTradeBehavior;
use anchor_spl::dex::serum_dex::matching::{OrderType, Side as SerumSide};
use anchor_spl::dex::serum_dex::state::MarketState;
use anchor_spl::token;
use solana_program::declare_id;
use std::num::NonZeroU64;

declare_id!("8TEaanmaLdMESU9rdjomaFfcbxkrp3EM1vbU4bWgzymz");

// Associated token account for Pubkey::default.
mod empty {
    use super::*;
    declare_id!("8TEaanmaLdMESU9rdjomaFfcbxkrp3EM1vbU4bWgzymz");
}

#[program]
pub mod serum_swap {
    use super::*;
    pub fn hello(ctx: Context<StructAccount>) -> ProgramResult {
        let owner =ctx.accounts.owner.key;
        msg!("karima test 2owner is {:?}",owner);
        Ok(())
    }
    /// Convenience API to initialize an open orders account on the Serum DEX.
    pub fn init_account<'info>(ctx: Context<'_, '_, '_, 'info, InitAccount<'info>>) -> Result<()> {
        let ctx = CpiContext::new(ctx.accounts.dex_program.clone(), ctx.accounts.into());
        dex::init_open_orders(ctx)?;
        Ok(())
    }

    /// Convenience API to close an open orders account on the Serum DEX.
    pub fn close_account<'info>(
        ctx: Context<'_, '_, '_, 'info, CloseAccount<'info>>,
    ) -> Result<()> {
        let ctx = CpiContext::new(ctx.accounts.dex_program.clone(), ctx.accounts.into());
        dex::close_open_orders(ctx)?;
        Ok(())
    }

    /// Swaps two tokens on a single A/B market, where A is the base currency
    /// and B is the quote currency. This is just a direct IOC trade that
    /// instantly settles.
    ///
    /// When side is "bid", then swaps B for A. When side is "ask", then swaps
    /// A for B.
    ///
    /// Arguments:
    ///
    /// * `side`              - The direction to swap.
    /// * `amount`            - The amount to swap *from*
    /// * `min_exchange_rate` - The exchange rate to use when determining
    ///    whether the transaction should abort.
    #[access_control(is_valid_swap(&ctx))]
    pub fn swap<'info>(
        ctx: Context<'_, '_, '_, 'info, Swap<'info>>,
        side: Side,
        amount: u64,
        min_exchange_rate: ExchangeRate,
    ) -> Result<()> {
        let mut min_exchange_rate = min_exchange_rate;
        msg!("swap 1");
        // Not used for direct swaps.
        min_exchange_rate.quote_decimals = 0;
        msg!("swap 2");
        // Optional referral account (earns a referral fee).
        let referral = ctx.remaining_accounts.iter().next().map(Clone::clone);
        msg!("swap 3");
        // Side determines swap direction.
        let (from_token, to_token) = match side {
            Side::Bid => (&ctx.accounts.pc_wallet, &ctx.accounts.market.coin_wallet),
            Side::Ask => (&ctx.accounts.market.coin_wallet, &ctx.accounts.pc_wallet),
        };
        msg!("swap 4 {:?}",from_token);
        // Token balances before the trade.
        let from_amount_before = token::accessor::amount(from_token)?;
        let to_amount_before = token::accessor::amount(to_token)?;
        msg!("from_amount_before  {:?} to_amount_before {:?} ",from_amount_before,to_amount_before);
        msg!("swap 5");
        // Execute trade.
        let orderbook: OrderbookClient<'info> = (&*ctx.accounts).into();
        msg!("swap 64");
        match side {
            Side::Bid => orderbook.buy(amount, None)?,
            Side::Ask => orderbook.sell(amount, None)?,
        };
        msg!("swap 6");
        orderbook.settle(referral)?;
        msg!("swap 7");
        // Token balances after the trade.
        let from_amount_after = token::accessor::amount(from_token)?;
        let to_amount_after = token::accessor::amount(to_token)?;
        msg!("from_amount_after  {:?} to_amount_after {:?} ",from_amount_after,to_amount_after);
        msg!("swap 8");
        //  Calculate the delta, i.e. the amount swapped.
        let from_amount = from_amount_before.checked_sub(from_amount_after).unwrap();
        msg!("swap 9 from_amount {}",from_amount);
        let to_amount = to_amount_after.checked_sub(to_amount_before).unwrap();
        msg!("swap 19 to_amount {}",to_amount);
        // Safety checks.
        apply_risk_checks(DidSwap {
            authority: *ctx.accounts.authority.key,
            given_amount: amount,
            min_exchange_rate,
            from_amount,
            to_amount,
            quote_amount: 0,
            spill_amount: 0,
            from_mint: token::accessor::mint(from_token)?,
            to_mint: token::accessor::mint(to_token)?,
            quote_mint: match side {
                Side::Bid => token::accessor::mint(from_token)?,
                Side::Ask => token::accessor::mint(to_token)?,
            },
        })?;
        msg!("swap 20");
        Ok(())
    }

    /// Swaps two base currencies across two different markets.
    ///
    /// That is, suppose there are two markets, A/USD(x) and B/USD(x).
    /// Then swaps token A for token B via
    ///
    /// * IOC (immediate or cancel) sell order on A/USD(x) market.
    /// * Settle open orders to get USD(x).
    /// * IOC buy order on B/USD(x) market to convert USD(x) to token B.
    /// * Settle open orders to get token B.
    ///
    /// Arguments:
    ///
    /// * `amount`            - The amount to swap *from*.
    /// * `min_exchange_rate` - The exchange rate to use when determining
    ///    whether the transaction should abort.
    #[access_control(is_valid_swap_transitive(&ctx))]
    pub fn swap_transitive<'info>(
        ctx: Context<'_, '_, '_, 'info, SwapTransitive<'info>>,
        amount: u64,
        min_exchange_rate: ExchangeRate,
    ) -> Result<()> {
        // Optional referral account (earns a referral fee).
        let referral = ctx.remaining_accounts.iter().next().map(Clone::clone);

        // Leg 1: Sell Token A for USD(x) (or whatever quote currency is used).
        let (from_amount, sell_proceeds) = {
            // Token balances before the trade.
            let base_before = token::accessor::amount(&ctx.accounts.from.coin_wallet)?;
            let quote_before = token::accessor::amount(&ctx.accounts.pc_wallet)?;

            // Execute the trade.
            let orderbook = ctx.accounts.orderbook_from();
            orderbook.sell(amount, None)?;
            orderbook.settle(referral.clone())?;

            // Token balances after the trade.
            let base_after = token::accessor::amount(&ctx.accounts.from.coin_wallet)?;
            let quote_after = token::accessor::amount(&ctx.accounts.pc_wallet)?;

            // Report the delta.
            (
                base_before.checked_sub(base_after).unwrap(),
                quote_after.checked_sub(quote_before).unwrap(),
            )
        };

        // Leg 2: Buy Token B with USD(x) (or whatever quote currency is used).
        let (to_amount, buy_proceeds) = {
            // Token balances before the trade.
            let base_before = token::accessor::amount(&ctx.accounts.to.coin_wallet)?;
            let quote_before = token::accessor::amount(&ctx.accounts.pc_wallet)?;

            // Execute the trade.
            let orderbook = ctx.accounts.orderbook_to();
            orderbook.buy(sell_proceeds, None)?;
            orderbook.settle(referral)?;

            // Token balances after the trade.
            let base_after = token::accessor::amount(&ctx.accounts.to.coin_wallet)?;
            let quote_after = token::accessor::amount(&ctx.accounts.pc_wallet)?;

            // Report the delta.
            (
                base_after.checked_sub(base_before).unwrap(),
                quote_before.checked_sub(quote_after).unwrap(),
            )
        };

        // The amount of surplus quote currency *not* fully consumed by the
        // second half of the swap.
        let spill_amount = sell_proceeds.checked_sub(buy_proceeds).unwrap();

        // Safety checks.
        apply_risk_checks(DidSwap {
            given_amount: amount,
            min_exchange_rate,
            from_amount,
            to_amount:1,
            quote_amount: sell_proceeds,
            spill_amount,
            from_mint: token::accessor::mint(&ctx.accounts.from.coin_wallet)?,
            to_mint: token::accessor::mint(&ctx.accounts.to.coin_wallet)?,
            quote_mint: token::accessor::mint(&ctx.accounts.pc_wallet)?,
            authority: *ctx.accounts.authority.key,
        })?;

        Ok(())
    }
}

// Asserts the swap event executed at an exchange rate acceptable to the client.
fn apply_risk_checks(event: DidSwap) -> Result<()> {
    // Emit the event for client consumption.
    emit!(event);
    msg!("apply_risk_checks event.to_amount {:?}",event.to_amount);
    if event.to_amount == 0 {
        return Err(ErrorCode::ZeroSwap.into());
    }
    msg!("apply_risk_checks 1");
    // Use the exchange rate to calculate the client's expectation.
    //
    // The exchange rate given must always have decimals equal to the
    // `to_mint` decimals, guaranteeing the `min_expected_amount`
    // always has decimals equal to
    //
    // `decimals(from_mint) + decimals(to_mint) + decimals(quote_mint)`.
    //
    // We avoid truncating by adding `decimals(quote_mint)`.
    msg!("apply_risk_checks 2");
    let min_expected_amount = u128::from(
        // decimals(from).
        event.from_amount,
    )
    .checked_mul(
        // decimals(from) + decimals(to).
        event.min_exchange_rate.rate.into(),
    )
    .unwrap()
    .checked_mul(
        // decimals(from) + decimals(to) + decimals(quote).
        10u128
            .checked_pow(event.min_exchange_rate.quote_decimals.into())
            .unwrap(),
    )
    .unwrap();
    msg!("apply_risk_checks 3");
    // If there is spill (i.e. quote tokens *not* fully consumed for
    // the buy side of a transitive swap), then credit those tokens marked
    // at the executed exchange rate to create an "effective" to_amount.
    msg!("apply_risk_checks 4");
    let effective_to_amount = {
        // Translates the leftover spill amount into "to" units via
        //
        // `(to_amount_received/quote_amount_given) * spill_amount`
        //
        let spill_surplus = match event.spill_amount == 0 || event.min_exchange_rate.strict {
            true => 0,
            false => u128::from(
                // decimals(to).
                event.to_amount,
            )
            .checked_mul(
                // decimals(to) + decimals(quote).
                event.spill_amount.into(),
            )
            .unwrap()
            .checked_mul(
                // decimals(to) + decimals(quote) + decimals(from).
                10u128
                    .checked_pow(event.min_exchange_rate.from_decimals.into())
                    .unwrap(),
            )
            .unwrap()
            .checked_mul(
                // decimals(to) + decimals(quote)*2 + decimals(from).
                10u128
                    .checked_pow(event.min_exchange_rate.quote_decimals.into())
                    .unwrap(),
            )
            .unwrap()
            .checked_div(
                // decimals(to) + decimals(quote) + decimals(from).
                event
                    .quote_amount
                    .checked_sub(event.spill_amount)
                    .unwrap()
                    .into(),
            )
            .unwrap(),
        };
        msg!("apply_risk_checks 5");
        // Translate the `to_amount` into a common number of decimals.
        let to_amount = u128::from(
            // decimals(to).
            event.to_amount,
        )
        .checked_mul(
            // decimals(to) + decimals(from).
            10u128
                .checked_pow(event.min_exchange_rate.from_decimals.into())
                .unwrap(),
        )
        .unwrap()
        .checked_mul(
            // decimals(to) + decimals(from) + decimals(quote).
            10u128
                .checked_pow(event.min_exchange_rate.quote_decimals.into())
                .unwrap(),
        )
        .unwrap();

        to_amount.checked_add(spill_surplus).unwrap()
    };

    // Abort if the resulting amount is less than the client's expectation.
    if effective_to_amount < min_expected_amount {
        msg!(
            "effective_to_amount, min_expected_amount: {:?}, {:?}",
            effective_to_amount,
            min_expected_amount,
        );
        return Err(ErrorCode::SlippageExceeded.into());
    }
    msg!("apply_risk_checks 5");
    Ok(())
}

#[derive(Accounts)]
pub struct InitAccount<'info> {
    #[account(mut)]
    open_orders: AccountInfo<'info>,
    #[account(signer)]
    authority: AccountInfo<'info>,
    market: AccountInfo<'info>,
    dex_program: AccountInfo<'info>,
    rent: AccountInfo<'info>,
}

impl<'info> From<&mut InitAccount<'info>> for dex::InitOpenOrders<'info> {
    fn from(accs: &mut InitAccount<'info>) -> dex::InitOpenOrders<'info> {
        dex::InitOpenOrders {
            open_orders: accs.open_orders.clone(),
            authority: accs.authority.clone(),
            market: accs.market.clone(),
            rent: accs.rent.clone(),
        }
    }
}
#[derive(Accounts)]
pub struct StructAccount<'info>{
 pub owner:AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct CloseAccount<'info> {
    #[account(mut)]
    open_orders: AccountInfo<'info>,
    #[account(signer)]
    authority: AccountInfo<'info>,
    #[account(mut)]
    destination: AccountInfo<'info>,
    market: AccountInfo<'info>,
    dex_program: AccountInfo<'info>,
}

impl<'info> From<&mut CloseAccount<'info>> for dex::CloseOpenOrders<'info> {
    fn from(accs: &mut CloseAccount<'info>) -> dex::CloseOpenOrders<'info> {
        dex::CloseOpenOrders {
            open_orders: accs.open_orders.clone(),
            authority: accs.authority.clone(),
            destination: accs.destination.clone(),
            market: accs.market.clone(),
        }
    }
}

// The only constraint imposed on these accounts is that the market's base
// currency mint is not equal to the quote currency's. All other checks are
// done by the DEX on CPI.
#[derive(Accounts)]
pub struct Swap<'info> {
    pub market: MarketAccounts<'info>,
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    #[account(mut, constraint = pc_wallet.key != &empty::ID)]
    pub pc_wallet: AccountInfo<'info>,
    // Programs.
    pub dex_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    // Sysvars.
    pub rent: AccountInfo<'info>,
}

impl<'info> From<&Swap<'info>> for OrderbookClient<'info> {
    fn from(accounts: &Swap<'info>) -> OrderbookClient<'info> {
        OrderbookClient {
            market: accounts.market.clone(),
            authority: accounts.authority.clone(),
            pc_wallet: accounts.pc_wallet.clone(),
            dex_program: accounts.dex_program.clone(),
            token_program: accounts.token_program.clone(),
            rent: accounts.rent.clone(),
        }
    }
}

// The only constraint imposed on these accounts is that the from market's
// base currency's is not equal to the to market's base currency. All other
// checks are done by the DEX on CPI (and the quote currency is ensured to be
// the same on both markets since there's only one account field for it).
#[derive(Accounts)]
pub struct SwapTransitive<'info> {
    pub from: MarketAccounts<'info>,
    pub to: MarketAccounts<'info>,
    // Must be the authority over all open orders accounts used.
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    #[account(mut, constraint = pc_wallet.key != &empty::ID)]
    pub pc_wallet: AccountInfo<'info>,
    // Programs.
    pub dex_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    // Sysvars.
    pub rent: AccountInfo<'info>,
}

impl<'info> SwapTransitive<'info> {
    fn orderbook_from(&self) -> OrderbookClient<'info> {
        OrderbookClient {
            market: self.from.clone(),
            authority: self.authority.clone(),
            pc_wallet: self.pc_wallet.clone(),
            dex_program: self.dex_program.clone(),
            token_program: self.token_program.clone(),
            rent: self.rent.clone(),
        }
    }
    fn orderbook_to(&self) -> OrderbookClient<'info> {
        OrderbookClient {
            market: self.to.clone(),
            authority: self.authority.clone(),
            pc_wallet: self.pc_wallet.clone(),
            dex_program: self.dex_program.clone(),
            token_program: self.token_program.clone(),
            rent: self.rent.clone(),
        }
    }
}

// Client for sending orders to the Serum DEX.
#[derive(Clone)]
struct OrderbookClient<'info> {
    market: MarketAccounts<'info>,
    authority: AccountInfo<'info>,
    pc_wallet: AccountInfo<'info>,
    dex_program: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    rent: AccountInfo<'info>,
}

impl<'info> OrderbookClient<'info> {
    // Executes the sell order portion of the swap, purchasing as much of the
    // quote currency as possible for the given `base_amount`.
    //
    // `base_amount` is the "native" amount of the base currency, i.e., token
    // amount including decimals.
    fn sell(
        &self,
        base_amount: u64,
        srm_msrm_discount: Option<AccountInfo<'info>>,
    ) -> ProgramResult {
        let limit_price = 1;
        let max_coin_qty = {
            // The loaded market must be dropped before CPI.
            let market = MarketState::load(&self.market.market, &dex::ID)?;
            coin_lots(&market, base_amount)
        };
        let max_native_pc_qty = u64::MAX;
        self.order_cpi(
            limit_price,
            max_coin_qty,
            max_native_pc_qty,
            Side::Ask,
            srm_msrm_discount,
        )
    }

    // Executes the buy order portion of the swap, purchasing as much of the
    // base currency as possible, for the given `quote_amount`.
    //
    // `quote_amount` is the "native" amount of the quote currency, i.e., token
    // amount including decimals.
    fn buy(
        &self,
        quote_amount: u64,
        srm_msrm_discount: Option<AccountInfo<'info>>,
    ) -> ProgramResult {
        let limit_price = u64::MAX;
        let max_coin_qty = u64::MAX;
        let max_native_pc_qty = quote_amount;
        self.order_cpi(
            limit_price,
            max_coin_qty,
            max_native_pc_qty,
            Side::Bid,
            srm_msrm_discount,
        )
    }

    // Executes a new order on the serum dex via CPI.
    //
    // * `limit_price` - the limit order price in lot units.
    // * `max_coin_qty`- the max number of the base currency lot units.
    // * `max_native_pc_qty` - the max number of quote currency in native token
    //                         units (includes decimals).
    // * `side` - bid or ask, i.e. the type of order.
    // * `referral` - referral account, earning a fee.
    fn order_cpi(
        &self,
        limit_price: u64,
        max_coin_qty: u64,
        max_native_pc_qty: u64,
        side: Side,
        srm_msrm_discount: Option<AccountInfo<'info>>,
    ) -> ProgramResult {
        // Client order id is only used for cancels. Not used here so hardcode.
        let client_order_id = 0;
        // Limit is the dex's custom compute budge parameter, setting an upper
        // bound on the number of matching cycles the program can perform
        // before giving up and posting the remaining unmatched order.
        let limit = 65535;

        let mut ctx = CpiContext::new(self.dex_program.clone(), self.clone().into());
        if let Some(srm_msrm_discount) = srm_msrm_discount {
            ctx = ctx.with_remaining_accounts(vec![srm_msrm_discount]);
        }
        dex::new_order_v3(
            ctx,
            side.into(),
            NonZeroU64::new(limit_price).unwrap(),
            NonZeroU64::new(max_coin_qty).unwrap(),
            NonZeroU64::new(max_native_pc_qty).unwrap(),
            SelfTradeBehavior::DecrementTake,
            OrderType::ImmediateOrCancel,
            client_order_id,
            limit,
        )
    }

    fn settle(&self, referral: Option<AccountInfo<'info>>) -> ProgramResult {
        let settle_accs = dex::SettleFunds {
            market: self.market.market.clone(),
            open_orders: self.market.open_orders.clone(),
            open_orders_authority: self.authority.clone(),
            coin_vault: self.market.coin_vault.clone(),
            pc_vault: self.market.pc_vault.clone(),
            coin_wallet: self.market.coin_wallet.clone(),
            pc_wallet: self.pc_wallet.clone(),
            vault_signer: self.market.vault_signer.clone(),
            token_program: self.token_program.clone(),
        };
        let mut ctx = CpiContext::new(self.dex_program.clone(), settle_accs);
        if let Some(referral) = referral {
            ctx = ctx.with_remaining_accounts(vec![referral]);
        }
        dex::settle_funds(ctx)
    }
}

impl<'info> From<OrderbookClient<'info>> for dex::NewOrderV3<'info> {
    fn from(c: OrderbookClient<'info>) -> dex::NewOrderV3<'info> {
        dex::NewOrderV3 {
            market: c.market.market.clone(),
            open_orders: c.market.open_orders.clone(),
            request_queue: c.market.request_queue.clone(),
            event_queue: c.market.event_queue.clone(),
            market_bids: c.market.bids.clone(),
            market_asks: c.market.asks.clone(),
            order_payer_token_account: c.market.order_payer_token_account.clone(),
            open_orders_authority: c.authority.clone(),
            coin_vault: c.market.coin_vault.clone(),
            pc_vault: c.market.pc_vault.clone(),
            token_program: c.token_program.clone(),
            rent: c.rent.clone(),
        }
    }
}

// Returns the amount of lots for the base currency of a trade with `size`.
fn coin_lots(market: &MarketState, size: u64) -> u64 {
    size.checked_div(market.coin_lot_size).unwrap()
}

// Market accounts are the accounts used to place orders against the dex minus
// common accounts, i.e., program ids, sysvars, and the `pc_wallet`.
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
    // The `spl_token::Account` that funds will be taken from, i.e., transferred
    // from the user into the market's vault.
    //
    // For bids, this is the base currency. For asks, the quote.
    #[account(mut, constraint = order_payer_token_account.key != &empty::ID)]
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
    #[account(mut, constraint = coin_wallet.key != &empty::ID)]
    pub coin_wallet: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum Side {
    Bid,
    Ask,
}

impl From<Side> for SerumSide {
    fn from(side: Side) -> SerumSide {
        match side {
            Side::Bid => SerumSide::Bid,
            Side::Ask => SerumSide::Ask,
        }
    }
}

// Access control modifiers.

fn is_valid_swap(ctx: &Context<Swap>) -> Result<()> {
    _is_valid_swap(&ctx.accounts.market.coin_wallet, &ctx.accounts.pc_wallet)
}

fn is_valid_swap_transitive(ctx: &Context<SwapTransitive>) -> Result<()> {
    _is_valid_swap(&ctx.accounts.from.coin_wallet, &ctx.accounts.to.coin_wallet)
}

// Validates the tokens being swapped are of different mints.
fn _is_valid_swap<'info>(from: &AccountInfo<'info>, to: &AccountInfo<'info>) -> Result<()> {
    let from_token_mint = token::accessor::mint(from)?;
    let to_token_mint = token::accessor::mint(to)?;
    if from_token_mint == to_token_mint {
        return Err(ErrorCode::SwapTokensCannotMatch.into());
    }
    Ok(())
}

// Event emitted when a swap occurs for two base currencies on two different
// markets (quoted in the same token).
#[event]
pub struct DidSwap {
    // User given (max) amount  of the "from" token to swap.
    pub given_amount: u64,
    // The minimum exchange rate for swapping `from_amount` to `to_amount` in
    // native units with decimals equal to the `to_amount`'s mint--specified
    // by the client.
    pub min_exchange_rate: ExchangeRate,
    // Amount of the `from` token sold.
    pub from_amount: u64,
    // Amount of the `to` token purchased.
    pub to_amount: u64,
    // The amount of the quote currency used for a *transitive* swap. This is
    // the amount *received* for selling on the first leg of the swap.
    pub quote_amount: u64,
    // Amount of the quote currency accumulated from a *transitive* swap, i.e.,
    // the difference between the amount gained from the first leg of the swap
    // (to sell) and the amount used in the second leg of the swap (to buy).
    pub spill_amount: u64,
    // Mint sold.
    pub from_mint: Pubkey,
    // Mint purchased.
    pub to_mint: Pubkey,
    // Mint of the token used as the quote currency in the two markets used
    // for swapping.
    pub quote_mint: Pubkey,
    // User that signed the transaction.
    pub authority: Pubkey,
}

// An exchange rate for swapping *from* one token *to* another.
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ExchangeRate {
    // The amount of *to* tokens one should receive for a single *from token.
    // This number must be in native *to* units with the same amount of decimals
    // as the *to* mint.
    pub rate: u64,
    // Number of decimals of the *from* token's mint.
    pub from_decimals: u8,
    // Number of decimals of the *to* token's mint.
    // For a direct swap, this should be zero.
    pub quote_decimals: u8,
    // True if *all* of the *from* currency sold should be used when calculating
    // the executed exchange rate.
    //
    // To perform a transitive swap, one sells on one market and buys on
    // another, where both markets are quoted in the same currency. Now suppose
    // one swaps A for B across A/USDC and B/USDC. Further suppose the first
    // leg swaps the entire *from* amount A for USDC, and then only half of
    // the USDC is used to swap for B on the second leg. How should we calculate
    // the exchange rate?
    //
    // If strict is true, then the exchange rate will be calculated as a direct
    // function of the A tokens lost and B tokens gained, ignoring the surplus
    // in USDC received. If strict is false, an effective exchange rate will be
    // used. I.e. the surplus in USDC will be marked at the exchange rate from
    // the second leg of the swap and that amount will be added to the
    // *to* mint received before calculating the swap's exchange rate.
    //
    // Transitive swaps only. For direct swaps, this field is ignored.
    pub strict: bool,
}

#[error]
pub enum ErrorCode {
    #[msg("The tokens being swapped must have different mints")]
    SwapTokensCannotMatch,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("No tokens received when swapping")]
    ZeroSwap,
}
