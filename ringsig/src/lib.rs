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

pub mod armor;
pub mod hashes;
pub mod keys;
pub mod radix64;
pub mod wasm;

use bitcoin_hashes::{Hash, HashEngine};
use curve25519_dalek::{constants, edwards::EdwardsPoint, scalar::Scalar};

use crate::hashes::{ChallengeHash, NonceHash, ParamsHash};
use crate::keys::{PublicKey, SecretKey};

fn param_hash(pks: &[PublicKey], message: &[u8]) -> ParamsHash {
    let mut eng = ParamsHash::engine();
    eng.input(&(u32::try_from(pks.len()).unwrap().to_le_bytes()));
    for pk in pks {
        eng.input(&pk.serialize());
    }
    eng.input(&(u64::try_from(message.len()).unwrap().to_le_bytes()));
    eng.input(message);
    ParamsHash::from_engine(eng)
}

/// Helper function to save typing
fn hash_to_sc<T: Hash<Inner = [u8; 32]>>(inp: T) -> Scalar {
    Scalar::from_bits(inp.into_inner())
}

pub fn verify(proof: &[u8], pks: &[PublicKey], message: &[u8]) -> Result<(), &'static str> {
    if pks.is_empty() {
        return Err("no public keys");
    }

    let mut pks = pks.to_owned();
    pks.sort_by_key(|pk| pk.serialize());
    if proof.len() != 32 * (pks.len() + 1) {
        return Err("proof wrong length");
    }

    let params = param_hash(&pks, message);
    let mut e_i = ChallengeHash::from_slice(&proof[..32]).unwrap();
    for idx in 0..pks.len() {
        let s_i = NonceHash::from_slice(&proof[32 * (idx + 1)..32 * (idx + 2)]).unwrap();
        let pubnonce = EdwardsPoint::vartime_double_scalar_mul_basepoint(&hash_to_sc(e_i), &-pks[idx].0, &hash_to_sc(s_i));

        let mut challenge_eng = ChallengeHash::engine();
        challenge_eng.input(&pubnonce.compress().to_bytes());
        challenge_eng.input(&params[..]);
        e_i = ChallengeHash::from_engine(challenge_eng);
    }
    if &e_i[..] != &proof[..32] {
        return Err("bad proof");
    }
    Ok(())
}

