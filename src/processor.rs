use crate::{instruction::InstructionEnum, state::Class};
use mpl_token_metadata::{
    self,
    state::{Collection, Creator, PREFIX},
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    native_token::LAMPORTS_PER_SOL,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction, system_program,
    sysvar::Sysvar, msg,
};
use spl_associated_token_account::{get_associated_token_address, *};
use spl_token::instruction::AuthorityType;

pub fn process_intruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {

    msg!("Hello World");
    match InstructionEnum::decode(data) {
        InstructionEnum::MintNft(class) => mint_nft(program_id, accounts, class)?,
        InstructionEnum::MintNewCollection => mint_collection(program_id, accounts)?,
        _ => Err(ProgramError::InvalidInstructionData)?,
    }

    Ok(())
}

pub fn mint_nft(program_id: &Pubkey, accounts: &[AccountInfo], class: Class) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let payer_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let mint_authority_account_info = next_account_info(account_info_iter)?;
    let associated_token_account_info = next_account_info(account_info_iter)?;
    let spl_token_program_account_info = next_account_info(account_info_iter)?;
    let sysvar_rent_accoount_info = next_account_info(account_info_iter)?;
    let system_program_account_info = next_account_info(account_info_iter)?;
    let meta_data_account_info = next_account_info(account_info_iter)?;
    let minting_pool_account_info = next_account_info(account_info_iter)?;

    let space = 82;
    let rent_lamports = Rent::get()?.minimum_balance(space);

    let (minting_pool_id, _minting_pool_bump) =
        Pubkey::find_program_address(&[b"minting_pool"], program_id);

    if minting_pool_id != *minting_pool_account_info.key {
        Err(ProgramError::Custom(0))?
    }

    let (mint_authority_key, mint_authority_bump) =
        Pubkey::find_program_address(&[b"mint_authority"], program_id);

    if mint_authority_key != *mint_authority_account_info.key {
        Err(ProgramError::Custom(1))?
    }

    if get_associated_token_address(payer_account_info.key, mint_account_info.key)
        != *associated_token_account_info.key
    {
        Err(ProgramError::Custom(2))?
    }

    if system_program::check_id(system_program_account_info.key) {
        Err(ProgramError::Custom(3))?
    }

    if spl_token::id() != *spl_token_program_account_info.key {
        Err(ProgramError::Custom(4))?
    }
    
    let mpl_token_metadata_id = mpl_token_metadata::id();
    let metadata_seeds = &[
        PREFIX.as_bytes(),
        mpl_token_metadata_id.as_ref(),
        mint_account_info.key.as_ref(),
    ];

    let (nft_metadata_key, _nft_metadata_bump) =
        Pubkey::find_program_address(metadata_seeds, program_id);

    if nft_metadata_key != *meta_data_account_info.key {
        Err(ProgramError::Custom(5))?
    }

    if mpl_token_metadata::id() != *meta_data_account_info.key {
        Err(ProgramError::Custom(6))?
    }

    let mint_cost = LAMPORTS_PER_SOL
        * match class {
            Class::Ruby => 5,
            Class::Diamond => 2,
            Class::Sapphire => 1,
        };
    //tranfer token from one account to an other
    invoke(
        &system_instruction::transfer(payer_account_info.key, &minting_pool_id, mint_cost),
        &[
            payer_account_info.clone(),
            minting_pool_account_info.clone(),
        ],
    )?;

    //create the mint account
    invoke(
        &system_instruction::create_account(
            payer_account_info.key,
            mint_account_info.key,
            rent_lamports,
            space as u64,
            spl_token_program_account_info.key,
        ),
        &[payer_account_info.clone(), mint_account_info.clone()],
    )?;

    invoke(
        &spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint_account_info.key,
            &mint_authority_key,
            Some(&mint_authority_key),
            0,
        )?,
        &[mint_account_info.clone(), sysvar_rent_accoount_info.clone()],
    )?;

    invoke(
        &spl_associated_token_account::instruction::create_associated_token_account(
            payer_account_info.key,
            payer_account_info.key,
            mint_account_info.key,
        ),
        &[
            payer_account_info.clone(),
            associated_token_account_info.clone(),
            payer_account_info.clone(),
            mint_account_info.clone(),
            system_program_account_info.clone(),
            spl_token_program_account_info.clone(),
        ],
    )?;

    invoke_signed(
        &spl_token::instruction::mint_to(
            spl_token_program_account_info.key,
            mint_account_info.key,
            associated_token_account_info.key,
            &mint_authority_key,
            &[],
            1,
        )?,
        &[
            mint_account_info.clone(),
            associated_token_account_info.clone(),
            mint_authority_account_info.clone(),
        ],
        &[&[b"mint_authority", &[mint_authority_bump]]],
    )?;

    let mut creators = Vec::new();
    creators.push(Creator {
        address: mint_authority_key,
        verified: true,
        share: 0,
    });

    let (ingl_nft_collection_key, _ingl_nft_bump) =
        Pubkey::find_program_address(&[b"ingl_nft_collection"], program_id);

    invoke(
        &mpl_token_metadata::instruction::create_metadata_accounts_v3(
            mpl_token_metadata_id,
            nft_metadata_key,
            *mint_account_info.key,
            *mint_authority_account_info.key,
            *payer_account_info.key,
            *mint_authority_account_info.key,
            String::from("The fox #1"),
            String::from("FX#1"),
            String::from("https://arweave.net/WimminaDHDBBxHby2dzbXTMomy2UYTLDf4ymCCQLtP4"),
            Some(creators),
            300,
            true,
            true,
            Some(Collection {
                key: ingl_nft_collection_key,
                verified: true,
            }),
            None,
            None,
        ),
        &[
            meta_data_account_info.clone(),
            mint_account_info.clone(),
            mint_authority_account_info.clone(),
            payer_account_info.clone(),
            mint_authority_account_info.clone(),
            system_program_account_info.clone(),
            sysvar_rent_accoount_info.clone(),
        ],
    )?;

    invoke_signed(
        &spl_token::instruction::set_authority(
            spl_token_program_account_info.key,
            mint_account_info.key,
            None,
            AuthorityType::MintTokens,
            &mint_authority_key,
            &[],
        )?,
        &[
            mint_account_info.clone(),
            mint_authority_account_info.clone(),
        ],
        &[&[b"mint_authority", &[mint_authority_bump]]],
    )?;

    invoke(
        &mpl_token_metadata::instruction::update_primary_sale_happened_via_token(
            mpl_token_metadata::id(),
            nft_metadata_key,
            spl_token::id(),
            *associated_token_account_info.key,
        ),
        &[
            meta_data_account_info.clone(),
            spl_token_program_account_info.clone(),
            associated_token_account_info.clone(),
        ],
    )?;

    Ok(())
}

