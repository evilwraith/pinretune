use psaperio::{config,scrambler::{IdScrambler}};

fn main() {
    let config = config::PsConfig::new();
    _ = config.load("config.pinsound");
    
    let scrambler = IdScrambler::new("JeanLucPicard");

    println!("{}", scrambler.unscramble(scrambler.scramble(500)));
}