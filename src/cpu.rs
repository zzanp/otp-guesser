use crate::device::Device;
use itertools::Itertools;
use tokio::task::JoinSet;

#[derive(Clone)]
pub struct CPU {
    encrypted: String,
    charset: String,
    wordlist: Vec<String>,
    callback: fn(&str, &str),
}

impl CPU {
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
}

impl Device for CPU {
    async fn run(
        encrypted: String,
        charset: String,
        wordlist: Vec<String>,
        cpus: usize,
        callback: fn(&str, &str),
    ) {
        let mut tasks = JoinSet::default();

        let char_iter: Vec<Vec<usize>> = (0..charset.chars().count())
            .combinations(encrypted.len())
            .collect();

        for chunk in char_iter.chunks(charset.chars().count() / cpus) {
            let chunk = chunk.to_owned();
            let cpu = Self {
                encrypted: encrypted.clone(),
                charset: charset.clone(),
                wordlist: wordlist.clone(),
                callback,
            };
            tasks.spawn(async move {
                cpu.guess(&*chunk).await;
            });
        }
        tasks.join_all().await;
    }
    async fn guess(&self, chunk: &[Vec<usize>]) {
        for char_indexes in chunk {
            let mut key = String::new();
            for char_idx in char_indexes {
                key.push(self.charset.chars().nth(*char_idx).unwrap());
            }

            let plain = CPU::otp_decrypt(self.encrypted.as_str(), &key);

            for word in &self.wordlist {
                if word.len() != key.len() {
                    continue;
                }
                let check: Vec<bool> = plain.split(' ').map(|item| item == word).collect();
                if check.contains(&true) {
                    (self.callback)(&plain, &key);
                }
            }
        }
    }
}
