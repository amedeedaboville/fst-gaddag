use fst::automaton::{Automaton, Str, Subsequence};
use fst::raw::{CompiledAddr, Node};
use fst::{IntoStreamer, Set};
use std::collections::BTreeSet;
use std::iter;
use std::str::from_utf8;

pub struct Gaddag(fst::Set<Vec<u8>>);
pub static SEP: u8 = ',' as u8;
pub static SEP_STR: &str = ",";
static MAX_WORD_LENGTH: usize = 15;

pub fn build_entries(input: impl IntoIterator<Item = String>) -> BTreeSet<Vec<u8>> {
    let mut entries: BTreeSet<Vec<u8>> = BTreeSet::new();
    let mut new_word: Vec<u8> = Vec::with_capacity(MAX_WORD_LENGTH);
    let mut before_diamond: Vec<u8> = Vec::with_capacity(MAX_WORD_LENGTH);
    let mut after_diamond: Vec<u8> = Vec::with_capacity(MAX_WORD_LENGTH);
    for word in input.into_iter() {
        after_diamond.clear();
        before_diamond.clear();

        before_diamond.extend(word.as_bytes());
        after_diamond.push(before_diamond.pop().unwrap());

        //#[cfg(feature = "debug")]
        //println!("Building entries for {}", word);

        let whole_word_rev = word.chars().rev().collect::<String>().as_bytes().to_vec();
        //#[cfg(feature = "debug")]
        //println!("Inserting entry {}", from_utf8(&whole_word_rev).unwrap());
        entries.insert(whole_word_rev);

        while before_diamond.len() > 0 {
            new_word.clear();
            new_word.extend(before_diamond.iter().rev());
            new_word.push(SEP);
            new_word.extend(after_diamond.iter().rev());
            after_diamond.push(before_diamond.pop().unwrap());
            //#[cfg(feature = "debug")]
            //println!("Inserting entry {}", from_utf8(&new_word).unwrap());

            entries.insert(new_word.iter().cloned().collect());
        }
    }
    entries
}

/*
 * CARES:
 * SERAC
 * ERAC,S
 * RAC,ES
 * AC,RES
 * C,ARES
 *
 * SERUM:
 * MURES
 * URES,M
 * RES,UM
 * ES,RUM
 * S,ERUM
 *
 * CARESS
 * SSERAC
 * SERAC,S
 * ERAC,SS
 * RAC,ESS
 * AC,RESS
 * C,ARESS
 */

impl Gaddag {
    pub fn ends_with(&self, input: &str) -> Vec<String> {
        //looks up input.rev(), then filters down to things that do not have a comma
        let search_val: String = input.chars().rev().collect();
        //#[cfg(feature = "debug")]
        println!("Searching for {}", search_val);

        let matcher = Str::new(&search_val)
            .starts_with()
            .intersection(Subsequence::new(SEP_STR).complement());

        let mut stream = self.0.search(matcher).into_stream();
        stream
            .into_strs()
            .unwrap()
            .iter()
            .map(|w| Self::demangle_item(w))
            .collect()
    }

    pub fn starts_with(&self, input: &str) -> Vec<String> {
        //looks up input.rev() + ','
        let search_val: String = input.chars().rev().chain(iter::once(SEP as char)).collect();
        #[cfg(feature = "debug")]
        println!("Searching for {}", search_val);

        let matcher = Str::new(&search_val).starts_with();
        let mut stream = self.0.search(matcher).into_stream();
        stream
            .into_strs()
            .unwrap()
            .iter()
            .map(|w| Self::demangle_item(w))
            .collect()
    }

    //exact match
    pub fn contains(&self, input: &str) -> bool {
        let mut search_vec: Vec<u8> =
            (*input.chars().rev().collect::<String>().as_bytes()).to_vec();
        #[cfg(feature = "debug")]
        println!("Searching for {}", from_utf8(&search_vec).unwrap());
        self.0.contains(search_vec)
    }

    //all the words that have 'input' in the middle of them
    pub fn substring(&self, input: &str) -> Vec<String> {
        let search_val: String = input.chars().rev().collect();
        #[cfg(feature = "debug")]
        println!("Searching for {}", search_val);

        let matcher = Str::new(&search_val).starts_with();
        let mut stream = self.0.search(matcher).into_stream();
        stream
            .into_strs()
            .unwrap()
            .iter()
            .map(|w| Self::demangle_item(w))
            .collect()
    }

    pub fn from_fst(set: Set<Vec<u8>>) -> Gaddag {
        Gaddag(set)
    }

    pub fn from_words(input: impl IntoIterator<Item = String>) -> Gaddag {
        Self::from_fst(Set::from_iter(build_entries(input)).unwrap())
    }

    fn search_fst(&self, matcher: impl Automaton) -> Vec<String> {
        self.0
            .search(matcher)
            .into_stream()
            .into_strs()
            .unwrap()
            .iter()
            .map(|w| Self::demangle_item(w))
            .collect()
    }

    fn demangle_item(item: &str) -> String {
        if let Some(idx) = item.find(SEP as char) {
            item[0..idx]
                .chars()
                .rev()
                .chain(item[(idx + 1)..].chars())
                .collect()
        } else {
            item.chars().rev().collect()
        }
    }
    pub fn node_for_prefix(&self, prefix: &str) -> Option<CompiledAddr> {
        let mut current_node: Node = self.0.as_fst().root();
        for c in prefix.chars() {
            if let Some(transition_idx) = current_node.find_input(c as u8) {
                let next_node = self
                    .0
                    .as_fst()
                    .node(current_node.transition_addr(transition_idx));
                current_node = next_node;
            } else {
                return None;
            }
        }
        Some(current_node.addr())
    }
    pub fn can_next(&self, node_addr: CompiledAddr, next: char) -> Option<CompiledAddr> {
        let current_node = self.0.as_fst().node(node_addr);
        current_node
            .find_input(next as u8) //todo enumerate the bytes of char
            .map(|i| current_node.transition(i).addr)
    }
}
