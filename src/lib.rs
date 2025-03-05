use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    program::invoke,
    system_instruction,
};
use spl_token_2022::{
    instruction as token_instruction,
    state::{Account, Mint},
};

mod rewards;

// Declare the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = TokenInstruction::unpack(instruction_data)?;

    match instruction {
        TokenInstruction::InitializeMint { decimals, mint_authority } => {
            msg!("Instruction: InitializeMint");
            process_initialize_mint(program_id, accounts, decimals, mint_authority)
        }
        TokenInstruction::MintTo { amount } => {
            msg!("Instruction: MintTo");
            process_mint_to(program_id, accounts, amount)
        }
        TokenInstruction::Transfer { amount, is_buy } => {
            msg!("Instruction: Transfer");
            process_transfer(program_id, accounts, amount, is_buy)
        }
        TokenInstruction::UpdateHolderBalance { holder, balance } => {
            msg!("Instruction: UpdateHolderBalance");
            process_update_holder_balance(program_id, accounts, holder, balance)
        }
    }
}

#[derive(Debug)]
enum TokenInstruction {
    InitializeMint {
        decimals: u8,
        mint_authority: Option<Pubkey>,
    },
    MintTo {
        amount: u64,
    },
    Transfer {
        amount: u64,
        is_buy: bool,
    },
    UpdateHolderBalance {
        holder: Pubkey,
        balance: u64,
    },
}

