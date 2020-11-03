# fst-gaddag

An unfinished GADDAG implementation in Rust built with the [fst](https://github.com/BurntSushi/fst) crate.

I'm not using this yet in any real capacity so it's unlikely to be ready to be usable (or even correct).

What's a GADDAG?
----
A GADDAG is a data structure invented for scrabble AIs to generate all possible words that contain a subsequence.

You could imagine using a regular prefix trie to quickly answer:

* "what can I build by appending letters to PLAIN" (PLAINS, PLAINLY, PLAINTIFF, ...)

or the same data in a reverse order to answer
* "what can I build by pre-pending to PLAIN" (EXPLAIN)

But what if you wanted to build EXPLAINING from PLAIN?

The GADDAG is a regular DAG that stores several variations of each word in a clever way to be able to answer that question.

Why is this one neat?
---
This one uses the fantastic `fst` crate as its underlying engine. I recommend reading the [blog post about it](https://blog.burntsushi.net/transducers/), but essentially it
optimises storing Sets or Maps of strings where the strings have a lot of overlap.

It has the ability to optimize shared postfixes as well as prefixes, so unlike a prefix trie, it can also share the last 3 nodes of eg
EXPLAINING and BOATING. It's also fast, and has _no_ dependencies for us.

How do I use it?
---

You build one by calling `from_words` with anything that implements `IntoIterator<string>`. For example, if you had a dictionary as a text file, you could
build it this way:

```rust
let file = File::open("dictionary.txt")?;
let reader = BufReader::new(file);
let gaddag = Gaddag::from_words(reader.lines().map(|l| l.unwrap()));
 ```
  
Then you can query it with:
```rust
println!("dict contains AA : {} ", gaddag.contains("AA"));
println!("dict words with .*TRING: {:#?} ", gaddag.ends_with("TRING"));
println!("dict words with BA.*: {:#?} ", gaddag.starts_with("BA"));
println!("dict words with .*PLING.*: {:#?} ", gaddag.substring("PLING"));
```

You may want to navigate it node by node when building your possible moves in a Scrabble game.
In that case you'll use the lower level `CompiledAddr` type from `fst` to refer to node addresses in the
gaddag.

You can get the node address for a prefix with:
```rust
let node_addr = gaddag.node_for_prefix("ban")
```
And then you can query for if you can continue with the tiles in your bag one by one with
```rust
if let Some(next_node_addr) = gaddag.can_next(node_addr, "a"); //checks if there is a word in the dictionary with the prefix "bana"
```

A note on text encodings
----
`fst` is optimized for storing bytes, which means that it's fine for utf8 strings as long as you remember to encode and decode your inputs and outputs.

This code has so far been only tested in English. While it should theoretically support multi-byte characters there
may be uncovered bugs in there.