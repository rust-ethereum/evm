//! # Network-Agnostic SputnikVM Patches
//!
//! This crate provides re-exports of the DynamicPatch API, and a set of precompiled contracts
//! covering everything up to ETH Constantinople
//!
//! There are two major approaches to the EVM configuration:
//!  - [Dynamic Patch](#dynamic-patch-api)
//!  - [Patch](#patch-api)
//!
//! Examples for both approaches may be found below
//!
//! # Dynamic Patch API
//!
//! DynamicPatch API is most useful for multi-network clients like multi-geth, where
//! it's preferable to configure the EVM feature-wise, instead of fork-wise.
//!
//! ### Example
//!
//! ```
//! use evm::{SeqTransactionVM, ValidTransaction, TransactionAction, HeaderParams};
//! use evm_network::{DynamicPatch, DynamicAccountPatch, PRECOMPILEDS};
//! use bigint::{Gas, U256, Address};
//! use std::rc::Rc;
//!
//! fn main() {
//!   let transaction = ValidTransaction {
//!      caller: Some(Address::default()),
//!      gas_price: Gas::zero(),
//!      gas_limit: Gas::max_value(),
//!      action: TransactionAction::Create,
//!      value: U256::zero(),
//!      input: Rc::new(Vec::new()),
//!      nonce: U256::zero()
//!   };
//!
//!   // Block Header
//!   let header = HeaderParams {
//!      beneficiary: Address::default(),
//!      timestamp: 0,
//!      number: U256::zero(),
//!      difficulty: U256::zero(),
//!      gas_limit: Gas::zero()
//!   };
//!
//!   // Account Patch for ETC MainNet
//!   let account_patch = DynamicAccountPatch {
//!      initial_nonce: U256::zero(),
//!      initial_create_nonce: U256::zero(),
//!      empty_considered_exists: true,
//!      allow_partial_change: true
//!   };
//!
//!   // Patch for Constantinople hardfork
//!   let patch = DynamicPatch {
//!      account_patch,
//!      code_deposit_limit: None,
//!      callstack_limit: 1024,
//!      gas_extcode: Gas::from(700_usize),
//!      gas_balance: Gas::from(400_usize),
//!      gas_sload: Gas::from(200_usize),
//!      gas_suicide: Gas::from(5000_usize),
//!      gas_suicide_new_account: Gas::from(25000_usize),
//!      gas_call: Gas::from(700_usize),
//!      gas_expbyte: Gas::from(50_usize),
//!      gas_transaction_create: Gas::from(32000_usize),
//!      force_code_deposit: false,
//!      has_delegate_call: true,
//!      has_static_call: true,
//!      has_revert: true,
//!      has_return_data: true,
//!      has_bitwise_shift: true,
//!      has_extcodehash: true,
//!      has_create2: true,
//!      has_reduced_sstore_gas_metering: true,
//!      err_on_call_with_more_gas: false,
//!      call_create_l64_after_gas: true,
//!      memory_limit: usize::max_value(),
//!      // Enable all eight precompiled contracts by their addresses
//!      enabled_precompileds: (0x1..=0x8).into_iter().map(Address::from).collect(),
//!      precompileds: &PRECOMPILEDS
//!   };
//!
//!   SeqTransactionVM::new(
//!       &patch,
//!       transaction,
//!       header
//!   );
//! }
//! ```
//!
//! # Patch API
//!
//! If you need just a single network or even a single feature-set, use the [Patch](evm::Patch)
//! trait directly, that allows to create custom feature sets without DynamicPatch's tiny overhead.
//!
//! ### Example
//!
//! ```
//! use evm::{SeqTransactionVM, ValidTransaction, TransactionAction, HeaderParams, Precompiled};
//! use evm_network::{AccountPatch, Patch, PRECOMPILEDS};
//! use bigint::{Gas, U256, Address};
//! use std::rc::Rc;
//!
//! struct MainnetAccountPatch;
//! impl AccountPatch for MainnetAccountPatch {
//!    fn initial_nonce(&self) -> U256 { U256::zero() }
//!    fn initial_create_nonce(&self) -> U256 { U256::zero() }
//!    fn empty_considered_exists(&self) -> bool { true }
//! }
//!
//! static MAINNET_ACCOUNT_PATCH: MainnetAccountPatch = MainnetAccountPatch;
//!
//! struct ConstantinoplePatch;
//! impl Patch for ConstantinoplePatch {
//!    type Account = MainnetAccountPatch;
//!    fn account_patch(&self) -> &'static Self::Account { &MAINNET_ACCOUNT_PATCH }
//!    fn code_deposit_limit(&self) -> Option<usize> { None }
//!    fn callstack_limit(&self) -> usize { 1024 }
//!    fn gas_extcode(&self) -> Gas { Gas::from(700usize) }
//!    fn gas_balance(&self) -> Gas { Gas::from(400usize) }
//!    fn gas_sload(&self) -> Gas { Gas::from(200usize) }
//!    fn gas_suicide(&self) -> Gas { Gas::from(5000usize) }
//!    fn gas_suicide_new_account(&self) -> Gas { Gas::from(25000usize) }
//!    fn gas_call(&self) -> Gas { Gas::from(700usize) }
//!    fn gas_expbyte(&self) -> Gas { Gas::from(50usize) }
//!    fn gas_transaction_create(&self) -> Gas { Gas::from(32000usize) }
//!    fn force_code_deposit(&self) -> bool { false }
//!    fn has_delegate_call(&self) -> bool { true }
//!    fn has_static_call(&self) -> bool { true }
//!    fn has_revert(&self) -> bool { true }
//!    fn has_return_data(&self) -> bool { true }
//!    fn has_bitwise_shift(&self) -> bool { true }
//!    fn has_create2(&self) -> bool { true }
//!    fn has_extcodehash(&self) -> bool { true }
//!    fn has_reduced_sstore_gas_metering(&self) -> bool { true }
//!    fn err_on_call_with_more_gas(&self) -> bool { false }
//!    fn call_create_l64_after_gas(&self) -> bool { true }
//!    fn memory_limit(&self) -> usize { usize::max_value() }
//!    fn is_precompiled_contract_enabled(&self, address: &Address) -> bool {
//!        match address.low_u64() {
//!            0x1 | 0x2 | 0x3 | 0x4 | 0x5 | 0x6 | 0x7 | 0x8 => true,
//!            _ => false,
//!        }
//!    }
//!    fn precompileds(&self) -> &'static [(Address, Option<&'static [u8]>, &'static Precompiled)] {
//!        &PRECOMPILEDS
//!    }
//! }
//!
//!
//! fn main() {
//!   let transaction = ValidTransaction {
//!      caller: Some(Address::default()),
//!      gas_price: Gas::zero(),
//!      gas_limit: Gas::max_value(),
//!      action: TransactionAction::Create,
//!      value: U256::zero(),
//!      input: Rc::new(Vec::new()),
//!      nonce: U256::zero()
//!   };
//!
//!   // Block Header
//!   let header = HeaderParams {
//!      beneficiary: Address::default(),
//!      timestamp: 0,
//!      number: U256::zero(),
//!      difficulty: U256::zero(),
//!      gas_limit: Gas::zero()
//!   };
//!
//!   let patch = ConstantinoplePatch;
//!
//!   SeqTransactionVM::new(
//!       &patch,
//!       transaction,
//!       header
//!   );
//! }
//! ```

