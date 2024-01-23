
use anchor_lang::{prelude::*, solana_program::program::invoke_signed};
use anchor_spl::{token::{TokenAccount, Token}, associated_token::AssociatedToken};
use arrayref::array_ref;


declare_id!("8NKtq4YKTd5oeJxBX9rcZAwabBong4Cb9G1muHc86SGt");

const PREFIX: &str = "pfp";

#[program]
pub mod pfp {

    use anchor_lang::solana_program::{entrypoint::ProgramResult, program::invoke_signed};

    use super::*;

    pub fn initialize(ctx: Context<Initialize>,_initial_fee: u64,) -> ProgramResult {
        let base_account = &mut ctx.accounts.base_account;
        let ata = &ctx.accounts.ata;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;
        let ata_program = &ctx.accounts.ata_program;
        let rent = &ctx.accounts.rent;
        
        if ata.data_is_empty() {
            make_ata(
                ata.to_account_info(),
                ctx.accounts.fee_receiver.to_account_info(),
                ctx.accounts.fee_token.to_account_info(),
                ctx.accounts.initializer.to_account_info(),
                ata_program.to_account_info(),
                token_program.to_account_info(),
                system_program.to_account_info(),
                rent.to_account_info(),
                &[],
            )?;
        }

        base_account.owner = *ctx.accounts.initializer.key;
        base_account.pimping_fee = _initial_fee;
        base_account.fee_receiver = *ata.key;
        base_account.fee_token = *ctx.accounts.fee_token.to_account_info().key;

        let ba_key = base_account.key();

        emit!(BaseAccountInfo {
            base_account: ba_key,
            pimping_fee: base_account.pimping_fee,
            fee_token: base_account.fee_token,
            fee_receiver: base_account.fee_receiver
        });
        Ok(())
    }

    pub fn set_fee_token(ctx: Context<SetFeeToken>) -> ProgramResult {
        let base_account = &mut ctx.accounts.base_account;
        let fee_token_mint = base_account.fee_token;
        let fee_payer = &ctx.accounts.payer;
        let new_fee_token_account = &ctx.accounts.new_fee_token_account;
        let new_fee_reciever_token_account = &ctx.accounts.new_fee_reciever_token_account;
        let new_fee_reciver = &ctx.accounts.new_receiver;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;
        let ata_program = &ctx.accounts.ata_program;
        let rent = &ctx.accounts.rent;
        assert_eq!(fee_payer.key(), base_account.owner, "Not authorized");
        assert_eq!(fee_token_mint, ctx.accounts.fee_token.key(), "invalid fee token");
        if new_fee_reciever_token_account.data_is_empty() {
            make_ata(
                new_fee_reciever_token_account.to_account_info(),
                new_fee_reciver.to_account_info(),
                new_fee_token_account.to_account_info(),
                fee_payer.to_account_info(),
                ata_program.to_account_info(),
                token_program.to_account_info(),
                system_program.to_account_info(),
                rent.to_account_info(),
                &[],
            )?;
        }

        base_account.fee_receiver = new_fee_reciever_token_account.key();
        base_account.fee_token = new_fee_token_account.key();
        Ok(())
    }

    pub fn set_fee_receiver(ctx: Context<SetFeeReceiver>) -> ProgramResult {
        let base_account = &mut ctx.accounts.base_account;
        let fee_token_mint = base_account.fee_token;
        let fee_payer = &ctx.accounts.payer;
        let new_fee_reciever_token_account = &ctx.accounts.new_fee_reciever_token_account;
        let new_fee_reciver = &ctx.accounts.new_receiver;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;
        let ata_program = &ctx.accounts.ata_program;
        let rent = &ctx.accounts.rent;
        assert_eq!(fee_payer.key(), base_account.owner, "Not authorized");
        assert_eq!(fee_token_mint, ctx.accounts.fee_token.key(), "invalid fee token");
        if new_fee_reciever_token_account.data_is_empty() {
            make_ata(
                new_fee_reciever_token_account.to_account_info(),
                new_fee_reciver.to_account_info(),
                ctx.accounts.fee_token.to_account_info(),
                fee_payer.to_account_info(),
                ata_program.to_account_info(),
                token_program.to_account_info(),
                system_program.to_account_info(),
                rent.to_account_info(),
                &[],
            )?;
        }

        base_account.fee_receiver = new_fee_reciever_token_account.key();
        Ok(())
    }

