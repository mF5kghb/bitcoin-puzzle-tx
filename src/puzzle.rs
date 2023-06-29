#![allow(dead_code)]

use std::ops::{Add, Sub};

use anyhow::anyhow;
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::{All, PublicKey, Scalar, Secp256k1, SecretKey};
use num_bigint::{BigUint, RandBigInt};
use num_traits::{Num, One};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::speed_checker::SpeedChecker;

static CURVE: Lazy<Secp256k1<All>> = Lazy::new(Secp256k1::new);

pub struct Puzzle {
    pub number: u8,
    pub ripemd160_address: [u8; 20],
    pub address: String,
    pub range: String,
    pub solution: Option<String>,
    speed_checker: SpeedChecker,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PuzzleJson {
    number: u8,
    address: String,
    range: String,
    private: Option<String>,
}

#[derive(Debug)]
pub enum Mode {
    Random { increment: BigUint },
    LinearButStartAtRandom,
    Linear,
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
            speed_checker: SpeedChecker::new(),
        }
    }

    fn range(&self) -> (BigUint, BigUint) {
        let range: Vec<BigUint> = self.range
            .split(':')
            .map(|value| BigUint::from_str_radix(value, 16).unwrap())
            .collect();

        (range[0].clone(), range[1].clone())
    }

    fn get_public_key(&self, private_key: &BigUint) -> anyhow::Result<PublicKey> {
        let mut private_key_bytes = private_key.to_bytes_le();
        private_key_bytes.resize(32, 0);
        private_key_bytes.reverse();

        let secret = SecretKey::from_slice(&private_key_bytes)?;

        Ok(PublicKey::from_secret_key(&CURVE, &secret))
    }

    pub fn start(&mut self, mode: Mode) -> anyhow::Result<String> {
        println!("Starting puzzle #{} {:?}", self.number, self.address);
        println!("Mode: {:?}", mode);

        match mode {
            Mode::Random { increment } => self.random_mode(increment),
            Mode::Linear => self.linear_mode(),
            Mode::LinearButStartAtRandom => self.linear_but_start_at_random_mode(),
        }
    }

    pub fn linear_but_start_at_random_mode(&mut self) -> anyhow::Result<String> {
        let (low, high) = self.range();

        let mut generator = rand::thread_rng();
        let min = generator.gen_biguint_range(&low, &high);

        println!("Range {}:{}", min.to_str_radix(16).to_uppercase(), high.to_str_radix(16).to_uppercase());

        self.compute(&min, &high)
    }

    pub fn linear_mode(&mut self) -> anyhow::Result<String> {
        let (low, high) = self.range();

        self.compute(&low, &high)
    }

    pub fn random_mode(&mut self, increment: BigUint) -> anyhow::Result<String> {
        let (low, high) = self.range();
        let mut generator = rand::thread_rng();

        loop {
            let max = generator.gen_biguint_range(&low, &high);
            let min = max.clone().sub(&increment);

            if min < low {
                continue;
            }

            if let Ok(private_key) = self.compute(&min, &max) {
                return Ok(private_key);
            }
        }
    }

    fn compute(&mut self, min: &BigUint, max: &BigUint) -> anyhow::Result<String> {
        let mut counter = min.clone();
        let mut public_key = self.get_public_key(&counter)?;

        while counter < *max {
            public_key = public_key.add_exp_tweak(&CURVE, &Scalar::ONE)?;
            counter = counter.add(BigUint::one());

            let hashed = bitcoin::hashes::sha256::Hash::hash(&public_key.serialize());
            let hashed = bitcoin::hashes::ripemd160::Hash::hash(&hashed.to_byte_array());

            if self.ripemd160_address == hashed.as_ref() {
                return Ok(counter.to_str_radix(16));
            }

            self.speed_checker.update();
        }

        Err(anyhow!("Solution not found..."))
    }
}
