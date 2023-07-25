use borsh::{BorshDeserialize, BorshSerialize}; 
use mpl_token_metadata::instruction as mpl_instruction; 
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,    
    program_pack::Pack, 
    pubkey::Pubkey,
    rent::Rent,         
    system_instruction, 
    sysvar::Sysvar, borsh::try_from_slice_unchecked,     
};
use spl_token::{instruction as token_instruction, state::Mint}; 

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let token_metadata = CreateTokenArgs::try_from_slice(instruction_data)?;

    // try_from_slice_unchecked(data)

    msg!("nft title: {:?} \n", token_metadata.nft_title);
    msg!("nft symbol: {:?} \n", token_metadata.nft_symbol);
    msg!("nft uri: {:?} \n", token_metadata.nft_uri);

    msg!("these are the accounts passed {:?}", accounts);
    // msg!("this is the data passed \n");
    // msg!("this is the data passed \n");

    create_nft(accounts, token_metadata)?;

    Ok(())
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CreateTokenArgs {
    pub nft_title: String,
    pub nft_symbol: String,
    pub nft_uri: String,
}

fn create_nft(accounts: &[AccountInfo], create_token_metadata: CreateTokenArgs) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    //pick out our accounts
    let mint_account = next_account_info(accounts_iter)?;
    let mint_authority = next_account_info(accounts_iter)?;
    let metadata_account = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let rent = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let token_metadata_program = next_account_info(accounts_iter)?;

    // invoke the system program to create our account
    msg!("Creating mint account...");
    msg!("Mint: {}", mint_account.key);
    invoke(
        &system_instruction::create_account(
            &payer.key,
            &mint_account.key,
            (Rent::get()?).minimum_balance(Mint::LEN),
            Mint::LEN as u64,
            &token_program.key,
        ),
        &[
            mint_account.clone(),
            payer.clone(),
            system_program.clone(),
            token_program.clone(),
        ],
    )?;

    //initialize the created account as the mint
    msg!("Initializing mint account...");
    msg!("Mint: {}", mint_account.key);
    invoke(
        &token_instruction::initialize_mint(
            &token_program.key,
            &mint_account.key,
            &mint_authority.key,
            Some(&mint_authority.key),
            0,
        )?,
        &[
            mint_account.clone(),
            mint_authority.clone(),
            token_program.clone(),
            rent.clone(),
        ],
    )?;

    //call Metaplex token metadata program
    msg!("Creating metadata account...");
    msg!("Metadata account address: {}", metadata_account.key);
    invoke(
        &mpl_instruction::create_metadata_accounts_v3(
            *token_metadata_program.key,      //program id
            *metadata_account.key,            // metadata address
            *mint_account.key,                //mint address
            *mint_authority.key,              //mint authority address
            *payer.key,                       //payer public key address
            *mint_authority.key,              //update authority address
            create_token_metadata.nft_title,  //nft title
            create_token_metadata.nft_symbol, //nft symbol
            create_token_metadata.nft_uri,    //nft uri
            None,  //creators. for something you pass in a VEC of their addresses
            0,     //royalties
            true,  //is the payer the update authority
            false, //can we update the token metadata
            None,  //collection the nft belongs to. struct of pubkey and
            None,  //uses
            None,  //collection details
        ),
        &[
            metadata_account.clone(),
            mint_account.clone(),
            mint_authority.clone(),
            payer.clone(),
            token_metadata_program.clone(),
            rent.clone(),
        ],
    )?;

    msg!("Token mint created successfully.");

    Ok(())
}
