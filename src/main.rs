use clap::Parser;
use itertools::Itertools;
use minus::Pager;
use std::fs;
use std::path::PathBuf;
use std::process::exit;
use tokio::task::JoinSet;

/// Very fast tool to bruteforce one-time pad ciphertext.
#[derive(Parser)]
struct CliArgs {
    /// Text you want to decrypt
    encrypted_word: Option<String>,
    /// Path to your wordlist
    wordlist: Option<PathBuf>,
    #[arg(short, long, default_value_t = String::from("qwertyuiopasdfghjklzxcvbnm"))]
    charset: String,
    /// [default: number of CPU cores]
    #[arg(short, long)]
    threads: Option<usize>,
    /// Print license
    #[arg(short, long)]
    license: bool,
}

fn otp_decrypt(text: &str, key: &str) -> String {
    let mut plain = String::new();
    for i in 0..key.chars().count() {
        let mut char = text.as_bytes()[i] as i16 - 97 - (key.as_bytes()[i] as i16 - 97);
        if char < 0 {
            char += 26;
        }
        char += 97;
        plain.push(char::from(char as u8));
    }
    plain
}

async fn guess(
    encrypted: String,
    char_iter: Vec<Vec<usize>>,
    charset: String,
    wordlist: Vec<String>,
    callback: fn(&str, &str),
) {
    for char_indexes in char_iter {
        let mut key = String::new();
        for char_idx in char_indexes {
            key.push(charset.chars().nth(char_idx).unwrap());
        }

        let plain = otp_decrypt(encrypted.as_str(), &key);

        for word in &wordlist {
            if word.len() != key.len() {
                continue;
            }
            let check: Vec<bool> = plain.split(' ').map(|item| item == word).collect();
            if check.contains(&true) {
                callback(&plain, &key);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();
    if args.license {
        let license = include_str!("../LICENSE");
        let print_license = |_| {
            println!("{}", license);
            exit(0);
        };
        let pager = Pager::new();
        pager.push_str(license).unwrap_or_else(print_license);
        minus::page_all(pager).unwrap_or_else(print_license);
        exit(0);
    }
    let mut tasks = JoinSet::default();
    let cpus = args.threads.unwrap_or(num_cpus::get());
    let wordlist: Vec<String> = fs::read_to_string(args.wordlist.unwrap())
        .unwrap()
        .lines()
        .map(String::from)
        .collect();
    let encrypted = args.encrypted_word.unwrap();
    let charset = args.charset;
    let char_iter: Vec<Vec<usize>> = (0..charset.chars().count())
        .combinations(encrypted.len())
        .collect();

    for chunk in char_iter.chunks(charset.chars().count() / cpus) {
        tasks.spawn(guess(
            encrypted.to_string(),
            chunk.to_vec(),
            charset.to_string(),
            wordlist.to_owned(),
            |plain, key| println!("{plain} for key: {key}"),
        ));
    }
    tasks.join_all().await;
}