pub fn prove(pks: &[PublicKey], message: &[u8], sk: SecretKey) -> Result<Vec<u8>, &'static str> {
    let mut pks = pks.to_owned();
    pks.sort_by_key(|pk| pk.serialize());
    let params = param_hash(&pks, message);
    let my_pk = sk.to_public();
    let my_idx = match pks.iter().position(|&pk| pk == my_pk) {
        Some(idx) => idx,
        None => return Err("secret key did not match any public key"),
    };
    let mut ret = vec![0; 32 * (pks.len() + 1)];

    let mut nonce_eng = NonceHash::engine();
    nonce_eng.input(&params[..]);
    nonce_eng.input(sk.as_bytes());
    let nonce = NonceHash::from_engine(nonce_eng);

    // Compute all the `s` values for indices greater than our own.
    // Note that this does not actually use any secret data anywhere.
    let mut pubnonce = &hash_to_sc(nonce) * &constants::ED25519_BASEPOINT_TABLE;
    for idx in (my_idx + 1..pks.len()).chain(0..my_idx) {
        // Hash the nonce before the params since the nonce is non-constant (in fact,
        // it is hard for an attacker to control at all). Assuming SHA256 is secure,
        // this accomplishes nothing except preventing the verifier from caching any
        // part of the hash computation. But if SHA2 were to be broken this would
        // plausibly save us.
        let mut challenge_eng = ChallengeHash::engine();
        challenge_eng.input(&pubnonce.compress().to_bytes());
        challenge_eng.input(&params[..]);
        let e_i = ChallengeHash::from_engine(challenge_eng);

        if idx == 0 {
            ret[0..32].copy_from_slice(&e_i[..]);
        }

        // Compute random s value, save it to proof
        let mut s_eng = NonceHash::engine();
        s_eng.input(&(idx as u64).to_be_bytes());
        s_eng.input(&params[..]);
        s_eng.input(sk.as_bytes());
        let s_i = NonceHash::from_engine(s_eng);
        ret[32 * (1 + idx)..32 * (2 + idx)].copy_from_slice(&s_i[..]);
        // Compute next R value as though we were a verifier
        pubnonce = EdwardsPoint::vartime_double_scalar_mul_basepoint(&hash_to_sc(e_i), &-pks[idx].0, &hash_to_sc(s_i));
    }
    // Now, we have filled in every s value except that at our own index. This one
    // we have to compute rather than randomly generating
    let mut challenge_eng = ChallengeHash::engine();
    challenge_eng.input(&pubnonce.compress().to_bytes());
    challenge_eng.input(&params[..]);
    let e_i = ChallengeHash::from_engine(challenge_eng);
    let s_i = &hash_to_sc(nonce) + (&hash_to_sc(e_i) * &sk.0);
    ret[32 * (1 + my_idx)..32 * (2 + my_idx)].copy_from_slice(s_i.as_bytes());
    if my_idx == 0 {
        ret[0..32].copy_from_slice(&e_i[..]);
    }

    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_key_proof() {
        let sk1 = SecretKey::from_bytes([
            0x60, 0xb0, 0x7c, 0x0a, 0xb3, 0xfc, 0xc3, 0xb0, 0x29, 0x54, 0xd0, 0xee, 0x5c, 0x5b,
            0xdd, 0xe5, 0xa0, 0x7d, 0x1f, 0xd1, 0x4e, 0xf4, 0x29, 0x5f, 0xfe, 0x13, 0xec, 0x00,
            0xdd, 0xc4, 0xa8, 0x5c,
        ]);

        let pk = sk1.to_public();
        let proof = prove(&[pk], b"Hello, world!", sk1).unwrap();
        verify(&proof, &[pk], b"Hello, world!").unwrap();
        assert!(verify(&proof, &[pk], b"Goodbye, world!").is_err());
    }

    #[test]
    fn empty_proof() {
        let proof = b"32 bytes32 bytes32 bytes32 bytes";
        assert!(verify(&proof[..], &[], b"Goodbye, world!").is_err());
    }

    #[test]
    fn multi_key_proof() {
        let key_str = [
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIKHQ634LrVRQ0bLDLZ5kdjcpmihQBtcJbGoMqCJh6i10", // apoelstra on github
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGMiyoNWxKsdbuZ9EeJA+QTTaKHYtpCrRBlvCez8ykRl", // davidvorick on github
            "ssh-ed25519	AAAAC3NzaC1lZDI1NTE5AAAAIDgiq1etF0aD94rG/UVmYEt4ij5K8MvHZwb4wIUi6Ihr", // also davidvorick on github
            "  ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHptEpqs57lhnHkfa+0SQgXQ4A63/YGV2cNTcGMQW+Jt", // also davidvorick on github
            "ssh-ed25519    AAAAC3NzaC1lZDI1NTE5AAAAICUrHXT71TxmXQA5jDLjPF8QsZ4txhRffAu9SG/dNt8+", // also davidvorick on github
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDdtluGSY0vvzgcdU3GTIfWtrr8KMSk8Y1i9NJfRCkV1 apoelstra@sultana", // generated locally
        ];
        // Matches last key; see armor.rs for unit test for the armored representation
        let sk = SecretKey::from_bytes([
            0x60, 0xb0, 0x7c, 0x0a, 0xb3, 0xfc, 0xc3, 0xb0, 0x29, 0x54, 0xd0, 0xee, 0x5c, 0x5b,
            0xdd, 0xe5, 0xa0, 0x7d, 0x1f, 0xd1, 0x4e, 0xf4, 0x29, 0x5f, 0xfe, 0x13, 0xec, 0x00,
            0xdd, 0xc4, 0xa8, 0x5c,
        ]);

        let mut keys = key_str.iter().map(|key| PublicKey::parse_pk_line(key).unwrap()).collect::<Vec<_>>();
        assert!(prove(&keys[..keys.len() - 1], b"Hello, world!", sk).is_err()); // my key not present
        let proof = prove(&keys, b"Hello, world!", sk).unwrap();
        verify(&proof, &keys, b"Hello, world!").unwrap();

        assert!(verify(&proof, &keys, b"Goodbye, world!").is_err()); // wrong message
        assert!(verify(&proof[..keys.len() - 1], &keys, b"Hello, world!").is_err()); // not enough keys
        // Key ordering does not matter
        keys.swap(0, 1);
        verify(&proof, &keys, b"Hello, world!").unwrap();
    }

    #[test]
    fn torsion_key() {
        assert!(PublicKey::parse_pk_line(
                "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAII0PQoSjaDulROj7qwNNsJ1cCa+sqlWsKs3e8nemW9J+ apoelstra-torsion"
        ).is_err()); // FIXME check that we get specifically the torsion error
    }
}


