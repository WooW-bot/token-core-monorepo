#![feature(more_qualified_paths)]
#![feature(test)]

use std::ffi::{CStr, CString};

use std::os::raw::c_char;

use anyhow::anyhow;
use handler::{backup, sign_bls_to_execution_change};
use log::{debug, error, info, LevelFilter};
use migration::{mark_identity_wallets, read_legacy_keystore_mnemonic_path};
use prost::Message;

#[cfg(feature = "android")]
use android_logger::{Config, FilterBuilder};

pub mod api;

use crate::api::{GeneralResult, TcxAction};

pub mod error_handling;
pub mod handler;
pub mod migration;
pub mod reset_password;
use anyhow::Error;
use std::result;

use crate::error_handling::{landingpad, LAST_ERROR};
use crate::handler::{
    create_keystore, decrypt_data_from_ipfs, delete_keystore, derive_accounts, derive_sub_accounts,
    encode_message, encrypt_data_to_ipfs, eth_batch_personal_sign, exists_json, exists_mnemonic,
    exists_private_key, export_json, export_mnemonic, export_private_key, get_derived_key,
    get_extended_public_keys, get_public_keys, import_json, import_mnemonic, import_private_key,
    mnemonic_to_public, scan_keystores, sign_authentication_message, sign_hashes, sign_message,
    sign_psbt, sign_psbts, sign_tx, unlock_then_crash, verify_password,
};
use crate::migration::{migrate_keystore, scan_legacy_keystores};

pub mod filemanager;
// mod identity;
mod macros;

use parking_lot::RwLock;
use tcx_common::{FromHex, ToHex};

extern crate serde_json;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref IS_DEBUG: RwLock<bool> = RwLock::new(false);
}

pub type Result<T> = result::Result<T, Error>;

#[no_mangle]
pub unsafe extern "C" fn init_logger() {
    #[cfg(feature = "android")]
    {
        let config = Config::default()
            .with_tag("tcx")
            .with_max_level(LevelFilter::Debug);
        android_logger::init_once(config);
        debug!("Android日志系统初始化完成");
    }

    #[cfg(not(feature = "android"))]
    {
        log::set_max_level(LevelFilter::Debug);
        debug!("标准日志系统初始化完成");
    }
}

/// # Safety
///
#[no_mangle]
pub unsafe extern "C" fn free_const_string(s: *const c_char) {
    if s.is_null() {
        return;
    }
    let _ = CStr::from_ptr(s);
}

