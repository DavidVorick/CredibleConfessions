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
use std::{env, fs, io};
use std::io::{BufRead, Read};

fn usage() -> Result<(), &'static str> {
    let name = env::args().next().unwrap();
    eprintln!("Usage: {} prove <public key list> <message file> [secret key file]", name);
    eprintln!("Usage: {} verify <public key list> <message file> <proof>", name);
    eprintln!("");
    eprintln!("Here <public key list> should refer to a text file containing a list of");
    eprintln!("public keys, in the .ssh/authorized_keys format, one on each line.");
    eprintln!("public keys, in the .ssh/authorized_keys format, one on each line.");
    eprintln!("");
    eprintln!("<message file> is the message to sign. To use stdin, provide '-'.");
    eprintln!("");
    eprintln!("If <secret key file> is provided this will be used as the signing key.");
    eprintln!("Otherwise, the tool will just try to use every file in ~/.ssh as a key.");
    eprintln!("");
    eprintln!("<proof> is a hex-encoded proof");
    eprintln!("");
    eprintln!("The message to sign is then provided on standard input. Use < to read");
    eprintln!("from a file instead.");
    Err("invalid-command-line-args")
}

fn main() -> Result<(), String> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        usage()?;
    }
    match &args[1][..] {
        "prove" if args.len() == 4 || args.len() == 5 => {},
        "verify" if args.len() == 5 => {},
        _ => usage()?,
    }

    // Obtain list of public keys
    let keyfile = fs::File::open(&args[2])
        .map(io::BufReader::new)
        .map_err(|e| e.to_string())?;
    let keys = keyfile
        .lines()
        .map(|result| result.map(|ln| PublicKey::parse_pk_line(&ln)))
        .filter_map(|result| match result {
            Ok(Ok(ok)) => Some(Ok(ok)),
            Ok(Err(ringsig::keys::Error::EmptyKey)) => None, // blank lines ok
            Ok(Err(e)) => Some(Err(format!("Parsing public keys from {}: {:?}", args[1], e))), // FIXME
            Err(e) => Some(Err(e.to_string())),
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Obtain message
    let mut msg = vec![];
    let mut file: Box<dyn Read> = if args[2] == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(fs::File::open(&args[2]).map_err(|e| e.to_string())?)
    };
    file.read_to_end(&mut msg).map_err(|e| e.to_string())?;

    // Obtain secret key for proving
    if args[1] == "prove" {
        let sk;
        if args.len() == 5 {
            let sk_str = fs::read_to_string(&args[4]).map_err(|e| e.to_string())?;
            sk = SecretKey::from_armor(&sk_str)
                .map_err(|e| format!("Reading secret key file {}: {:?}", args[4], e))?; // FIXME
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
        let proof = ringsig::prove(&keys, &msg, sk)?;
        println!("{}", proof.to_hex());
    }

    // Obtain proof for verifying
    if args[1] == "verify" {
        let proof = Vec::<u8>::from_hex(&args[4])
            .map_err(|e| e.to_string())?;

        // Obtain message
        let mut msg = vec![];
        io::stdin().read_to_end(&mut msg).map_err(|e| e.to_string())?;

        ringsig::verify(&proof, &keys, &msg)?;
    }

    Ok(())
}

