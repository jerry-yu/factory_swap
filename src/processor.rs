use crate::instruction::FactoryInstruction;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::{PrintProgramError, ProgramError},
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::{Account as SplAccount, Mint};

pub struct Processor {}

impl Processor {
    pub fn process(
        program_id: &Pubkey, // Public key of the account the hello world program was loaded into
        accounts: &[AccountInfo], // The account to say hello to
        instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
    ) -> ProgramResult {
        Processor::process_instruction(program_id, accounts, instruction_data)
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

                let mint_a_authority = mint_a_info
                    .mint_authority
                    .ok_or(ProgramError::InvalidArgument)?;
                let mint_b_authority = mint_b_info
                    .mint_authority
                    .ok_or(ProgramError::InvalidArgument)?;
                if mint_b_authority != *mint_authority.key
                    || mint_a_authority != *mint_authority.key
                {
                    return Err(ProgramError::InvalidAccountData);
                }
                account_a_token_info
                    .amount
                    .checked_sub(amount)
                    .ok_or(ProgramError::InsufficientFunds)?;

                let ix = spl_token::instruction::mint_to(
                    spl_id.key,
                    mint_b.key,
                    account_b.key,
                    mint_authority.key,
                    &[owner.key],
                    amount,
                )?;
                SplAccount::pack(account_b_token_info, &mut account_b.data.borrow_mut())?;
                return invoke(
                    &ix,
                    &[
                        mint_b.clone(),
                        account_b.clone(),
                        mint_authority.clone(),
                        spl_id.clone(),
                    ],
                );
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

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::{instruction::Instruction, program_stubs, rent::Rent};
    use solana_sdk::account::{create_account_for_test, create_is_signer_account_infos, Account};
    use spl_token::{
        error::TokenError,
        instruction::{
            approve, initialize_account, initialize_mint, mint_to, revoke, set_authority,
            AuthorityType,
        },
    };

    // Test program id for the swap program.
    const SWAP_PROGRAM_ID: Pubkey = Pubkey::new_from_array([2u8; 32]);

    struct TestSyscallStubs {}
    impl program_stubs::SyscallStubs for TestSyscallStubs {
        fn sol_invoke_signed(
            &self,
            instruction: &Instruction,
            account_infos: &[AccountInfo],
            signers_seeds: &[&[&[u8]]],
        ) -> ProgramResult {
            msg!("TestSyscallStubs::sol_invoke_signed()");

            let mut new_account_infos = vec![];

            // mimic check for token program in accounts
            if !account_infos.iter().any(|x| *x.key == spl_token::id()) {
                return Err(ProgramError::InvalidAccountData);
            }

            for meta in instruction.accounts.iter() {
                for account_info in account_infos.iter() {
                    if meta.pubkey == *account_info.key {
                        let mut new_account_info = account_info.clone();
                        for seeds in signers_seeds.iter() {
                            let signer =
                                Pubkey::create_program_address(&seeds, &SWAP_PROGRAM_ID).unwrap();
                            if *account_info.key == signer {
                                new_account_info.is_signer = true;
                            }
                        }
                        new_account_infos.push(new_account_info);
                    }
                }
            }

            spl_token::processor::Processor::process(
                &instruction.program_id,
                &new_account_infos,
                &instruction.data,
            )
        }
    }

    fn test_syscall_stubs() {
        use std::sync::Once;
        static ONCE: Once = Once::new();

        ONCE.call_once(|| {
            program_stubs::set_syscall_stubs(Box::new(TestSyscallStubs {}));
        });
    }

    // pub fn setup_token_accounts(
    //     &mut self,
    //     mint_owner: &Pubkey,
    //     account_owner: &Pubkey,
    //     a_amount: u64,
    //     b_amount: u64,
    //     pool_amount: u64,
    // ) -> (Pubkey, Account, Pubkey, Account, Pubkey, Account) {
    //     let (token_a_key, token_a_account) = mint_token(
    //         &spl_token::id(),
    //         &self.token_a_mint_key,
    //         &mut self.token_a_mint_account,
    //         &mint_owner,
    //         &account_owner,
    //         a_amount,
    //     );
    //     let (token_b_key, token_b_account) = mint_token(
    //         &spl_token::id(),
    //         &self.token_b_mint_key,
    //         &mut self.token_b_mint_account,
    //         &mint_owner,
    //         &account_owner,
    //         b_amount,
    //     );
    //     let (pool_key, pool_account) = mint_token(
    //         &spl_token::id(),
    //         &self.pool_mint_key,
    //         &mut self.pool_mint_account,
    //         &self.authority_key,
    //         &account_owner,
    //         pool_amount,
    //     );
    //     (
    //         token_a_key,
    //         token_a_account,
    //         token_b_key,
    //         token_b_account,
    //         pool_key,
    //         pool_account,
    //     )
    // }

    fn mint_minimum_balance() -> u64 {
        Rent::default().minimum_balance(spl_token::state::Mint::get_packed_len())
    }

    fn account_minimum_balance() -> u64 {
        Rent::default().minimum_balance(spl_token::state::Account::get_packed_len())
    }

    fn do_process_instruction(
        instruction: Instruction,
        accounts: Vec<&mut Account>,
    ) -> ProgramResult {
        test_syscall_stubs();

        // approximate the logic in the actual runtime which runs the instruction
        // and only updates accounts if the instruction is successful
        let mut account_clones = accounts.iter().map(|x| (*x).clone()).collect::<Vec<_>>();
        let mut meta = instruction
            .accounts
            .iter()
            .zip(account_clones.iter_mut())
            .map(|(account_meta, account)| (&account_meta.pubkey, account_meta.is_signer, account))
            .collect::<Vec<_>>();
        let mut account_infos = create_is_signer_account_infos(&mut meta);
        let res = if instruction.program_id == SWAP_PROGRAM_ID {
            Processor::process_instruction(
                &instruction.program_id,
                &account_infos,
                &instruction.data,
            )
        } else {
            spl_token::processor::Processor::process(
                &instruction.program_id,
                &account_infos,
                &instruction.data,
            )
        };

        if res.is_ok() {
            let mut account_metas = instruction
                .accounts
                .iter()
                .zip(accounts)
                .map(|(account_meta, account)| (&account_meta.pubkey, account))
                .collect::<Vec<_>>();
            for account_info in account_infos.iter_mut() {
                for account_meta in account_metas.iter_mut() {
                    if account_info.key == account_meta.0 {
                        let account = &mut account_meta.1;
                        account.owner = *account_info.owner;
                        account.lamports = **account_info.lamports.borrow();
                        account.data = account_info.data.borrow().to_vec();
                    }
                }
            }
        }
        res
    }

    fn mint_token(
        program_id: &Pubkey,
        mint_key: &Pubkey,
        mut mint_account: &mut Account,
        mint_authority_key: &Pubkey,
        account_owner_key: &Pubkey,
        amount: u64,
    ) -> (Pubkey, Account) {
        let account_key = Pubkey::new_unique();
        let mut account_account = Account::new(
            account_minimum_balance(),
            spl_token::state::Account::get_packed_len(),
            &program_id,
        );
        let mut mint_authority_account = Account::default();
        let mut rent_sysvar_account = create_account_for_test(&Rent::free());

        do_process_instruction(
            initialize_account(&program_id, &account_key, &mint_key, account_owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut mint_authority_account,
                &mut rent_sysvar_account,
            ],
        )
        .unwrap();

        if amount > 0 {
            do_process_instruction(
                mint_to(
                    &program_id,
                    &mint_key,
                    &account_key,
                    &mint_authority_key,
                    &[],
                    amount,
                )
                .unwrap(),
                vec![
                    &mut mint_account,
                    &mut account_account,
                    &mut mint_authority_account,
                ],
            )
            .unwrap();
        }

        (account_key, account_account)
    }

    fn create_mint(
        program_id: &Pubkey,
        authority_key: &Pubkey,
        freeze_authority: Option<&Pubkey>,
    ) -> (Pubkey, Account) {
        let mint_key = Pubkey::new_unique();
        let mut mint_account = Account::new(
            mint_minimum_balance(),
            spl_token::state::Mint::get_packed_len(),
            &program_id,
        );
        let mut rent_sysvar_account = create_account_for_test(&Rent::free());

        do_process_instruction(
            initialize_mint(&program_id, &mint_key, authority_key, freeze_authority, 2).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar_account],
        )
        .unwrap();

        (mint_key, mint_account)
    }
}
