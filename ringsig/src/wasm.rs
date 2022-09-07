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

use bitcoin_hashes::hex::{FromHex, ToHex};
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

pub fn verify_internal(
    proof: &str,
    pks: &[String],
    msg: &str,
) -> Result<(), String> {
    let pks = pks
        .iter()
        .map(|key| PublicKey::parse_pk_line(key))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("{:?}", e))?; // FIXME don't use debug

    let proof = Vec::<u8>::from_hex(proof)
        .map_err(|e| e.to_string())?;

    crate::verify(&proof, &pks, msg.as_bytes())
        .map_err(|e| e.to_owned())
}

/// Verifies a proof. Returns an error string. If the proof is good, returns the empty string.
#[wasm_bindgen]
pub fn verify(
    proof: &str,
    pks: js_sys::Array,
    msg: &str,
) -> String {
    let pks_rust: Vec<String> = pks
        .iter()
        .map(|v| v.as_string().unwrap_or("js unknown".to_owned()))
        .collect();

    match verify_internal(proof, &pks_rust, msg) {
        Ok(()) => "".to_owned(),
        Err(e) => e,
    }
}

#[wasm_bindgen]
pub fn is_secret_key(data: &str) -> bool {
    SecretKey::from_armor(data).is_ok()
}

#[wasm_bindgen]
pub fn is_proof(data: &str) -> bool {
    data.len() % 32 == 0 && Vec::<u8>::from_hex(data).is_ok()
}

#[wasm_bindgen]
pub fn is_acceptable_pubkey(data: &str) -> bool {
   // checks that the key is parseable *and* that it's in the prime group
   PublicKey::parse_pk_line(data).is_ok()
}