    pub fn set_pimping_fee(ctx: Context<SetPimpimgFee>, new_set_pimping_fee: u64) -> ProgramResult {
        let base_account = &mut ctx.accounts.base_account;
        let owner_key = *ctx.accounts.payer.key;
        assert_eq!(owner_key, base_account.owner, "Not authorized");
        base_account.pimping_fee = new_set_pimping_fee;
        Ok(())
    }
    
    pub fn pimp_my_pfp(ctx: Context<PimpMyPfp>) -> ProgramResult {

        let base_account = ctx.accounts.base_account.clone();
        let token_id = &ctx.accounts.token_account;
        let token_account_clone = token_id.to_account_info();
        let nft_token_account = &ctx.accounts.nft_token_account;
        let token_mint = get_mint_from_token_account(&nft_token_account)?;
        let payer_key = &ctx.accounts.payer;
        let fee_reciever_token_account = &ctx.accounts.fee_reciever_token_account;
        let token_program = &ctx.accounts.token_program;
        let fee_payer_token_account = &ctx.accounts.fee_payer_token_account;

        assert_eq!(fee_reciever_token_account.key(), base_account.fee_receiver, "Not valid payment reciever");
        invoke_signed(
            &spl_token::instruction::transfer(
                token_program.key,
                &fee_payer_token_account.key(),
                &fee_reciever_token_account.key(),
                &payer_key.key(),
                &[],
                base_account.pimping_fee,
            )?,
            &[
                token_account_clone,
                fee_payer_token_account.to_account_info(),
                fee_reciever_token_account.to_account_info(),
                payer_key.to_account_info(),
            ],
            &[]
        )?;

        emit!(OrderPlaced {
            user: payer_key.key(),
            nft_address: token_mint, 
            is_nft: true
        });

        Ok(())
    }
    
    pub fn pimp_my_jpeg(ctx: Context<PimpMyJpeg>) -> ProgramResult {
        let base_account = ctx.accounts.base_account.clone();
        let token_id = &ctx.accounts.token_account;
        let token_account_clone = token_id.to_account_info();
        let payer_key = &ctx.accounts.payer;
        let fee_reciever_token_account = &ctx.accounts.fee_reciever_token_account;
        let token_program = &ctx.accounts.token_program;
        let fee_payer_token_account = &ctx.accounts.fee_payer_token_account;

        assert_eq!(fee_reciever_token_account.key(), base_account.fee_receiver, "Not valid payment reciever");
        invoke_signed(
            &spl_token::instruction::transfer(
                token_program.key,
                &fee_payer_token_account.key(),
                &fee_reciever_token_account.key(),
                &payer_key.key(),
                &[],
                base_account.pimping_fee,
            )?,
            &[
                token_account_clone,
                fee_payer_token_account.to_account_info(),
                fee_reciever_token_account.to_account_info(),
                payer_key.to_account_info(),
            ],
            &[]
        )?;

        emit!(JpegOrderPlaced {
            user: payer_key.key(),
            is_nft: false
        });
        Ok(())
    }

    pub fn get_fee_token(ctx: Context<GetFeeToken>) -> ProgramResult {
        let base_account = ctx.accounts.base_account.clone();
        emit!(BaseAccountInfo {
            base_account: *ctx.accounts.base_account.to_account_info().key,
            pimping_fee: base_account.pimping_fee,
            fee_token: base_account.fee_token,
            fee_receiver: base_account.fee_receiver,
        });
        Ok(())
    }
    
    pub fn withdraw(ctx: Context<Withdraw>) -> ProgramResult {
        let base_account = &mut ctx.accounts.base_account;
        let owner_key = *ctx.accounts.owner.key;
        assert_eq!(owner_key, *base_account.owner, "Not authorized");
        let base_lamports = **base_account.to_account_info().lamports.borrow();
        
        **base_account.to_account_info().lamports.borrow_mut() = 0;
    
        **ctx.accounts.owner.to_account_info().lamports.borrow_mut() = 
          ctx
            .accounts
            .owner
            .to_account_info()
            .lamports()
            .checked_add(base_lamports)
            .ok_or(ErrorCode::Overflow).unwrap();
    
        Ok(())
      }
}

