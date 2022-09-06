// Deterministic RSA Prime Generation
// Written in 2020 by
//   Andrew Poelstra <apoelstra@wpsoftware.net>
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

//! Radix-64 CRC
//!
//! Computes the CRC as specified in RFC 4880 Section 6. Basically a
//! transliteration of the C code in 6.1 to Rust

/// Radix-64 parsing error
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    /// String ended early
    EarlyEof,
    /// String was not entirely ASCII
    NonAsciiString(String),
    /// Some character was not in the radix64 alphabet.
    NonRadix64Character(u8),
    /// A character occurred after an = sign
    ExtraData(u8),
}

const BASE64_CH: [u8; 64] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Base-64 encodes data
pub fn base64_encode(mut data: &[u8]) -> String {
    let mut ret = Vec::with_capacity((data.len() * 4 + 2) / 3);

    loop {
        enum Npad {
            Zero,
            One,
            Two,
        }
        let (three, npad) = match data.len() {
            0 => return String::from_utf8(ret).unwrap(),
            1 => ([data[0], 0, 0], Npad::Two),
            2 => ([data[0], data[1], 0], Npad::One),
            _ => ([data[0], data[1], data[2]], Npad::Zero),
        };

        let sext = [
            three[0] >> 2,
            ((three[0] & 0x03) << 4) + (three[1] >> 4),
            ((three[1] & 0x0f) << 2) + (three[2] >> 6),
            three[2] & 0x3f,
        ];

        ret.push(BASE64_CH[sext[0] as usize]);
        match npad {
            Npad::Zero => {
                ret.push(BASE64_CH[sext[1] as usize]);
                ret.push(BASE64_CH[sext[2] as usize]);
                ret.push(BASE64_CH[sext[3] as usize]);
                data = &data[3..];
            }
            Npad::One => {
                ret.push(BASE64_CH[sext[1] as usize]);
                ret.push(BASE64_CH[sext[2] as usize]);
                ret.push(b'=');
                data = &data[2..];
            }
            Npad::Two => {
                ret.push(BASE64_CH[sext[1] as usize]);
                ret.push(b'=');
                ret.push(b'=');
                data = &data[1..];
            }
        }
        if ret.len() % 77 == 76 {
            ret.push(b'\n');
        }
    }
}

/// Computes the CRC and outputs it as a base64 string
pub fn crc24_bytes(data: &[u8]) -> [u8; 3] {
    const CRC24_INIT: u32 = 0x00B7_04CE;
    const CRC24_POLY: u32 = 0x0186_4CFB;

    let mut crc = CRC24_INIT;
    for byte in data {
        crc ^= (*byte as u32) << 16;
        for _ in 0..8 {
            crc <<= 1;
            if crc & 0x0100_0000 != 0 {
                crc ^= CRC24_POLY;
            }
        }
    }
    [
        ((crc >> 16) & 0xff) as u8,
        ((crc >> 8) & 0xff) as u8,
        ((crc >> 0) & 0xff) as u8,
    ]
}

pub fn crc24_string(data: &[u8]) -> String {
    base64_encode(&crc24_bytes(data)[..])
}

/// Decodes a single character as a 6-bit number
///
/// WARNING: this parses '=' as 0x80 which is otherwise impossible
fn base64_decode_ch(ch: u8) -> Result<u8, Error> {
    const TABLE: [u8; 128] = [
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, // 0-15
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, // 16-31
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x3e, 0xff, 0xff, 0xff,
        0x3f, // 32-47
        0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0xff, 0xff, 0xff, 0x80, 0xff,
        0xff, // 48-63
        0xff, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
        0x0e, // 64-79
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0xff, 0xff, 0xff, 0xff,
        0xff, // 80-95
        0xff, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27,
        0x28, // 96-111
        0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0xff, 0xff, 0xff, 0xff,
        0xff, // 112-127
    ];
    if ch > 128 || TABLE[ch as usize] == 0xff {
        return Err(Error::NonRadix64Character(ch));
    }
    Ok(TABLE[ch as usize])
}

/// Decode radix64-encoded  data
pub fn radix64_decode(s: &str) -> Result<Vec<u8>, Error> {
    if !s.is_ascii() {
        return Err(Error::NonAsciiString(s.to_owned()));
    }
    let mut ret = Vec::with_capacity((s.len() * 3 + 3) / 4);

    let mut iter = s.bytes().filter(|b| !b.is_ascii_whitespace());
    loop {
        let quad = [
            base64_decode_ch(match iter.next() {
                Some(b) => b,
                None => break,
            })?,
            base64_decode_ch(iter.next().ok_or(Error::EarlyEof)?)?,
            base64_decode_ch(iter.next().ok_or(Error::EarlyEof)?)?,
            base64_decode_ch(iter.next().ok_or(Error::EarlyEof)?)?,
        ];
        let skip;
        match (quad[2] == 0x80, quad[3] == 0x80) {
            // 0x80 means "=", see base64_decode_ch
            (false, false) => skip = 0,
            (false, true) => skip = 1,
            (true, true) => skip = 2,
            (true, false) => return Err(Error::ExtraData(quad[3])),
        }
        ret.push((quad[0] << 2) + (quad[1] >> 4));
        if skip < 2 {
            ret.push((quad[1] << 4) + (quad[2] >> 2));
        }
        if skip < 1 {
            ret.push((quad[2] << 6) + quad[3]);
        }
        if skip > 0 {
            if let Some(bad) = iter.next() {
                return Err(Error::ExtraData(bad));
            } else {
                break;
            }
        }
    }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"x"), "eA==");
        assert_eq!(
            base64_encode(b"this is a test sentence"),
            "dGhpcyBpcyBhIHRlc3Qgc2VudGVuY2U=",
        );
    }

    #[test]
    fn radix64() {
        // From RFC 4880 6.6
        let data = [
            0xc8, 0x38, 0x01, 0x3b, 0x6d, 0x96, 0xc4, 0x11, 0xef, 0xec, 0xef, 0x17, 0xec, 0xef,
            0xe3, 0xca, 0x00, 0x04, 0xce, 0x89, 0x79, 0xea, 0x25, 0x0a, 0x89, 0x79, 0x95, 0xf9,
            0x79, 0xa9, 0x0a, 0xd9, 0xa9, 0xa9, 0x05, 0x0a, 0x89, 0x0a, 0xc5, 0xa9, 0xc9, 0x45,
            0xa9, 0x40, 0xc1, 0xa2, 0xfc, 0xd2, 0xbc, 0x14, 0x85, 0x8c, 0xd4, 0xa2, 0x54, 0x7b,
            0x2e, 0x00,
        ];
        assert_eq!(
            base64_encode(&data[..]),
            "yDgBO22WxBHv7O8X7O/jygAEzol56iUKiXmV+XmpCtmpqQUKiQrFqclFqU\
             DBovzSvBSFjNSiVHsu\nAA==",
        );
        assert_eq!(crc24_string(&data[..]), "njUN");
        assert_eq!(
            Ok(data.to_vec()),
            radix64_decode(
                "yDgBO22WxBHv7O8X7O/jygAEzol56iUKiXmV+XmpCtmpqQUKiQrFqclFqU\
                 DBovzSvBSFjNSiVHsu\nAA==",
            )
        );
    }
}
