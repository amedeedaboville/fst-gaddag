use std::collections::BTreeSet;
use std::fs::{read, read_to_string, File};
use std::io::{self, prelude::*, BufReader};
use std::str::from_utf8;

use fst::raw::Fst;
use fst::{IntoStreamer, Set, SetBuilder, Streamer};
use fst_gaddag::{build_entries, Gaddag};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("dictionary.txt")?;
    let reader = BufReader::new(file);
    let words: BTreeSet<Vec<u8>> = build_entries(reader.lines().map(|l| l.unwrap()));

    let wtr = io::BufWriter::new(File::create("dictionary.fst")?);
    let mut build = SetBuilder::new(wtr).unwrap();
    build.extend_iter(words).unwrap();
    build.finish().unwrap();
    println!("Done writing dictionary!");
    let fst = Set::new(read("dictionary.fst")?)?;
    println!("Done reading dictionary!");
    let set: Gaddag = Gaddag::from_fst(fst);
    println!("dict contains COW : {} ", set.contains("COW"));
    println!("dict contains AA : {} ", set.contains("AA"));
    println!("dict words with .*TRING: {:#?} ", set.ends_with("TRING"));

    Ok(())
}
