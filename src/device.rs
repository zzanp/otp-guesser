pub trait Device {
    async fn run(
        encrypted: String,
        charset: String,
        wordlist: Vec<String>,
        cores: usize,
        callback: fn(&str, &str),
    );
    async fn guess(&self, chunk: &[Vec<usize>]);
}
