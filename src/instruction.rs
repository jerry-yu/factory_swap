use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::convert::TryInto;
use std::mem::size_of;

/// Define the type of state stored in accounts
#[derive(Debug)]
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

impl FactoryInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input.split_first().ok_or(ProgramError::InvalidArgument)?;
        Ok(match tag {
            1 => {
                if rest.len() < 8 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                Self::Recv(u64::from_le_bytes(rest[0..8].try_into().unwrap()))
            }
            _ => return Err(ProgramError::InvalidArgument),
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::Recv(amount) => {
                buf.push(1);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
        };
        buf
    }
}

///   0. `[signer]` owner of source token a account
///   1. `[writable]` A account from Token A. source account
///   2. `[writable]` B account from Token B.  destination account
///   3. `[]` token_a mint.
///   4. `[]` token_b mint.
///   5. `[]` mint authority ： Token A，Token B same。 maybe multi-sign
///   6. '[]` Token program id

pub fn instruction_recv(
    program_id: &Pubkey,
    amount: u64,
    owner: &Pubkey,
    acount_a_token: &Pubkey,
    acount_b_token: &Pubkey,
    acount_a_mint: &Pubkey,
    acount_b_mint: &Pubkey,
    mint_authority_pubkey: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let data = FactoryInstruction::Recv(amount).pack();

    let accounts = vec![
        AccountMeta::new(*owner, true),
        AccountMeta::new(*acount_a_token, false),
        AccountMeta::new(*acount_a_token, false),
        AccountMeta::new(*acount_a_mint, false),
        AccountMeta::new(*acount_b_mint, false),
        AccountMeta::new(*mint_authority_pubkey, false),
        AccountMeta::new(spl_token::id(), false),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}
