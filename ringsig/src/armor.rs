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

use crate::keys::{PublicKey, SecretKey};
use crate::radix64::radix64_decode;
use bitcoin_hashes::{sha512, Hash};
use curve25519_dalek::scalar::Scalar;

/// ASCII armor parsing error
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    /// The begin string did not appear
    NoBeginStr,
    /// The end string did not appear
    NoEndStr,
    /// The end string occurred before the beginning string
    EndBeforeBegin { start_idx: usize, end_idx: usize },
    /// Early end of data when parsing
    EarlyEof,
    /// Expected some target data, read something else
    UnexpectedData { expected: Vec<u8>, got: Vec<u8> },
    /// Expected some target number, read something else
    UnexpectedNumber { expected: usize, got: usize },
    /// The pubilc key in the key packet didn't match the secret key
    PrivPubMismatch {
        encoded_public: PublicKey,
        from_private: PublicKey,
    },
    /// Pubkey parsing
    Key(crate::keys::Error),
    /// Radix-64 parsing
    Radix64(crate::radix64::Error),
}

impl From<crate::radix64::Error> for Error {
    fn from(e: crate::radix64::Error) -> Self {
        Error::Radix64(e)
    }
}

impl From<crate::keys::Error> for Error {
    fn from(e: crate::keys::Error) -> Self {
        Error::Key(e)
    }
}

/// Trait describing types that can be parsed from ASCII armor
pub trait FromArmor: Sized {
    /// The "-----BEGIN THING-----" string
    const BEGIN_STR: &'static str;
    /// The "-----END THING-----" string
    const END_STR: &'static str;

    /// Attempt to parse an object from ASCII armor
    fn from_armor(s: &str) -> Result<Self, Error>;
}

/// Helper function that converts from armor into a bytestring
pub fn parse_armor<T: FromArmor>(s: &str) -> Result<Vec<u8>, Error> {
    let start_idx = s.find(T::BEGIN_STR).ok_or(Error::NoBeginStr)?;
    let end_idx = s.find(T::END_STR).ok_or(Error::NoEndStr)?;
    if end_idx < start_idx {
        return Err(Error::EndBeforeBegin { start_idx, end_idx });
    }
    radix64_decode(&s[start_idx + T::BEGIN_STR.len()..end_idx]).map_err(From::from)
}

/// Helper to read a 32-bit big-endian number as a usize.
///
/// Will panic on 16-bit systems I guess
fn read_length(sl: &mut &[u8]) -> Result<usize, Error> {
    if sl.len() < 4 {
        return Err(Error::EarlyEof);
    }
    let ret = u32::from_be_bytes(<[u8; 4]>::try_from(&sl[0..4]).unwrap()); // no panic
    *sl = &sl[4..];
    Ok(ret.try_into().unwrap()) // panic on 16-bit systems
}

/// Helper to read a fixed string from a slice and match against a target
fn check_string_no_prefix(sl: &mut &[u8], target: &[u8]) -> Result<(), Error> {
    if sl.len() < target.len() {
        return Err(Error::EarlyEof);
    }
    if &sl[..target.len()] != target {
        return Err(Error::UnexpectedData {
            expected: target.to_vec(),
            got: sl[..target.len()].to_vec(),
        });
    }
    *sl = &sl[target.len()..];
    Ok(())
}

/// Helper to read a fixed string from a slice and match against a target
fn check_string(sl: &mut &[u8], target: &[u8]) -> Result<(), Error> {
    let len = read_length(sl)?;
    if sl.len() < len {
        return Err(Error::EarlyEof);
    }
    if &sl[..len] != target {
        return Err(Error::UnexpectedData {
            expected: target.to_vec(),
            got: sl[..len].to_vec(),
        });
    }
    *sl = &sl[len..];
    Ok(())
}

fn check_string_has_ed(sl: &mut &[u8]) -> Result<(), Error> {
    // There are several allowable prefixes, all of which have ed25519 in them, according to the ssh source
    let keytype_len = read_length(sl)?;
    if sl.len() < keytype_len {
        return Err(Error::EarlyEof);
    }
    let has_ed = match std::str::from_utf8(&sl[..keytype_len]) {
        Ok(s) => s.contains("ssh-ed25519"),
        Err(_) => false,
    };
    if !has_ed {
        return Err(Error::UnexpectedData {
            expected: b"ssh-ed25519".to_vec(),
            got: sl[..keytype_len].to_vec(),
        });
    }
    *sl = &sl[keytype_len..];
    Ok(())
}

fn read_string32(sl: &mut &[u8]) -> Result<[u8; 32], Error> {
    let len = read_length(sl)?;
    if len != 32 {
        return Err(Error::UnexpectedNumber {
            expected: 32,
            got: len,
        });
    }
    if sl.len() < 32 {
        return Err(Error::EarlyEof);
    }
    let mut ret = [0; 32];
    ret.copy_from_slice(&sl[..32]);
    *sl = &sl[32..];
    Ok(ret)
}

