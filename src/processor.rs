
use solana_program::{
    account_info::{next_account_info,AccountInfo},
    entrypoint, entrypoint::ProgramResult,
    program_error::{ProgramError,PrintProgramError}, 
    pubkey::Pubkey,
    program::invoke,
    program_option::COption,
    msg,program_pack::Pack
};
use crate::instruction::FactoryInstruction;
use borsh::{BorshDeserialize, BorshSerialize};
use spl_token::state::{Mint,Account as SplAccount};


pub struct Processor {}

impl Processor {

pub fn process(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &[AccountInfo], // The account to say hello to
    instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
)-> ProgramResult {
    Processor::process_instruction(program_id,accounts,instruction_data)
}

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo], 
    instruction_data: &[u8], 
) -> ProgramResult {
    msg!("process_instruction");

    let inst = FactoryInstruction::try_from_slice(instruction_data)?;
    match inst {
        FactoryInstruction::Recv(amount) => {
            let accounts_iter = &mut accounts.iter();
            let owner = next_account_info(accounts_iter)?;
            let account_a = next_account_info(accounts_iter)?;
            let account_b = next_account_info(accounts_iter)?;
            let mint_a = next_account_info(accounts_iter)?;
            let mint_b = next_account_info(accounts_iter)?;
            let mint_authority = next_account_info(accounts_iter)?;
            let spl_id = next_account_info(accounts_iter)?;

            let account_a_token_info = SplAccount::unpack(&account_a.data.borrow())?;
            let account_b_token_info = SplAccount::unpack(&account_b.data.borrow())?;
            
            if account_a_token_info.owner != *owner.key {
                return Err(ProgramError::IllegalOwner);
            }

            if account_a_token_info.amount < amount {
                return Err(ProgramError::InsufficientFunds);
            }

            let mint_a_info = Mint::unpack(&mint_a.data.borrow())?;
            let mint_b_info = Mint::unpack(&mint_b.data.borrow())?;

            // if mint_a_info.mint_authority != mint_a.key || mint_b_info.mint_authority != mint_b.key {
            //     return Err(ProgramError::BorshIoError("mint key not equal".to_string()));
            // }

            let mint_a_authority = mint_a_info.mint_authority.ok_or(ProgramError::InvalidArgument)?;
            let mint_b_authority = mint_b_info.mint_authority.ok_or(ProgramError::InvalidArgument)?;
            if mint_b_authority != *mint_authority.key || mint_a_authority != *mint_authority.key {
                return Err(ProgramError::InvalidAccountData);
            }
            account_a_token_info.amount.checked_sub(amount).ok_or(ProgramError::InsufficientFunds)?;

            let ix = spl_token::instruction::mint_to(
                spl_id.key,
                mint_b.key,
                account_b.key,
                mint_authority.key,
                &[owner.key],
                amount,
            )?;
            SplAccount::pack(account_b_token_info, &mut account_b.data.borrow_mut())?;
            return invoke(&ix, &[mint_b.clone(), account_b.clone(), mint_authority.clone(), spl_id.clone()]);
        }

    }

    // // Iterating accounts is safer then indexing
    // let accounts_iter = &mut accounts.iter();

    // // Get the account to say hello to
    // let account = next_account_info(accounts_iter)?;

    // // The account must be owned by the program in order to modify its data
    // if account.owner != program_id {
    //     msg!("Greeted account does not have the correct program id");
    //     return Err(ProgramError::IncorrectProgramId);
    // }

    // // Increment and store the number of times the account has been greeted
    // let mut greeting_account = GreetingAccount::try_from_slice(&account.data.borrow())?;
    // greeting_account.counter += 1;
    // greeting_account.serialize(&mut &mut account.data.borrow_mut()[..])?;

    // msg!("Greeted {} time(s)!", greeting_account.counter);

    Ok(())
}
}