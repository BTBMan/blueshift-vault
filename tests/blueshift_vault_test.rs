use blueshift_vault::ID;
use mollusk_svm::{program::keyed_account_for_system_program, result::Check, Mollusk};
use pinocchio::Address;
use solana_account::Account;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey,
};

#[test]
fn test() {
    let mollusk = Mollusk::new(&ID, "target/deploy/blueshift_vault");

    // 创建签名者测试账户 (模拟钱包)
    let payer = pubkey::Pubkey::new_unique(); // 生成账户所对应的地址
    let payer_account = Account::new(10_000_000_000, 0, &pinocchio_system::ID); // 创建账户

    // 创建 vault PDA
    let (vault_pda, ..) = Address::find_program_address(&[b"vault", payer.as_ref()], &ID);
    let vault_pda_account = Account::new(0, 0, &pinocchio_system::ID);

    // 获取系统程序和系统账户
    let (system_program, system_program_account) = keyed_account_for_system_program();

    // 组合成 mollusk 所须要的格式
    let accounts = [
        (payer, payer_account),
        (vault_pda, vault_pda_account),
        (system_program, system_program_account),
    ];

    // 创建 deposit 指令
    // accounts 里装了指令所需的账户地址, 这里只有一个, 是一个可变的签名账户
    let deposit_instruction = Instruction::new_with_bytes(
        ID,
        &[&[0u8], &5_000_000_000u64.to_le_bytes()[..]].concat(),
        vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(system_program, false),
        ],
    );

    // 创建 withdraw 指令
    let withdraw_instruction = Instruction::new_with_bytes(
        ID,
        &[1u8],
        vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(system_program, false),
        ],
    );

    let r = mollusk.process_and_validate_instruction_chain(
        &[
            (
                &deposit_instruction,
                &[
                    Check::success(),
                    Check::account(&payer).lamports(5_000_000_000).build(),
                    Check::account(&vault_pda).lamports(5_000_000_000).build(),
                ],
            ),
            (
                &withdraw_instruction,
                &[
                    Check::success(),
                    Check::account(&payer).lamports(10_000_000_000).build(),
                    Check::account(&vault_pda).lamports(0).build(),
                ],
            ),
        ],
        &accounts,
    );

    println!("{:?}", r);
}
