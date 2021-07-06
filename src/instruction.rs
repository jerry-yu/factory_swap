use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use std::convert::TryInto;
use std::mem::size_of;
use borsh::{BorshDeserialize, BorshSerialize};


/// Define the type of state stored in accounts
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum FactoryInstruction {
    ///   0. `[signer]` owner of source token a account
    ///   1. `[writable]` A account from Token A. source account
    ///   2. `[writable]` B account from Token B.  destination account
    ///   3. `[]` token_a mint.
    ///   4. `[]` token_b mint.
    ///   5. `[]` mint authority ： Token A，Token B same。 maybe multi-sign
    ///   6. '[]` Token program id
    Recv(u64),
}
