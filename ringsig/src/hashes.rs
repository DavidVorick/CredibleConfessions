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

use bitcoin_hashes::sha256t_hash_newtype;

const MIDSTATE_CHALLENGE_HASH: [u8; 32] = [
    0xe8, 0xb2, 0x2d, 0x66, 0xaf, 0x38, 0xce, 0x01, 0xa6, 0x7e, 0x49, 0x04, 0xec, 0x70, 0x25, 0xac,
    0xac, 0x86, 0xbd, 0x85, 0x7d, 0xbc, 0xd4, 0x4f, 0x9b, 0x4f, 0x1b, 0xb8, 0xa1, 0x82, 0x60, 0xfb,
];

const MIDSTATE_PARAMS_HASH: [u8; 32] = [
    0x7f, 0xca, 0x3f, 0x68, 0xe5, 0x18, 0x29, 0x68, 0xa5, 0xb5, 0x7a, 0xfe, 0x0e, 0x65, 0x31, 0x8b,
    0xb2, 0x7d, 0x45, 0x40, 0x85, 0xa9, 0x4c, 0xf3, 0x88, 0x23, 0x38, 0x63, 0x9a, 0x22, 0x63, 0x22,
];

const MIDSTATE_NONCE_HASH: [u8; 32] = [
    0xc3, 0x05, 0x00, 0xed, 0xc2, 0x35, 0xd1, 0x1f, 0x44, 0x90, 0x0b, 0xc5, 0x49, 0x53, 0x76, 0x7a,
    0x6c, 0x46, 0x3b, 0xd2, 0xf2, 0xc0, 0xec, 0x08, 0x4e, 0x2d, 0xda, 0x6d, 0x81, 0xf0, 0xbd, 0xcc,
];

sha256t_hash_newtype!(
    ChallengeHash,
    ChallengeHashTag,
    MIDSTATE_CHALLENGE_HASH,
    64,
    doc = "BIP-340 tagged hash for Crypto Confessions ringsig challenge hashes",
    false // whether to reverse the hash when serializing
);

sha256t_hash_newtype!(
    ParamsHash,
    ParamsHashTag,
    MIDSTATE_PARAMS_HASH,
    64,
    doc = "BIP-340 tagged hash for Crypto Confessions ringsig param hash (pks and message)",
    false // whether to reverse the hash when serializing
);

sha256t_hash_newtype!(
    NonceHash,
    NonceHashTag,
    MIDSTATE_NONCE_HASH,
    64,
    doc = "BIP-340 tagged hash for Crypto Confessions ringsig param hash (generating nonce and other secret data)",
    false // whether to reverse the hash when serializing
);

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin_hashes::{hex::ToHex, sha256, Hash, HashEngine};

    // Lifted from rust-bitcoin
    fn tag_engine(tag_name: &str) -> sha256::HashEngine {
        let mut engine = sha256::Hash::engine();
        let tag_hash = sha256::Hash::hash(tag_name.as_bytes());
        engine.input(&tag_hash[..]);
        engine.input(&tag_hash[..]);
        engine
    }

    #[test]
    fn tagged_hashes() {
        assert_eq!(
            MIDSTATE_CHALLENGE_HASH[..].to_hex(),
            tag_engine("CryptoConfessions-1.0/Challenge")
                .midstate()
                .into_inner()[..]
                .to_hex(),
        );

        assert_eq!(
            MIDSTATE_PARAMS_HASH[..].to_hex(),
            tag_engine("CryptoConfessions-1.0/Params")
                .midstate()
                .into_inner()[..]
                .to_hex(),
        );

        assert_eq!(
            MIDSTATE_NONCE_HASH[..].to_hex(),
            tag_engine("CryptoConfessions-1.0/Nonce")
                .midstate()
                .into_inner()[..]
                .to_hex(),
        );
    }
}
