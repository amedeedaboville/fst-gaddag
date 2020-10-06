use std::collections::BTreeSet;
use std::str::from_utf8;
use fst::{IntoStreamer, Streamer, Set, SetBuilder};
use fst::automaton::{Automaton, Subsequence};
use regex_automata::dense;

pub struct Gaddag(fst::Set<Vec<u8>>);
static SEP : u8 = ',' as u8;
static SEP_CHAR : char = SEP as char;

pub fn build_entries(input: impl IntoIterator<Item= String>) -> BTreeSet<Vec<u8>> {
    let mut entries: BTreeSet<Vec<u8>> = BTreeSet::new();
    let mut new_word : Vec<u8> = Vec::with_capacity(15);
    let mut before_diamond: Vec<u8> = Vec::with_capacity(15);
    let mut after_diamond: Vec<u8> = Vec::with_capacity(15);
    for word in input.into_iter() {
        after_diamond.clear();
        before_diamond.clear();
        before_diamond.extend(word.as_bytes());
        #[cfg(feature = "debug")]
        println!("Building entries for {}", word);
        while before_diamond.len() > 0 {

            new_word.clear();
            new_word.extend(before_diamond.iter().rev());
            new_word.push(SEP);
            new_word.extend(after_diamond.iter().rev());
            after_diamond.push(before_diamond.pop().unwrap());

            #[cfg(feature = "debug")]
            println!("Inserting entry {}", from_utf8(&new_word).unwrap());
            entries.insert(new_word.iter().cloned().collect());
        }
    }
    entries
}

impl Gaddag {
    /*
    pub fn ends_with(&self, input: &str) -> Iterator<Item = String> {
        //$input_rev[a-z]*, 
        //reverse input, and look for that + , endword
        //self.0.startswith(input.rev())
        let pattern = r"(?i)foo";
        let dfa = dense::Builder::new().anchored(true).build(pattern).unwrap();;
        let mut stream = self.0.search(&dfa).into_stream();
        self.0.search(search_vec)
        }
        */
    /*
    pub fn starts_with(&self, input: &str)-> Iterator<Item = String>  {
        //look up input.rev() + ','
        let search_str = input.rev() + SEP;
        let matcher = StartsWith(
        self.0.starts_with(search_str)
        }
        */
    //exact match
    pub fn contains(&self, input: &str) -> bool {
        let mut search_vec : Vec<u8> = (*input.chars().rev().collect::<String>().as_bytes()).to_vec();
        search_vec.push(SEP);
        #[cfg(feature = "debug")]
        println!("Searching for {}", from_utf8(&search_vec).unwrap());
        self.0.contains(search_vec)
    }
    //rev the input, return everything before the comma and after
    pub fn substring(&self, input: &str) -> Vec<(String, String)> {
        let search_val : String = input.chars().rev().collect();
        #[cfg(feature = "debug")]
        println!("Searching for {}", search_val);

        let matcher = Subsequence::new(&search_val).starts_with();
        let mut stream = self.0.search(matcher).into_stream();
        let mut keys = Vec::new();
        while let Some(key) = stream.next() {
            let gaddag_string : String = String::from_utf8(key.to_vec()).unwrap();
            let (pre, suf) = gaddag_string.split_at((&gaddag_string).find(SEP as char).unwrap());
            keys.push((pre.to_string().chars().rev().collect(), suf.to_string()));
        }
        keys
    }
    pub fn from(set : Set<Vec<u8>>) -> Gaddag {
        return Gaddag(set);
    }
}
