use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};
use pinocchio_system::instructions::Transfer;
use solana_program_log::log;

// 定义金库存钱所需要的账户列表
pub struct DepositAccounts<'a> {
    // 定义金库的所有者, 即是创建者, 也是 signer
    pub owner: &'a AccountView,
    // 定义金库 PDA 账户
    pub vault: &'a AccountView,
}

// 为账户列表结构体实现 TryFrom trait
impl<'a> TryFrom<&'a [AccountView]> for DepositAccounts<'a> {
    // 设置 TryFrom trait 中的 Error 类型为 ProgramError
    type Error = ProgramError;

    // 实现 try_from 方法
    // 参数为账户列表切片
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        log!("accounts length: {}", accounts.len());

        // 解构账户列表, 获取 owner address 和 vault PDA
        let [owner, vault, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 验证 owner 是不是签名者
        if !owner.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // 验证 vault PDA 的所有者是不是系统程序
        // 因为 PDA 只是一个地址, 没有被重新分配 owner, 所以它的 owner 必须是系统程序
        if !vault.owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // 验证 vault 的余额必须为 0
        if vault.lamports().ne(&0) {
            return Err(ProgramError::InvalidAccountData);
        }

        // 通过金库创建者地址和金库 PDA 程序地址, 计算出金库 PDA 地址
        let (vault_key, _) =
            Address::find_program_address(&[b"vault", owner.address().as_ref()], &crate::ID);

        // 验证 vault PDA 地址是否正确
        if vault.address().ne(&vault_key) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // 将金库的创建者和 PDA 账户返回
        Ok(Self { owner, vault })
    }
}

// 定义金库存钱指令的数据字段结构体
pub struct DepositInstructionData {
    // 定义金库存钱的金额
    pub amount: u64,
}

// 为数据结构体实现 TryFrom trait
impl<'a> TryFrom<&'a [u8]> for DepositInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        // 验证 data 的长度是否和 u64 的字节长度相同(8 字节)
        if data.len() != size_of::<u64>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        // 将 data 以小端序转换为 u64, 真正的 amount
        let amount = u64::from_le_bytes(data.try_into().unwrap());

        // 验证 amount 必须不为 0, 且不能超过 u64 的最大值
        if amount.eq(&0) || amount.gt(&u64::MAX) {
            return Err(ProgramError::InvalidInstructionData);
        }

        // 将 amount 返回
        Ok(Self { amount })
    }
}

// 定义 Deposit 指令的结构体
pub struct Deposit<'a> {
    // 指令所用到的账户
    pub accounts: DepositAccounts<'a>,
    // 指令所接收的数据
    pub instruction_data: DepositInstructionData,
}

// 为 Deposit 结构体实现 TryFrom trait
impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for Deposit<'a> {
    type Error = ProgramError;

    // 接收一个元祖 (data, accounts)
    fn try_from((data, accounts): (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
        // 试图转换为 DepositAccounts 结构体
        let accounts = DepositAccounts::try_from(accounts)?;
        // 试图转换为 DepositInstructionData 结构体
        let instruction_data = DepositInstructionData::try_from(data)?;

        // 返回 Deposit 结构体
        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

// 实现 Deposit 结构体
impl<'a> Deposit<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0; // 定义指令的标识符常量

    // 定义 process 方法, 此方法为指令所调用的方法
    pub fn process(&mut self) -> ProgramResult {
        // CPI 调用系统程序的 transfer 指令进行转账
        Transfer {
            from: self.accounts.owner,              // 从创建者账户
            to: self.accounts.vault,                // 转到金库 PDA 账户
            lamports: self.instruction_data.amount, // 转账金额
        }
        .invoke()?;

        Ok(())
    }
}
