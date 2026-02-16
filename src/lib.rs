#![no_std]

use pinocchio::{
    address::address, entrypoint, error::ProgramError, nostd_panic_handler, AccountView, Address,
    ProgramResult,
};

entrypoint!(process_instruction);
nostd_panic_handler!();

pub mod instructions;
pub use instructions::*;

// 22222222222222222222222222222222222222222222
pub const ID: Address = address!("22222222222222222222222222222222222222222222");

fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    // 取出 instruction_data 切片中的第一个元素, 用来匹配调用的指令
    match instruction_data.split_first() {
        Some((Deposit::DISCRIMINATOR, data)) => Deposit::try_from((data, accounts))?.process(),
        Some((Withdraw::DISCRIMINATOR, _)) => Withdraw::try_from(accounts)?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