pub fn get_mint_from_token_account(
    token_account_info: &AccountInfo,
) -> Result<Pubkey> {
    // TokeAccount layout:   mint(32), owner(32), ...
    let data = token_account_info.try_borrow_data()?;
    let mint_data = array_ref![data, 0, 32];
    Ok(Pubkey::new_from_array(*mint_data))
}


pub fn make_ata<'a>(
    ata: AccountInfo<'a>,
    wallet: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    fee_payer: AccountInfo<'a>,
    ata_program: AccountInfo<'a>,
    token_program: AccountInfo<'a>,
    system_program: AccountInfo<'a>,
    rent: AccountInfo<'a>,
    fee_payer_seeds: &[&[u8]],
) -> Result<()> {
    let as_arr = [fee_payer_seeds];

    let seeds: &[&[&[u8]]] = if !fee_payer_seeds.is_empty() {
        &as_arr
    } else {
        &[]
    };

    invoke_signed(
        &spl_associated_token_account::instruction::create_associated_token_account(
            fee_payer.key,
            wallet.key,
            mint.key,
            &spl_token::ID,
        ),
        &[
            ata,
            wallet,
            mint,
            fee_payer,
            ata_program,
            system_program,
            rent,
            token_program,
        ],
        seeds,
    )?;

    Ok(())

    
}




#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(init, payer = initializer, space = 8 + 32 + 32 + 32 + 32, seeds=[PREFIX.as_bytes(), initializer.key().as_ref()], bump)]
    pub base_account: Account<'info, BaseAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub fee_receiver: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub fee_token: AccountInfo<'info>,
    #[account(mut)] 
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub ata: UncheckedAccount<'info>,
    #[account(executable)]
    pub token_program: Program<'info, Token>,
    pub ata_program: Program<'info, AssociatedToken>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct BaseAccount {
    pub owner: Pubkey,
    pub pimping_fee: u64,
    pub fee_receiver: Pubkey, 
    pub fee_token: Pubkey,
    pub bump: u8,
}

#[derive(Accounts)]    
pub struct SetFeeToken<'info> {
    #[account(mut)]
    pub base_account: Account<'info, BaseAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub new_receiver: AccountInfo<'info>,
    pub payer: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub fee_token: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub new_fee_reciever_token_account: UncheckedAccount<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub new_fee_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    #[account(executable)]
    pub token_program: Program<'info, Token>,
    pub ata_program: Program<'info, AssociatedToken>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]    
pub struct SetPimpimgFee<'info> {
    #[account(mut)]
    pub base_account: Account<'info, BaseAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub new_pimping_fee: AccountInfo<'info>,
    pub payer: Signer<'info>,
}

#[derive(Accounts)]   
pub struct SetFeeReceiver<'info> {
    #[account(mut)]
    pub base_account: Account<'info, BaseAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub new_receiver: AccountInfo<'info>,
    pub payer: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub fee_token: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub new_fee_reciever_token_account: UncheckedAccount<'info>,
    #[account(executable)]
    pub token_program: Program<'info, Token>,
    pub ata_program: Program<'info, AssociatedToken>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}


#[derive(Accounts)]
pub struct PimpMyPfp<'info> {
    #[account(mut)]
    pub base_account: Account<'info, BaseAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_token_account: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub fee_reciever_token_account: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    fee_payer_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    #[account(executable)]
    token_program: Program<'info, Token>,
    ata_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct PimpMyJpeg<'info> {
    #[account(mut)]
    pub base_account: Account<'info, BaseAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub fee_reciever_token_account: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    fee_payer_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    #[account(executable)]
    token_program: Program<'info, Token>,
    ata_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub base_account: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub owner: AccountInfo<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
    InsufficientFunds,
    Overflow,
}

#[event]
pub struct OrderPlaced {
  pub user: Pubkey,
  pub nft_address: Pubkey, 
  pub is_nft: bool
}

#[event]
pub struct JpegOrderPlaced {
  pub user: Pubkey,
  pub is_nft: bool
}


#[event]
pub struct BaseAccountInfo {
  pub base_account: Pubkey,
  pub pimping_fee: u64, 
  pub fee_token: Pubkey,
  pub fee_receiver: Pubkey
}

#[derive(Accounts)]
pub struct GetFeeToken<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub base_account: Account<'info, BaseAccount>,
}

