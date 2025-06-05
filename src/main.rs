mod cpu;
mod device;

use crate::cpu::CPU;
use crate::device::Device;
use clap::Parser;
use minus::Pager;
use std::fs;
use std::path::PathBuf;
use std::process::exit;

/// Very fast tool to bruteforce one-time pad ciphertext.
#[derive(Parser)]
#[command(arg_required_else_help = true)]
struct CliArgs {
    /// Text you want to decrypt
    encrypted_word: Option<String>,
    /// Path to your wordlist
    wordlist: Option<PathBuf>,
    #[arg(short, long, default_value_t = String::from("qwertyuiopasdfghjklzxcvbnm"))]
    charset: String,
    /// [default: number of device cores]
    #[arg(short, long)]
    threads: Option<usize>,
    /// Use GPU
    #[arg(short, long, default_value_t = false)]
    gpu: bool,
    /// Print license
    #[arg(short, long)]
    license: bool,
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();
    // if args.encrypted_word.is_none() {
    //     println!("{}", args.a)
    // }
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
    let wordlist: Vec<String> = fs::read_to_string(args.wordlist.unwrap())
        .unwrap()
        .lines()
        .map(String::from)
        .collect();
    let encrypted = args.encrypted_word.unwrap();
    let charset = args.charset;

    if args.gpu {
        panic!("GPU support is not implemented yet.");
    } else {
        let cpus = args.threads.unwrap_or(num_cpus::get());
        CPU::run(encrypted, charset, wordlist, cpus, |plain, key| {
            println!("Key: {}, Plain: {}", key, plain);
        })
        .await;
    }
}
