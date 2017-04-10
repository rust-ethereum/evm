use utils::u256::U256;
use merkle::{MerkleTree};
use crypto::sha3::Sha3;

pub struct Account {
    nonce: usize,
    balance: U256,
    pub storageRoot: MerkleTree<U256>,
    codeHash: U256,
}

impl Default for Account {
    fn default() -> Account {
        Account {
            nonce: 0,
            balance: 0.into(),
            storageRoot: MerkleTree::from_vec::<U256>(Sha3::keccak256(), Vec::new()),
            codeHash: 0.into()
        }
    }
}
