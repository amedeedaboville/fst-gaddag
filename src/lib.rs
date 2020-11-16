use fst::automaton::{Automaton, Str, Subsequence};
pub use fst::raw::{CompiledAddr, Node};
use fst::{IntoStreamer, Set, Result};
use std::collections::BTreeSet;
use std::iter;


pub struct Gaddag(fst::Set<Vec<u8>>);
pub static SEP: u8 = ',' as u8;
pub static SEP_STR: &str = ",";

static MAX_WORD_LENGTH: usize = 15;

pub fn build_entries(input: impl IntoIterator<Item = String>) -> BTreeSet<Vec<u8>> {
    let mut entries: BTreeSet<Vec<u8>> = BTreeSet::new();
    let mut new_word: Vec<u8> = Vec::with_capacity(MAX_WORD_LENGTH);
    let mut before_sep: Vec<u8> = Vec::with_capacity(MAX_WORD_LENGTH);
    let mut after_sep: Vec<u8> = Vec::with_capacity(MAX_WORD_LENGTH);
    for word in input.into_iter() {
        after_sep.clear();
        before_sep.clear();

        before_sep.extend(word.as_bytes());
        after_sep.push(before_sep.pop().unwrap());

        let whole_word_rev = word.chars().rev().collect::<String>().as_bytes().to_vec();
        entries.insert(whole_word_rev);

        while before_sep.len() > 0 {
            new_word.clear();
            new_word.extend(before_sep.iter().rev());
            new_word.push(SEP);
            new_word.extend(after_sep.iter().rev());
            after_sep.push(before_sep.pop().unwrap());

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
    ///Returns all the words that end with a given suffix.
    ///Searches for `^input.rev()[^,]*` . That is, the reversed input plus
    ///any sequence that doesn't include the separator.
    pub fn ends_with(&self, input: &str) -> Vec<String> {
        //looks up input.rev(), then filters down to things that do not have a comma
        let search_val: String = input.chars().rev().collect();

        let matcher = Str::new(&search_val)
            .starts_with()
            .intersection(Subsequence::new(SEP_STR).complement());

        let stream = self.0.search(matcher).into_stream();
        stream
            .into_strs()
            .unwrap()
            .iter()
            .map(|w| Self::demangle_item(w))
            .collect()
    }

    ///Returns all the words that start with a given prefix.
    ///Searches for `^input.rev(),.*`
    pub fn starts_with(&self, input: &str) -> Vec<String> {
        let search_val: String = input.chars().rev().chain(iter::once(SEP as char)).collect();
        let matcher = Str::new(&search_val).starts_with();
        self.search_fst(matcher)
    }

    ///Returns true if the given word is in the dictionary.
    ///Searches for `^input.rev()$`.
    pub fn contains(&self, input: &str) -> bool {
        let search_vec: Vec<u8> = (*input.chars().rev().collect::<String>().as_bytes()).to_vec();
        self.0.contains(search_vec)
    }

    ///Returns all the words that contain the input anywhere in them.
    ///Searches for `^input.rev().*`
    pub fn substring(&self, input: &str) -> Vec<String> {
        let search_val: String = input.chars().rev().collect();
        let matcher = Str::new(&search_val).starts_with();
        self.search_fst(matcher)
    }

    ///Takes a Fst::Set and returns a Gaddag.
    pub fn from_fst(set: Set<Vec<u8>>) -> Gaddag {
        Gaddag(set)
    }

    ///Builds a Gaddag from its byte representation.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Gaddag> {
        let fst_set = Set::new(bytes)?;
        Ok(Gaddag::from_fst(fst_set))
    }

    ///Builds a Gaddag from an input list of words.
    pub fn from_words(input: impl IntoIterator<Item = String>) -> Gaddag {
        Self::from_fst(Set::from_iter(build_entries(input)).unwrap())
    }

    ///Returns the byte representation of the Gaddag.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_fst().as_bytes()
    }

    ///Applies a fst matcher to the Gaddag, and returns all the words that
    ///match.
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

    ///Turns the GADDAG row for a word back into that word.
    ///For example GINT+BOA will demangle to BOATING.
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

    ///Returns the node address for a prefix in the dictionary.
    ///This means the input doesn't have to be a full word, but has to be a prefix
    ///of a word in the dictionary. Will return None if the word doesn't exist in the
    ///dictionary.
    pub fn node_for_prefix(&self, prefix: &str) -> Option<CompiledAddr> {
        let mut current_node: Node = self.0.as_fst().root();
        for c in prefix.chars() {
            if let Some(transition_idx) = current_node.find_input(c as u8) {
                //TODO don't just cast char to u8
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
    ///Attempts to follow the node in the GADDAG, and returns the next node.
    pub fn can_next(&self, node_addr: CompiledAddr, next: char) -> Option<CompiledAddr> {
        let current_node = self.0.as_fst().node(node_addr);
        current_node
            .find_input(next as u8) //TODO enumerate the bytes of char
            .map(|i| current_node.transition(i).addr)
    }
}