impl TokenInstruction {
    fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input.split_first().ok_or(ProgramError::InvalidInstructionData)?;
        Ok(match tag {
            0 => {
                let (decimals, rest) = rest.split_at(1);
                let (mint_authority, _) = rest.split_at(32);
                let mint_authority = if mint_authority.iter().all(|&x| x == 0) {
                    None
                } else {
                    Some(Pubkey::new_from_array(mint_authority.try_into().unwrap()))
                };
                Self::InitializeMint {
                    decimals: decimals[0],
                    mint_authority,
                }
            }
            1 => {
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(ProgramError::InvalidInstructionData)?;
                Self::MintTo { amount }
            }
            2 => {
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(ProgramError::InvalidInstructionData)?;
                let is_buy = rest.get(8).map(|&x| x != 0).unwrap_or(false);
                Self::Transfer { amount, is_buy }
            }
            3 => {
                let (holder, rest) = rest.split_at(32);
                let balance = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(ProgramError::InvalidInstructionData)?;
                Self::UpdateHolderBalance {
                    holder: Pubkey::new_from_array(holder.try_into().unwrap()),
                    balance,
                }
            }
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}

#[derive(Debug)]
struct TransferFeeConfig {
    buy_fee_basis_points: u16,  // 5% = 500 basis points
    sell_fee_basis_points: u16, // 5% = 500 basis points
    fee_collector: Pubkey,
    rewards_program: Pubkey,
}

fn process_initialize_mint(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    decimals: u8,
    mint_authority: Option<Pubkey>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let mint_account = next_account_info(account_info_iter)?;
    let rent = next_account_info(account_info_iter)?;
    let fee_collector = next_account_info(account_info_iter)?;
    let rewards_program = next_account_info(account_info_iter)?;

    // Verify the mint account is owned by the program
    if mint_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Initialize the mint account
    let mint = Mint {
        mint_authority,
        freeze_authority: None,
        decimals,
        is_initialized: true,
    };

    let mut mint_data = mint_account.data.borrow_mut();
    mint.serialize(&mut &mut mint_data[..])?;

    // Store transfer fee configuration with 5% fees
    let fee_config = TransferFeeConfig {
        buy_fee_basis_points: 500,  // 5%
        sell_fee_basis_points: 500, // 5%
        fee_collector: *fee_collector.key,
        rewards_program: *rewards_program.key,
    };

    // Store fee config after mint data
    let fee_config_data = bincode::serialize(&fee_config)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    mint_data[mint.serialized_len()..mint.serialized_len() + fee_config_data.len()]
        .copy_from_slice(&fee_config_data);

    Ok(())
}

fn process_mint_to(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let mint_account = next_account_info(account_info_iter)?;
    let destination_account = next_account_info(account_info_iter)?;
    let authority_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    // Verify the mint account is owned by the program
    if mint_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Create mint instruction
    let mint_instruction = token_instruction::mint_to(
        token_program.key,
        mint_account.key,
        destination_account.key,
        authority_account.key,
        &[],
        amount,
    )?;

    // Execute the mint instruction
    invoke(
        &mint_instruction,
        &[
            mint_account.clone(),
            destination_account.clone(),
            authority_account.clone(),
            token_program.clone(),
        ],
    )?;

    Ok(())
}

fn process_transfer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    is_buy: bool,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let source_account = next_account_info(account_info_iter)?;
    let destination_account = next_account_info(account_info_iter)?;
    let authority_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;

    // Get transfer fee configuration
    let mint_data = mint_account.data.borrow();
    let mint = Mint::deserialize(&mint_data)?;
    let fee_config: TransferFeeConfig = bincode::deserialize(
        &mint_data[mint.serialized_len()..mint.serialized_len() + std::mem::size_of::<TransferFeeConfig>()],
    ).map_err(|_| ProgramError::InvalidInstructionData)?;

    // Calculate transfer fee based on whether it's a buy or sell
    let fee_basis_points = if is_buy {
        fee_config.buy_fee_basis_points
    } else {
        fee_config.sell_fee_basis_points
    };

    let fee_amount = (amount as u128)
        .checked_mul(fee_basis_points as u128)
        .ok_or(ProgramError::Overflow)?
        .checked_div(10000)
        .ok_or(ProgramError::Overflow)? as u64;

    // Transfer the fee to the fee collector
    let fee_collector_account = next_account_info(account_info_iter)?;
    let fee_transfer_instruction = token_instruction::transfer(
        token_program.key,
        source_account.key,
        fee_collector_account.key,
        authority_account.key,
        &[],
        fee_amount,
    )?;

    invoke(
        &fee_transfer_instruction,
        &[
            source_account.clone(),
            fee_collector_account.clone(),
            authority_account.clone(),
            token_program.clone(),
        ],
    )?;

    // Transfer the remaining amount to the destination
    let remaining_amount = amount.checked_sub(fee_amount).ok_or(ProgramError::Overflow)?;
    let transfer_instruction = token_instruction::transfer(
        token_program.key,
        source_account.key,
        destination_account.key,
        authority_account.key,
        &[],
        remaining_amount,
    )?;

    invoke(
        &transfer_instruction,
        &[
            source_account.clone(),
            destination_account.clone(),
            authority_account.clone(),
            token_program.clone(),
        ],
    )?;

    // Update holder balance in rewards program
    let update_balance_instruction = create_update_holder_balance_instruction(
        program_id,
        destination_account.key,
        remaining_amount,
    )?;

    invoke(
        &update_balance_instruction,
        &[
            mint_account.clone(),
            destination_account.clone(),
            fee_config.rewards_program,
        ],
    )?;

    Ok(())
}

fn process_update_holder_balance(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    holder: Pubkey,
    balance: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let mint_account = next_account_info(account_info_iter)?;
    let rewards_program = next_account_info(account_info_iter)?;

    // Verify the rewards program
    let mint_data = mint_account.data.borrow();
    let mint = Mint::deserialize(&mint_data)?;
    let fee_config: TransferFeeConfig = bincode::deserialize(
        &mint_data[mint.serialized_len()..mint.serialized_len() + std::mem::size_of::<TransferFeeConfig>()],
    ).map_err(|_| ProgramError::InvalidInstructionData)?;

    if rewards_program.key != &fee_config.rewards_program {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Forward the update to the rewards program
    let update_instruction = rewards::create_update_holder_balance_instruction(
        holder,
        balance,
    )?;

    invoke(
        &update_instruction,
        &[
            mint_account.clone(),
            rewards_program.clone(),
        ],
    )?;

    Ok(())
}

// Helper function to create update holder balance instruction
fn create_update_holder_balance_instruction(
    program_id: &Pubkey,
    holder: &Pubkey,
    balance: u64,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let mut data = Vec::new();
    data.push(3); // UpdateHolderBalance instruction tag
    data.extend_from_slice(holder.as_ref());
    data.extend_from_slice(&balance.to_le_bytes());

    Ok(solana_program::instruction::Instruction {
        program_id: *program_id,
        accounts: vec![],
        data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::clock::Epoch;

    #[test]
    fn test_sanity() {
        // Add tests here
    }
} 