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

use bitcoin_hashes::hex::{FromHex, ToHex};
use home::home_dir;
use ringsig::armor::FromArmor;
use ringsig::keys::{PublicKey, SecretKey};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{env, fs, io};
use std::io::Read;

#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
struct FileContents {
    version: usize,
    #[serde(rename = "publicKeys")]
    pks: Vec<String>,
    message: String,
    proof: Option<String>,
}

fn usage() -> Result<(), &'static str> {
    let name = env::args().next().unwrap();
    eprintln!("Usage: {} prove <json file> [secret key file]", name);
    eprintln!("Usage: {} verify <json file> <proof>", name);
    eprintln!("");
    eprintln!("Here <json file> is a text file containing a JSON object with the");
    eprintln!("fields `publicKeys`, `message`, and (for verification) `proof`. If");
    eprintln!("the filename provided is `-` then standard input will be used.");
    eprintln!("");
    eprintln!("If <secret key file> is provided this will be used as the signing key.");
    eprintln!("Otherwise, when proving, the tool will just try to use every file in");
    eprintln!("~/.ssh as a key.");
    Err("invalid-command-line-args")
}

fn main() -> Result<(), String> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        usage()?;
    }
    match &args[1][..] {
        "prove" if args.len() == 3 || args.len() == 4 => {},
        "verify" if args.len() == 3 => {},
        _ => usage()?,
    }

    // Parse JSON
    let file: Box<dyn Read> = if args[2] == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(fs::File::open(&args[2]).map_err(|e| e.to_string())?)
    };
    let mut contents: FileContents = serde_json::from_reader(file).map_err(|e| e.to_string())?;

    if contents.version != 1 { return Err("JSON version was not 1".into()) }

    let keys = contents
        .pks
        .iter()
        .map(|ln| PublicKey::parse_pk_line(ln))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("parsing public keys: {:?}", e))?; // FIXME

    // Obtain secret key for proving
    if args[1] == "prove" {
        let sk;
        if args.len() == 4 {
            let sk_str = fs::read_to_string(&args[3]).map_err(|e| e.to_string())?;
            sk = SecretKey::from_armor(&sk_str)
                .map_err(|e| format!("Reading secret key file {}: {:?}", args[3], e))?; // FIXME
        } else {
            let mut try_sk = None;
            let mut ssh_dir = match home_dir() {
                Some(homedir) => homedir,
                None => return Err("Unknown home directory. Please specify a secret key file on the command line.".into()),
            };
            ssh_dir.push(".ssh");
            for file in fs::read_dir(ssh_dir).map_err(|e| e.to_string())? {
                let file = file.map_err(|e| e.to_string())?;
                let sk_str = fs::read_to_string(file.path()).map_err(|e| e.to_string())?;
                try_sk = SecretKey::from_armor(&sk_str).ok();
                if try_sk.is_some() { break }
            }
            match try_sk {
                Some(found_sk) => sk = found_sk,
                None => return Err("no-sk-found".into()),
            }
        }

        // Do the proof
        let proof = ringsig::prove(&keys, contents.message.as_bytes(), sk)?;
        contents.proof = Some(proof.to_hex());
        println!("{}", serde_json::to_value(&contents).expect("serializing JSON"));
    }

    // Obtain proof for verifying
    if args[1] == "verify" {
        let proof = contents.proof.ok_or("missing proof in JSON")?;
        let proof = Vec::<u8>::from_hex(&proof).map_err(|e| e.to_string())?;
        ringsig::verify(&proof, &keys, contents.message.as_bytes())?;
        println!("{}", contents.message);
        println!("-----END OF MESSAGE-----");
        println!("SUCCESSFULLY VERIFIED PROOF with one of");
        for key in &contents.pks {
            println!("{}", key);
        }
    }

    Ok(())
}

