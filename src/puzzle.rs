#![allow(dead_code)]

use std::ops::{Sub};
use anyhow::{anyhow};
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::{All, PublicKey, Scalar, Secp256k1, SecretKey};
use once_cell::sync::Lazy;
use rug::{Integer};
use rug::integer::Order;
use rug::rand::RandState;
use serde::{Deserialize, Serialize};

static CURVE: Lazy<Secp256k1<All>> = Lazy::new(Secp256k1::new);

pub struct Puzzle {
    pub number: u8,
    pub ripemd160_address: [u8; 20],
    pub address: String,
    pub range: String,
    pub solution: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PuzzleJson {
    number: u8,
    address: String,
    range: String,
    private: Option<String>,
}

pub enum Mode {
    Random { increment: Integer },
    Linear
}

impl Puzzle {
    pub fn number(data: usize) -> Puzzle {
        let input_path = "./puzzles.json";
        let text = std::fs::read_to_string(input_path).unwrap();
        let puzzles = serde_json::from_str::<Vec<PuzzleJson>>(&text).unwrap();
        let data = puzzles.get(data - 1).unwrap();

        Puzzle::from_json(data)
    }

    pub fn from_json(data: &PuzzleJson) -> Puzzle {
        Puzzle::new(
            data.number.to_owned(),
            data.address.to_owned(),
            data.range.to_owned(),
            data.private.to_owned(),
        )
    }

    pub fn new(number: u8, address: String, range: String, solution: Option<String>) -> Puzzle {
        let mut decoded = bitcoin::base58::decode_check(address.as_str()).unwrap();
        decoded.remove(0);
        let ripemd160_address: [u8; 20] = decoded.try_into().unwrap();

        Puzzle {
            number,
            ripemd160_address,
            address,
            range,
            solution,
        }
    }

    fn range(&self) -> (Integer, Integer) {
        let range: Vec<Integer> = self.range
            .split(':')
            .map(|value| Integer::from_str_radix(value, 16).unwrap())
            .collect();

        (range[0].clone(), range[1].clone())
    }

    fn get_public_key(&self, private_key: &Integer) -> anyhow::Result<PublicKey> {
        let mut private_key_bytes = private_key.to_digits::<u8>(Order::LsfBe);
        private_key_bytes.resize(32, 0);
        private_key_bytes.reverse();

        let secret = SecretKey::from_slice(&private_key_bytes)?;

        Ok(PublicKey::from_secret_key(&CURVE, &secret))
    }

    pub fn start(&self, mode: Mode) -> anyhow::Result<String> {
        println!("Starting puzzle #{} {:?}", self.number, self.address);

        match mode {
            Mode::Random { increment } => self.random_mode(increment),
            Mode::Linear => self.linear_mode()
        }
    }

    pub fn linear_mode(&self) -> anyhow::Result<String> {
        let (low, high) = self.range();

        self.compute(&low, &high)
    }

    pub fn random_mode(&self, increment: Integer) -> anyhow::Result<String> {
        let (low, high) = self.range();
        let mut rand = RandState::new();

        loop {
            let max = high.clone().random_below(&mut rand);
            let min = max.clone().sub(&increment);

            if min < low {
                continue;
            }

            if let Ok(private_key) = self.compute(&min, &max) {
                return Ok(private_key);
            }
        }
    }

    fn compute(&self, min: &Integer, max: &Integer) -> anyhow::Result<String> {
        let mut counter = min.clone();
        let mut public_key = self.get_public_key(&counter)?;

        while counter < *max {
            public_key = public_key.add_exp_tweak(&CURVE, &Scalar::ONE)?;
            counter += 1;

            let hashed = bitcoin::hashes::sha256::Hash::hash(&public_key.serialize());
            let hashed = bitcoin::hashes::ripemd160::Hash::hash(&hashed.to_byte_array());

            if self.ripemd160_address == hashed.as_ref() {
                return Ok(counter.to_string_radix(16));
            }
        }

        Err(anyhow!("Solution not found..."))
    }
}
