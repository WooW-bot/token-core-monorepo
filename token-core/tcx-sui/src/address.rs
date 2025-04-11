use core::str::FromStr;

use sp_core::bytes::to_hex;
use tcx_constants::CoinInfo;
use tcx_keystore::Address;
use tcx_keystore::PublicKeyEncoder;
use tcx_keystore::Result;
use tcx_primitive::TypedPublicKey;

pub const DEFAULT_HASH_SIZE: usize = 32;
pub const ED25519_FLAG: u8 = 0;
pub const SECP256K1_FLAG: u8 = 1;

#[derive(PartialEq, Eq, Clone)]
pub struct SuiAddress(pub String);

impl Address for SuiAddress {
    fn from_public_key(public_key: &TypedPublicKey, _coin: &CoinInfo) -> Result<Self> {
        let flag = match public_key {
            TypedPublicKey::Ed25519(_) => ED25519_FLAG,
            TypedPublicKey::Secp256k1(_) => SECP256K1_FLAG,
            _ => return Err(anyhow::anyhow!("Unsupported public key type")),
        };
        let mut result = [0u8; 32];
        let pk = public_key.to_bytes();
        let mut hasher = blake2b_rs::Blake2bBuilder::new(DEFAULT_HASH_SIZE).build();
        hasher.update(&[flag]);
        hasher.update(&pk);
        hasher.finalize(&mut result);
        let address = to_hex(&result, false);
        Ok(SuiAddress(address))
    }

    fn is_valid(address: &str, _coin: &CoinInfo) -> bool {
        if address.is_empty() {
            return false;
        };
        let address = address.strip_prefix("0x").unwrap_or(address);
        if address.len() != 64 {
            return false;
        }
        true
    }
}

impl FromStr for SuiAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if Self::is_valid(s, &CoinInfo::default()) {
            Ok(SuiAddress(s.to_string()))
        } else {
            Err(anyhow::anyhow!("Invalid Sui address"))
        }
    }
}

impl ToString for SuiAddress {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}
