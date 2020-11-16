use std::fs::File;
use std::io::{prelude::*, BufReader};

use fst_gaddag::Gaddag;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("dictionary.txt")?;
    let reader = BufReader::new(file);
    let set: Gaddag = Gaddag::from_words(reader.lines().map(|l| l.unwrap()));

    println!("dict contains COW : {} ", set.contains("COW"));
    println!("dict contains AA : {} ", set.contains("AA"));
    println!("dict words with .*TRING: {:#?} ", set.ends_with("TRING"));

    Ok(())
}
