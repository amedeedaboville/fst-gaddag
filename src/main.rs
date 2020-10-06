use std::fs::{File, read, read_to_string};
use std::collections::BTreeSet;
use std::io::{self, prelude::*, BufReader};

use gaddag_bundle::{build_entries, Gaddag};
use fst::{IntoStreamer, Streamer, Set, SetBuilder};
use fst::raw::Fst;

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
    let set : Gaddag =  Gaddag::from(fst);
    println!("dict contains COW : {} ", set.contains("COW"));
    println!("dict words with .*COW.*: {:#?} ", set.substring("COW"));
    println!("dict words with COW.*: {:#?} ", set.starts_with("COW"));
    println!("dict words with .*LY: {:#?} ", set.ends_with("LY"));

    Ok(())
}
