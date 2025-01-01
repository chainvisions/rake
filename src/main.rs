use clap::Parser;
use rake::{constants::PREPOSITIONS, start_cracking, RakeBuilder};

#[derive(Parser, Debug)]
struct Args {
    #[arg(
        short = 'd',
        long,
        value_delimiter = ',',
        help = "List of known words to use in an attempt to brute force a matching signature",
        required = true
    )]
    pub dictionary: Vec<String>,
    #[arg(
        short = 'a',
        long = "args",
        help = "Arguments of the function being brute forced, used for constructing a valid signature"
    )]
    pub func_args: String,
    #[arg(short = 'i', long, help = "Open Rake inside of an interactive TUI")]
    pub interactive: bool,
    #[arg(
        short = 'm',
        long,
        default_value = "00000000",
        help = "Selector to attempt to create a matching signature for"
    )]
    pub match_selector: String,
    #[arg(
        short = 'o',
        long,
        default_value = "false",
        help = "Enable to submit the matching signature to Openchain after a successful match"
    )]
    pub openchain: bool,
    #[arg(
        short = 'p',
        long,
        default_value = "false",
        long_help = "Append a few common English prepositions (as, of, for, by, like, in, from, into) to the dictionary. Adds potential accuracy to brute forcing whilst adding some additional overhead cost due to additional combinations to sift through"
    )]
    pub prepositions: bool,
}

impl Args {
    pub fn append_prepositions(&mut self) {
        for prep in PREPOSITIONS.into_iter() {
            self.dictionary.push(prep.to_string());
        }
    }
}

fn main() {
    let mut args = Args::parse();
    if args.prepositions {
        args.append_prepositions();
    }
    let flags = RakeBuilder::new()
        .dictionary(args.dictionary)
        .function_args(args.func_args)
        .match_selector(args.match_selector)
        .with_openchain(args.openchain)
        .build();
    start_cracking(flags);
}