/// # Safety
///
/// dispatch protobuf rpc call
#[allow(deprecated)]
#[no_mangle]
pub unsafe extern "C" fn call_tcx_api(hex_str: *const c_char) -> *const c_char {
    debug!("开始处理API调用");
    if hex_str.is_null() {
        error!("API调用参数为空");
        return CString::new("").unwrap().into_raw();
    }

    let hex_c_str = match CStr::from_ptr(hex_str).to_str() {
        Ok(s) => s,
        Err(e) => {
            error!("解析API参数失败: {}", e);
            return CString::new("").unwrap().into_raw();
        }
    };

    let data = match Vec::from_hex(hex_c_str) {
        Ok(d) => d,
        Err(e) => {
            error!("解码十六进制数据失败: {}", e);
            return CString::new("").unwrap().into_raw();
        }
    };

    let action: TcxAction = match TcxAction::decode(data.as_slice()) {
        Ok(a) => a,
        Err(e) => {
            error!("解码API消息失败: {}", e);
            return CString::new("").unwrap().into_raw();
        }
    };

    debug!("API方法: {}", action.method);

    let reply: Result<Vec<u8>> = match action.method.to_lowercase().as_str() {
        "init_token_core_x" => landingpad(|| {
            debug!("初始化TokenCore");
            handler::init_token_core_x(&action.param.unwrap().value).unwrap();
            Ok(vec![])
        }),
        "scan_legacy_keystores" => landingpad(|| {
            debug!("扫描旧版钱包");
            let ret = scan_legacy_keystores()?;
            encode_message(ret)
        }),
        "scan_keystores" => landingpad(|| {
            debug!("扫描钱包");
            let ret = scan_keystores()?;
            encode_message(ret)
        }),
        "read_keystore_mnemonic_path" => {
            landingpad(|| read_legacy_keystore_mnemonic_path(&action.param.unwrap().value))
        }
        "create_keystore" => landingpad(|| create_keystore(&action.param.unwrap().value)),
        "import_mnemonic" => landingpad(|| import_mnemonic(&action.param.unwrap().value)),
        "export_mnemonic" => landingpad(|| export_mnemonic(&action.param.unwrap().value)),
        "derive_accounts" => landingpad(|| derive_accounts(&action.param.unwrap().value)),
        "import_private_key" => landingpad(|| import_private_key(&action.param.unwrap().value)),
        "export_private_key" => landingpad(|| export_private_key(&action.param.unwrap().value)),
        "verify_password" => landingpad(|| verify_password(&action.param.unwrap().value)),
        "delete_keystore" => landingpad(|| delete_keystore(&action.param.unwrap().value)),
        "exists_mnemonic" => landingpad(|| exists_mnemonic(&action.param.unwrap().value)),
        "exists_private_key" => landingpad(|| exists_private_key(&action.param.unwrap().value)),
        "derive_sub_accounts" => landingpad(|| derive_sub_accounts(&action.param.unwrap().value)),
        "sign_tx" => landingpad(|| {
            debug!("开始交易签名");
            let result = sign_tx(&action.param.unwrap().value);
            match &result {
                Ok(_) => debug!("交易签名成功"),
                Err(e) => error!("交易签名失败: {:?}", e),
            }
            result
        }),
        "sign_msg" => landingpad(|| sign_message(&action.param.unwrap().value)),
        "exists_json" => landingpad(|| exists_json(&action.param.unwrap().value)),
        "import_json" => landingpad(|| import_json(&action.param.unwrap().value)),
        "export_json" => landingpad(|| export_json(&action.param.unwrap().value)),
        "backup" => landingpad(|| backup(&action.param.unwrap().value)),

        // !!! WARNING !!! used for `cache_dk` feature
        "get_derived_key" => landingpad(|| get_derived_key(&action.param.unwrap().value)),
        // !!! WARNING !!! used for test only
        "unlock_then_crash" => landingpad(|| unlock_then_crash(&action.param.unwrap().value)),

        "encrypt_data_to_ipfs" => landingpad(|| encrypt_data_to_ipfs(&action.param.unwrap().value)),
        "decrypt_data_from_ipfs" => {
            landingpad(|| decrypt_data_from_ipfs(&action.param.unwrap().value))
        }
        "sign_authentication_message" => {
            landingpad(|| sign_authentication_message(&action.param.unwrap().value))
        }
        "migrate_keystore" => landingpad(|| migrate_keystore(&action.param.unwrap().value)),

        "get_extended_public_keys" => {
            landingpad(|| get_extended_public_keys(&action.param.unwrap().value))
        }
        "get_public_keys" => landingpad(|| get_public_keys(&action.param.unwrap().value)),
        "sign_hashes" => landingpad(|| sign_hashes(&action.param.unwrap().value)),
        "mnemonic_to_public" => landingpad(|| mnemonic_to_public(&action.param.unwrap().value)),
        "sign_bls_to_execution_change" => {
            landingpad(|| sign_bls_to_execution_change(&action.param.unwrap().value))
        }
        "eth_batch_personal_sign" => {
            landingpad(|| eth_batch_personal_sign(&action.param.unwrap().value))
        }
        "mark_identity_wallets" => {
            landingpad(|| mark_identity_wallets(&action.param.unwrap().value))
        }
        "sign_psbt" => landingpad(|| sign_psbt(&action.param.unwrap().value)),
        "sign_psbts" => landingpad(|| sign_psbts(&action.param.unwrap().value)),
        _ => {
            error!("不支持的方法: {}", action.method);
            landingpad(|| Err(anyhow!("unsupported_method")))
        }
    };

    match reply {
        Ok(reply) => {
            debug!("API调用成功");
            let ret_str = reply.to_hex();
            CString::new(ret_str).unwrap().into_raw()
        }
        Err(e) => {
            error!("API调用失败: {:?}", e);
            CString::new("").unwrap().into_raw()
        }
    }
}

/// # Safety
///
#[no_mangle]
pub unsafe extern "C" fn clear_err() {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = None;
    });
}

/// # Safety
///
#[no_mangle]
pub unsafe extern "C" fn get_last_err_message() -> *const c_char {
    LAST_ERROR.with(|e| {
        if let Some(ref err) = *e.borrow() {
            let rsp = GeneralResult {
                is_success: false,
                error: err.to_string(),
            };
            let rsp_bytes = encode_message(rsp).expect("encode error");
            let ret_str = rsp_bytes.to_hex();
            CString::new(ret_str).unwrap().into_raw()
        } else {
            CString::new("").unwrap().into_raw()
        }
    })
}
