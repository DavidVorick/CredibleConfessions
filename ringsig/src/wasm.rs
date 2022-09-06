// Crypto Confessions
// Written in 2022 by
//   Andrew Poelstra <cryptoconfessions@wpsoftware.net>
//   or David Vorick <cryptoconfessions@wpsoftware.net>
//   or Liam Eagen <cryptoconfessions@wpsoftware.net>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

use wasm_bindgen::prelude::*;

use bitcoin_hashes::hex::ToHex;
use crate::armor::FromArmor;
use crate::keys::{PublicKey, SecretKey};

pub fn prove_internal(
    pks: &[String],
    msg: &str,
    sk: &str,
) -> Result<String, String> {

    let pks = pks
        .iter()
        .map(|key| PublicKey::parse_pk_line(key))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("{:?}", e))?; // FIXME don't use debug

    let sk = SecretKey::from_armor(sk)
        .map_err(|e| format!("{:?}", e))?; // FIXME don't use debug

    match crate::prove(&pks, msg.as_bytes(), sk) {
        Ok(proof) => Ok(proof.to_hex()),
        Err(e) => Err(format!("{:?}", e)), // FIXME don't use debug
    }
}


#[wasm_bindgen]
pub fn prove(
    pks: js_sys::Array,
    msg: &str,
    sk: &str,
) -> js_sys::Array {
    let pks_rust: Vec<String> = pks
        .iter()
        .map(|v| v.as_string().unwrap_or("js unknown".to_owned()))
        .collect();
    let ret = js_sys::Array::new(); // FIXME this doesn't need to be mutable?? how does push work??
    match prove_internal(&pks_rust, msg, sk) {
        Ok(s) => {
            ret.push(&JsValue::from_str(&s));
            ret.push(&JsValue::from_str(""));
        },
        Err(e) => {
            ret.push(&JsValue::from_str(""));
            ret.push(&JsValue::from_str(&e));
        },
    }
    ret
}