use bigint::{Address, H160};
use evm::{Precompiled, ECREC_PRECOMPILED, ID_PRECOMPILED, RIP160_PRECOMPILED, SHA256_PRECOMPILED};
use evm_precompiled_bn128::{BN128_ADD_PRECOMPILED, BN128_MUL_PRECOMPILED, BN128_PAIRING_PRECOMPILED};
use evm_precompiled_modexp::MODEXP_PRECOMPILED;

// Re-export DynamicPatch and Patch APIs
pub use evm::{AccountPatch, DynamicAccountPatch, DynamicPatch, Patch};

#[rustfmt::skip]
pub static PRECOMPILEDS: [(Address, Option<&'static [u8]>, &'static Precompiled); 8] = [
    (H160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01]),
     None,
     &ECREC_PRECOMPILED),
    (H160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x02]),
     None,
     &SHA256_PRECOMPILED),
    (H160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x03]),
     None,
     &RIP160_PRECOMPILED),
    (H160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x04]),
     None,
     &ID_PRECOMPILED),
    (H160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x05]),
     None,
     &MODEXP_PRECOMPILED),
    (H160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x06]),
     None,
     &BN128_ADD_PRECOMPILED),
    (H160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x07]),
     None,
     &BN128_MUL_PRECOMPILED),
    (H160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x08]),
     None,
     &BN128_PAIRING_PRECOMPILED),
];
