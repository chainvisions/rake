use color_eyre::Result;
use heck::ToTitleCase;
use rand::{seq::IteratorRandom, Rng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, RwLock,
    },
};
use tiny_keccak::{Hasher, Keccak};

pub mod constants;
use constants::MAX_ITER_COUNT;

pub struct RakeFlags {
    dictionary: Vec<String>,
    func_args: String,
    match_selector: String,
    openchain: bool,
}

impl RakeFlags {
    pub fn new(
        dictionary: Vec<String>,
        func_args: String,
        match_selector: String,
        openchain: bool,
    ) -> Self {
        Self {
            dictionary,
            func_args,
            match_selector,
            openchain,
        }
    }

    pub fn builder() -> RakeBuilder {
        RakeBuilder::default()
    }
}

#[derive(Default)]
pub struct RakeBuilder {
    dictionary: Vec<String>,
    func_args: String,
    match_selector: String,
    openchain: bool,
}

impl RakeBuilder {
    pub fn new() -> Self {
        Self {
            dictionary: Vec::new(),
            func_args: String::default(),
            match_selector: String::default(),
            openchain: false,
        }
    }
    pub fn build(self) -> RakeFlags {
        RakeFlags {
            dictionary: self.dictionary,
            func_args: self.func_args,
            match_selector: self.match_selector,
            openchain: self.openchain,
        }
    }

    pub fn dictionary(mut self, dictionary: Vec<String>) -> Self {
        self.dictionary = dictionary;
        self
    }

    pub fn function_args(mut self, func_args: String) -> Self {
        self.func_args = func_args;
        self
    }

    pub fn match_selector(mut self, selector: String) -> Self {
        self.match_selector = selector;
        self
    }

    pub fn with_openchain(mut self, openchain: bool) -> Self {
        self.openchain = openchain;
        self
    }
}

#[derive(Serialize, Deserialize)]
pub struct FunctionData {
    pub signature: String,
    pub selector: String,
}

#[derive(Serialize, Deserialize)]
struct OpenchainImportParams {
    function: Vec<String>,
}

pub fn start_cracking(args: RakeFlags) -> Result<Receiver<FunctionData>> {
    // Calculate total possibilities.
    //let num_possible = fact(u128::try_from(args.dictionary.len()).unwrap() * 2_u128);
    // TODO: Handle the above ^ math better
    println!("Dictionary loaded: {:?}", args.dictionary);
    println!(
        "Iterating over {} total possibilities or factorial({})",
        0,
        args.dictionary.len() * 2
    );
    let hasher = Keccak::v256();
    let sig_list = Arc::new(RwLock::new(HashMap::new()));
    let (tx, rx): (Sender<FunctionData>, Receiver<FunctionData>) = mpsc::channel();

    // Spawn iterators and calculate the hashes.
    (0..MAX_ITER_COUNT).into_par_iter().for_each(|_| {
        let sig_map = sig_list.clone();
        let mut rng = rand::thread_rng();
        let total_words: u128 = rng.gen_range(0..u128::try_from(args.dictionary.len()).unwrap());
        let word_sample = args
            .dictionary
            .iter()
            .choose_multiple(&mut rng, total_words.try_into().unwrap());
        let mut func_name = String::new();
        for (idx, word) in word_sample.iter().enumerate() {
            if idx == 0 {
                func_name.push_str(word.as_str());
            } else {
                func_name.push_str(word.to_title_case().as_str());
            }
        }
        let mut hash = hasher.clone();
        let mut hash_result = [0u8; 32];
        let func_sig = format!("{}({})", func_name, args.func_args);
        hash.update(func_sig.as_bytes());
        hash.finalize(&mut hash_result);
        let func_selector = hex::encode(hash_result[..4].to_vec());
        let map = sig_map.read().expect("Poisoned RwLock");
        if map.contains_key(&func_selector) {
            return;
        }
        drop(map);
        let mut map = sig_map.write().expect("Poisoned RwLock");
        map.insert(func_selector.clone(), func_sig.clone());
        if func_selector == args.match_selector {
            println!(
                "Found target function signature for {}: {}",
                args.match_selector, func_sig
            );
            if args.openchain {
                println!("Submitting to Openchain...");
                let openchain_params = OpenchainImportParams {
                    function: vec![func_sig],
                };
                let client = reqwest::blocking::Client::new();
                client
                    .post("https://api.openchain.xyz/signature-database/v1/import")
                    .header("accept", "application/json")
                    .header("Content-Type", "application/json")
                    .json(&openchain_params)
                    .send()
                    .unwrap();
                println!("Signature successfully submitted!");
            }
            std::process::exit(1);
        }
        tx.send(FunctionData {
            signature: func_sig,
            selector: func_selector,
        })
        .unwrap();
    });

    Ok(rx)
}
