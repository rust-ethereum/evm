use utils::u256::U256;
use ring::digest::{Algorithm, Context, SHA256};
use merkle::{MerkleTree};

pub struct Account {
    nonce: usize,
    balance: U256,
    pub storageRoot: MerkleTree<U256>,
    codeHash: U256,
}

static digest: &'static Algorithm = &SHA256;

impl Default for Account {
    fn default() -> Account {
        Account {
            nonce: 0,
            balance: 0.into(),
            storageRoot: MerkleTree::from_vec(digest, Vec::new()),
            codeHash: 0.into()
        }
    }
}
