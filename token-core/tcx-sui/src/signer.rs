use crate::address::{DEFAULT_HASH_SIZE, ED25519_FLAG, SECP256K1_FLAG};
use crate::primitives::SuiUnsignedMessage;
use crate::transaction::{sui_tx_input::SuiTxType, SuiTxInput, SuiTxOuput};
use crate::Error;
use sha2::{Digest, Sha256};
use tcx_primitive::TypedPrivateKey;

use tcx_keystore::{Keystore, Result, SignatureParameters, Signer, TransactionSigner};

impl TransactionSigner<SuiTxInput, SuiTxOuput> for Keystore {
    fn sign_transaction(
        &mut self,
        sign_context: &SignatureParameters,
        tx: &SuiTxInput,
    ) -> Result<SuiTxOuput> {
        const DEFAULT_HASH_SIZE: usize = 32;
        // 1. 转换为 SuiUnsignedMessage
        let unsigned_tx = SuiUnsignedMessage::try_from(tx)?;
        // 2. BCS 序列化
        let msg_to_sign = bcs::to_bytes(&unsigned_tx).map_err(|_| Error::BcsSerializeFailed)?;
        // 3. Blake2b 哈希
        let mut hash = [0u8; 32];
        let mut hasher = blake2b_rs::Blake2bBuilder::new(DEFAULT_HASH_SIZE).build();
        hasher.update(&msg_to_sign);
        hasher.finalize(&mut hash);

        // 4. 调用签名方法
        let signature = self.sui_sign(&hash, &sign_context.derivation_path)?;

        // 5. 处理交易数据
        let tx_data = match &tx.sui_tx_type.as_ref().ok_or(crate::Error::EmptyTxType)? {
            SuiTxType::RawTx(tx) => tx.tx_data.clone(),
            SuiTxType::Transfer(_) => base64::encode(
                &bcs::to_bytes(&unsigned_tx.value).map_err(|_| Error::BcsSerializeFailed)?,
            ),
        };

        Ok(SuiTxOuput {
            tx_data,
            signature: base64::encode(&signature),
        })
    }
}
