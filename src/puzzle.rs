use std::ops::Add;
use std::str::FromStr;
use std::time::Instant;

use bitcoin::{Address, Network, PrivateKey, PublicKey};
use bitcoin::secp256k1::{All, Secp256k1};
use num_bigint::{BigUint, RandBigInt};
use num_traits::{FromPrimitive, One};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

static CURVE: Lazy<Secp256k1<All>> = Lazy::new(Secp256k1::new);

fn check_address(current: &BigUint, target: &Address, start: Instant) -> bool {
    let private_key_bytes = current.to_bytes_be();
    let length = private_key_bytes.len();

    let mut raw_private_key: Vec<u8> = vec![];
    raw_private_key.resize(32 - length, 0);
    raw_private_key.extend_from_slice(&private_key_bytes);

    let private = PrivateKey::from_slice(&raw_private_key, Network::Bitcoin).unwrap();
    let public = PublicKey::from_private_key(&CURVE, &private);
    let address = Address::p2pkh(&public, Network::Bitcoin);

    if address.to_string() == target.to_string() {
        println!("matched");
    }

    if address.eq(target) {
        let joined = raw_private_key.iter().map(|x| format!("{:02x}", x)).collect::<String>();
        let duration = start.elapsed();
        println!("{:?} {:?} {:?}", address, joined, duration);

        return true;
    }

    false
}

pub struct Puzzle {
    pub address: Address,
    pub range: String,
    pub solution: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PuzzleJson {
    address: String,
    range: String,
    private: Option<String>,
}

impl Puzzle {
    pub fn number(data: usize) -> Puzzle {
        let input_path = "./src/puzzles.json";
        let text = std::fs::read_to_string(input_path).unwrap();
        let puzzles = serde_json::from_str::<Vec<PuzzleJson>>(&text).unwrap();
        let data = puzzles.get(data - 1).unwrap();

        Puzzle::from_json(data)
    }

    pub fn from_json(data: &PuzzleJson) -> Puzzle {
        Puzzle::new(
            data.address.to_owned(),
            data.range.to_owned(),
            data.private.to_owned(),
        )
    }

    pub fn new(address: String, range: String, solution: Option<String>) -> Puzzle {
        Puzzle {
            address: Address::from_str(address.as_str()).unwrap().assume_checked(),
            range,
            solution,
        }
    }

    fn range(&self) -> (BigUint, BigUint) {
        let range: Vec<_> = self.range
            .split(':')
            .map(|value| value.chars().collect::<Vec<char>>())
            .map(|value| value.chunks(2).map(|mut value| value.iter().collect::<String>()).collect::<Vec<String>>())
            .map(|mut value| value.iter().map(|value| u8::from_str_radix(value, 16).unwrap()).collect::<Vec<u8>>())
            .collect();

        (
            BigUint::from_bytes_be(range[0].as_slice()),
            BigUint::from_bytes_be(range[1].as_slice()),
        )
    }

    pub fn compute(&self, increments: u32) {
        let mut random_generator = rand::thread_rng();
        let (low, high) = self.range();
        let increment = BigUint::from_u32(increments).unwrap();

        println!("Starting puzzle: {:?} {:?} {:?}", self.address.to_string(), self.range, self.solution);

        loop {
            let min = random_generator.gen_biguint_range(&low, &high);
            let max = min.clone().add(&increment);
            let mut value = min;
            let start = Instant::now();

            loop {
                if value.gt(&max) {
                    break;
                }

                if check_address(&value, &self.address, start) {
                    break;
                }

                value = value.add(&BigUint::one());
            }
        }
    }
}