impl FromArmor for PublicKey {
    const BEGIN_STR: &'static str = "";
    const END_STR: &'static str = "";
    fn from_armor(s: &str) -> Result<Self, Error> {
        let data = radix64_decode(s)?;
        let mut sl = &data[..];
        check_string_has_ed(&mut sl)?; // key type
        let pk = read_string32(&mut sl)?;
        PublicKey::parse(&pk).map_err(From::from)
    }
}

impl FromArmor for SecretKey {
    const BEGIN_STR: &'static str = "-----BEGIN OPENSSH PRIVATE KEY-----";
    const END_STR: &'static str = "-----END OPENSSH PRIVATE KEY-----";

    // Format from https://coolaj86.com/articles/the-openssh-private-key-format/
    fn from_armor(s: &str) -> Result<Self, Error> {
        let data = parse_armor::<Self>(s)?;
        let mut sl = &data[..];
        check_string_no_prefix(&mut sl, b"openssh-key-v1\0")?;
        check_string(&mut sl, b"none")?; // ciphername
        check_string(&mut sl, b"none")?; // kdfname
        check_string(&mut sl, b"")?; // kdf
        check_string_no_prefix(&mut sl, &[0, 0, 0, 1])?; // number of keys, always 1
                                                         // Public key segment
        let total_len = read_length(&mut sl)?;
        check_string_has_ed(&mut sl)?; // key type
        let pubkey_1 = read_string32(&mut sl)?;
        if total_len < 51 {
            // 32 + 2*4 (lengths) + len(ssh-ed25519)
            return Err(Error::UnexpectedNumber {
                expected: 51,
                got: total_len,
            });
        }
        // Private key segment
        read_length(&mut sl)?; // total length, unnecessary
        read_length(&mut sl)?; // unclear what the purpose of this is
        read_length(&mut sl)?; // unclear what the purpose of this is
        check_string_has_ed(&mut sl)?; // key type
        let pubkey_2 = read_string32(&mut sl)?;
        if pubkey_1 != pubkey_2 {
            return Err(Error::UnexpectedData {
                expected: pubkey_1.to_vec(),
                got: pubkey_2.to_vec(),
            });
        }
        let priv_len = read_length(&mut sl)?;
        if priv_len != 64 {
            return Err(Error::UnexpectedNumber {
                expected: 64,
                got: priv_len,
            });
        }
        // First 32 bytes are the actual private key. Last 32 are the public key, again, inexplicably
        if sl.len() < 64 {
            return Err(Error::EarlyEof);
        }
        if pubkey_1 != &sl[32..64] {
            return Err(Error::UnexpectedData {
                expected: pubkey_1.to_vec(),
                got: sl[32..64].to_vec(),
            });
        }

        // DANGER WILL ROBINSON
        // We need to mangle the secret key prior to use because the ed25519 public
        // key is actually derived from the mangled key rather than from the original.
        // This means that these keys are biased and strictly speaking no security
        // argument for AOS (or Schnorr for that matter..) goes through
        let mut extsk = sha512::Hash::hash(&sl[..32]).into_inner();
        extsk[0] &= 0xf8;
        extsk[31] &= 0x7f;
        extsk[31] |= 0x40;
        // end DANGER
        let mut sk = [0; 32];
        sk.copy_from_slice(&extsk[..32]);
        let sk = SecretKey(Scalar::from_bits(sk));
        // extsk[32..64] is used in ed25519 as a "nonce" which provides more entropy
        // for signature nonces. I think this is silly and we won't do it for the
        // ring signatures.

        let pk_encoded = PublicKey::parse(&pubkey_1)?;
        let pk_from_priv = sk.to_public();
        if pk_encoded != pk_from_priv {
            return Err(Error::PrivPubMismatch {
                encoded_public: pk_encoded,
                from_private: pk_from_priv,
            });
        }
        Ok(sk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_fixed_sk() {
        let sk = SecretKey::from_armor(
            "\n\
            -----BEGIN OPENSSH PRIVATE KEY-----\n\
            b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW\n\
            QyNTUxOQAAACA3bZbhkmNL784HHVNxkyH1ra6/CjEpPGNYvTSX0QpFdQAAAJin2/I9p9vy\n\
            PQAAAAtzc2gtZWQyNTUxOQAAACA3bZbhkmNL784HHVNxkyH1ra6/CjEpPGNYvTSX0QpFdQ\n\
            AAAEDl+pu1FRvTBgWPp+7D4F7PVACxPiFLr0MKDZotYW01qDdtluGSY0vvzgcdU3GTIfWt\n\
            rr8KMSk8Y1i9NJfRCkV1AAAAEWFwb2Vsc3RyYUBzdWx0YW5hAQIDBA==\n\
            -----END OPENSSH PRIVATE KEY-----\n\
        ",
        )
        .unwrap();
        assert_eq!(
            sk,
            SecretKey::from_bytes([
                0x60, 0xb0, 0x7c, 0x0a, 0xb3, 0xfc, 0xc3, 0xb0, 0x29, 0x54, 0xd0, 0xee, 0x5c, 0x5b,
                0xdd, 0xe5, 0xa0, 0x7d, 0x1f, 0xd1, 0x4e, 0xf4, 0x29, 0x5f, 0xfe, 0x13, 0xec, 0x00,
                0xdd, 0xc4, 0xa8, 0x5c,
            ])
        );
    }
}