pub fn mint_collection(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let payer_account_info = next_account_info(account_info_iter)?;
    let mint_account_info = next_account_info(account_info_iter)?;
    let mint_authority_account_info = next_account_info(account_info_iter)?;
    let associated_token_account_info = next_account_info(account_info_iter)?;
    let spl_token_program_account_info = next_account_info(account_info_iter)?;
    let sysvar_rent_accoount_info = next_account_info(account_info_iter)?;
    let system_program_account_info = next_account_info(account_info_iter)?;
    let meta_data_account_info = next_account_info(account_info_iter)?;

    let (ingl_nft_collection_key, _ingl_nft_bump) =
        Pubkey::find_program_address(&[b"ingl_nft_collection1"], program_id);

    if ingl_nft_collection_key != *mint_account_info.key {
        msg!("Mint account info don't match");
        Err(ProgramError::Custom(0))?
    }

    let space = 82;
    let rent_lamports = Rent::get()?.minimum_balance(space);

    msg!("Create mint account");
    invoke_signed(
        &system_instruction::create_account(
            payer_account_info.key,
            mint_account_info.key,
            rent_lamports,
            space as u64,
            spl_token_program_account_info.key,
        ),
        &[payer_account_info.clone(), mint_account_info.clone()],
        &[&[b"ingl_nft_collection1", &[_ingl_nft_bump]]],
    )?;

    let (mint_authority_key, _mint_authority_bump) =
        Pubkey::find_program_address(&[b"mint_authority"], program_id);

    if mint_authority_key != *mint_authority_account_info.key {
        Err(ProgramError::Custom(1))?
    }

    msg!("Initialize mint account");
    invoke(
        &spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint_account_info.key,
            &mint_authority_key,
            Some(&mint_authority_key),
            0,
        )?,
        &[mint_account_info.clone(), sysvar_rent_accoount_info.clone()],
    )?;

    msg!("Create associated token account");
    invoke(
        &spl_associated_token_account::instruction::create_associated_token_account(
            payer_account_info.key,
            payer_account_info.key,
            mint_account_info.key,
        ),
        &[
            payer_account_info.clone(),
            associated_token_account_info.clone(),
            payer_account_info.clone(),
            mint_account_info.clone(),
            system_program_account_info.clone(),
            spl_token_program_account_info.clone(),
        ],
    )?;

    let (mint_authority_key, mint_authority_bump) =
        Pubkey::find_program_address(&[b"mint_authority"], program_id);

    if mint_authority_key != *mint_authority_account_info.key {
        Err(ProgramError::Custom(1))?
    }

    msg!("Mint new collection token");
    invoke_signed(
        &spl_token::instruction::mint_to(
            spl_token_program_account_info.key,
            mint_account_info.key,
            associated_token_account_info.key,
            &mint_authority_key,
            &[],
            1,
        )?,
        &[
            mint_account_info.clone(),
            associated_token_account_info.clone(),
            mint_authority_account_info.clone(),
        ],
        &[&[b"mint_authority", &[mint_authority_bump]]],
    )?;

    let mut creators = Vec::new();
    creators.push(Creator {
        address: mint_authority_key,
        verified: true,
        share: 100,
    });

    let mpl_token_metadata_id = mpl_token_metadata::id();
    let metadata_seeds = &[
        PREFIX.as_bytes(),
        mpl_token_metadata_id.as_ref(),
        mint_account_info.key.as_ref(),
    ];

    let (nft_metadata_key, _nft_metadata_bump) =
        Pubkey::find_program_address(metadata_seeds, &mpl_token_metadata_id);

    if nft_metadata_key != *meta_data_account_info.key {
        Err(ProgramError::Custom(4))?
    }

    msg!("Create metaplex nft account v3");
    invoke_signed(
        &mpl_token_metadata::instruction::create_metadata_accounts_v3(
            mpl_token_metadata_id,
            nft_metadata_key,
            *mint_account_info.key,
            *mint_authority_account_info.key,
            *payer_account_info.key,
            *mint_authority_account_info.key,
            String::from("Ingl Collection"),
            String::from("INGL#COL"),
            String::from("https://cdn.discordapp.com/attachments/952653904376659968/999014566505750692/0001-0300.mp4??ext=mp4"),
            Some(creators),
            300,
            true,
            true,
            None,
            None,
            None,
        ),
        &[
            meta_data_account_info.clone(),
            mint_account_info.clone(),
            mint_authority_account_info.clone(),
            payer_account_info.clone(),
            mint_authority_account_info.clone(),
            system_program_account_info.clone(),
            sysvar_rent_accoount_info.clone(),
        ],
        &[&[b"mint_authority", &[mint_authority_bump]]]
    )?;

    msg!("Setting mint authority");
    invoke_signed(
        &spl_token::instruction::set_authority(
            spl_token_program_account_info.key,
            mint_account_info.key,
            None,
            AuthorityType::MintTokens,
            &mint_authority_key,
            &[],
        )?,
        &[
            mint_account_info.clone(),
            mint_authority_account_info.clone(),
        ],
        &[&[b"mint_authority", &[mint_authority_bump]]],
    )?;
    
    Ok(())
}
