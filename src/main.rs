use clap::Parser;
use heck::ToTitleCase;
use rand::{seq::IteratorRandom, Rng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tiny_keccak::{Hasher, Keccak};

const MAX_ITER_COUNT: u64 = 0xffffffffffff; // TODO: Temporarily until we handle the math better.

#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'd', long, value_delimiter = ',')]
    dictionary: Vec<String>,
    #[arg(short = 'a', long = "args")]
    func_args: String,
    #[arg(short = 'm', long, default_value = "00000000")]
    match_selector: String,
    #[arg(short = 'o', long, default_value = "false")]
    openchain: bool,
}

#[derive(Serialize, Deserialize)]
struct OpenchainImportParams {
    function: Vec<String>,
}

fn main() {
    let args = Args::parse();
    start_cracking(args);
}

fn fact(num: u128) -> u128 {
    (1..=num).product()
}

fn start_cracking(args: Args) {
    // Calculate total possibilities.
    //let num_possible = fact(u128::try_from(args.dictionary.len()).unwrap() * 2_u128);
    // TODO: Handle the above ^ math better
    println!(
        "Iterating over {} total possibilities or factorial({})",
        0,
        args.dictionary.len() * 2
    );
    let mut hasher = Keccak::v256();
    let sig_list = Arc::new(RwLock::new(HashMap::new()));

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
        println!("function {} => {}", func_sig, func_selector);
    });
}
