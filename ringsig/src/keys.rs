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

use crate::armor::FromArmor;
use curve25519_dalek::{
    constants,
    edwards::{CompressedEdwardsY, EdwardsPoint},
    scalar::Scalar,
};

/// Key-related error
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    /// Tried to parse a key with apparently no data
    EmptyKey,
    /// Key had a keytype but no key (or more likely, a key but no type)
    NoKey,
    /// The key had an incorrect type
    WrongKeyType {
        expected: String,
        got: String,
    },
    /// The key had an incorrect length
    WrongKeyLength {
        expected: usize,
        got: usize,
    },
    /// Key was not in the prime order group
    TorsionKey(Vec<u8>),
    /// Key did not parse as a public key (e.g. point not on curve)
    InvalidKey(Vec<u8>),
    /// Radix-64 parsing
    Radix64(crate::radix64::Error),
    /// ASCII-armor related error (stringified to avoid a loop in the error types
    Armor(String),
}

impl From<crate::radix64::Error> for Error {
    fn from(e: crate::radix64::Error) -> Self {
        Error::Radix64(e)
    }
}

/// A public key
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct PublicKey(pub(crate) EdwardsPoint);

impl PublicKey {
    /// Serialize the public key as a 32-byte point
    pub fn serialize(&self) -> [u8; 32] {
        self.0.compress().to_bytes()
    }

    /// Parse a public key from 32 bytes
    pub fn parse(data: &[u8]) -> Result<Self, Error> {
        if data.len() != 32 {
            return Err(Error::WrongKeyLength { expected: 32, got: data.len() });
        }
        match CompressedEdwardsY::from_slice(data).decompress() {
            Some(pt) => {
                if pt.is_torsion_free() {
                    Ok(PublicKey(pt))
                } else {
                    Err(Error::TorsionKey(data.to_vec()))
                }
            }
            None => Err(Error::InvalidKey(data.to_vec())),
        }
    }

    /// Parse a public key from the "id_ed25519.pub" format
    pub fn parse_pk_line(data: &str) -> Result<Self, Error> {
        let pieces: Vec<_> = data
            .split(|c: char| c.is_ascii_whitespace())
            .filter(|frag| !frag.is_empty())
            .collect();
        if pieces.is_empty() {
            return Err(Error::EmptyKey);
        }
        // There are several allowable prefixes, all of which have ed25519 in them, according to the ssh source
        if !pieces[0].contains("ssh-ed25519") {
            return Err(Error::WrongKeyType { expected: "ssh-ed25519".to_string(), got: pieces[0].to_string() });
        }
        if pieces.len() < 2 {
            return Err(Error::NoKey);
        }
        match PublicKey::from_armor(&pieces[1]) {
            Ok(pk) => Ok(pk),
            Err(crate::armor::Error::Key(err)) => Err(err),
            Err(other) => Err(Error::Armor(format!("{:?}", other))), // FIXME do not use debug output here
        }
    }
}

/// A secret key
#[derive(Copy, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub struct SecretKey(pub(crate) Scalar);

impl SecretKey {
    /// Construct a secret key from raw bytes
    pub fn from_bytes(data: [u8; 32]) -> Self {
        SecretKey(Scalar::from_bits(data))
    }

    /// Output bytes
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Convert to a public key
    pub fn to_public(&self) -> PublicKey {
        PublicKey(&self.0 * &constants::ED25519_BASEPOINT_TABLE)
    }
}
