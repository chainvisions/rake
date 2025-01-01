use clap::Parser;
use heck::ToTitleCase;
use rand::{seq::IteratorRandom, Rng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, RwLock,
    },
    thread,
    time::{SystemTime, UNIX_EPOCH},
};
use tiny_keccak::{Hasher, Keccak};

const MAX_ITER_COUNT: u64 = 0xffffffffffff; // TODO: Temporarily until we handle the math better.
const PREPOSITIONS: [&'static str; 8] = ["as", "of", "for", "by", "like", "in", "from", "into"];

#[derive(Parser, Debug)]
struct Args {
    #[arg(
        short = 'd',
        long,
        value_delimiter = ',',
        help = "List of known words to use in an attempt to brute force a matching signature",
        required = true
    )]
    dictionary: Vec<String>,
    #[arg(
        short = 'a',
        long = "args",
        help = "Arguments of the function being brute forced, used for constructing a valid signature"
    )]
    func_args: String,
    #[arg(
        short = 'm',
        long,
        default_value = "00000000",
        help = "Selector to attempt to create a matching signature for"
    )]
    match_selector: String,
    #[arg(
        short = 'o',
        long,
        default_value = "false",
        help = "Enable to submit the matching signature to Openchain after a successful match"
    )]
    openchain: bool,
    #[arg(
        short = 'p',
        long,
        default_value = "false",
        long_help = "Append a few common English prepositions (as, of, for, by, like, in, from, into) to the dictionary. Adds potential accuracy to brute forcing whilst adding some additional overhead cost due to additional combinations to sift through"
    )]
    prepositions: bool,
}

impl Args {
    fn append_prepositions(&mut self) {
        for prep in PREPOSITIONS.into_iter() {
            self.dictionary.push(prep.to_string());
        }
    }
}

#[derive(Serialize, Deserialize)]
struct OpenchainImportParams {
    function: Vec<String>,
}

fn main() {
    let mut args = Args::parse();
    if args.prepositions {
        args.append_prepositions();
    }
    start_cracking(args);
}

fn fact(num: u128) -> u128 {
    (1..=num).product()
}

fn start_cracking(args: Args) {
    // Calculate total possibilities.
    //let num_possible = fact(u128::try_from(args.dictionary.len()).unwrap() * 2_u128);
    // TODO: Handle the above ^ math better
    println!("Dictionary loaded: {:?}", args.dictionary);
    println!(
        "Iterating over {} total possibilities or factorial({})",
        0,
        args.dictionary.len() * 2
    );
    let mut hasher = Keccak::v256();
    let sig_list = Arc::new(RwLock::new(HashMap::new()));
    let (tx, rx): (Sender<String>, Receiver<String>) = channel();

    // Spawn iterators and calculate the hashes.
    thread::spawn(move || {
        (0..MAX_ITER_COUNT).into_par_iter().for_each(|_| {
            let sig_map = sig_list.clone();
            let mut rng = rand::thread_rng();
            let total_words: u128 =
                rng.gen_range(0..u128::try_from(args.dictionary.len()).unwrap());
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
            tx.clone().send(func_sig.clone()).unwrap();
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
            //println!("function {} => {}", func_sig, func_selector);
        });
    });

    let mut start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let mut nonce: u64 = 1;
    while let Ok(_received) = rx.recv() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let elapsed = now - start_time;
        if elapsed > 1.5 {
            let rate: f64 = nonce as f64 / elapsed;
            println!("\x1B[2J\x1B[1;1H"); // Clear screen
            println!("Rate: {:.0} signatures per second", rate);
            // Measure every 2.5 seconds.
            start_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            nonce = 1;
        }

        nonce += 1;
    }
}
