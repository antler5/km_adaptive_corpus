// Copyright 2025 antlers <antlers@illucid.net>
//
// SPDX-License-Identifier: GPL-3.0-only

//! Dynamic trigram adjustments for simple magic rules.
//!
//! Right now, "simple" means adaptive-key style bigram rules like `h* -> he`.
//!
//! # Examples
//!
//! ```no_run
//! use std::fs;
//! use kc::Corpus;
//! use km_adaptive_corpus::AdaptiveCorpus;
//!
//! let b = fs::read("./corpora/shai-iweb.corpus").unwrap();
//! let mut corpus: Corpus = rmp_serde::from_slice(&b).unwrap();
//! <Corpus as AdaptiveCorpus<[char; 1]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
//! <Corpus as AdaptiveCorpus<[char; 2]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
//! <Corpus as AdaptiveCorpus<[char; 3]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
//! ```

use crate::CorpusExt;

pub mod bigrams;
pub mod monograms;
pub mod pentagrams;
pub mod quadgrams;
pub mod trigrams;

#[cfg(test)]
mod tests;

#[rustfmt::skip]
static DEBUG_TRIGRAMS: &[[char; 3]] = &[
    // ['r', 'h', 'e'],
];

#[rustfmt::skip]
static DEBUG_QUADGRAMS: &[[char; 4]] = &[
    // ['e', 'r', 'h', 'e'],
    // ['e', 'r', 'h', '†'],
    // ['r', '†', 'h', 'e'],
    // ['r', '†', 'h', '†'],
];

// # Generics

// XXX: There being two uses of "old" is confusing.

pub struct ExpansionStruct<O, N> {
    old: O,
    new: N,
    count: Option<u32>,
}

impl<O, N> std::fmt::Debug for ExpansionStruct<O, N>
where
    O: std::fmt::Debug,
    N: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExpansionStruct")
            .field("old", &self.old)
            .field("new", &self.new)
            .field("count", &self.count)
            .finish()
    }
}

impl<O, N> ExpansionStruct<O, N> {
    fn new(old: O, new: N) -> Self {
        Self {
            old,
            new,
            count: None,
        }
    }

    fn read_count(&self) -> u32 {
        self.count.unwrap_or_default()
    }

    fn set_count(&mut self, count: u32) {
        self.count = Some(count);
    }
}

#[derive(Debug)]
pub struct Expansions<N, S, L> {
    left: Option<ExpansionStruct<S, N>>,
    right: Option<ExpansionStruct<S, N>>,
    both: Option<ExpansionStruct<L, N>>,
}

trait Expand<N, S, L> {
    fn expand(&self, old: [char; 2], new: [char; 2]) -> Expansions<N, S, L>;
}

pub trait GetCount<O, N> {
    fn get_count<U: CorpusExt>(&self, corpus: &mut U) -> u32;
}

pub trait AdaptiveCorpusBase<N>: CorpusExt {
    fn adapt_boundary_ngram<O>(&mut self, exp: &mut Option<ExpansionStruct<O, N>>, bcount: u32)
    where
        ExpansionStruct<O, N>: GetCount<O, N>,
        O: std::fmt::Debug;
}

pub trait AdaptiveCorpus<N>: AdaptiveCorpusBase<N> {
    fn adapt_ngrams(&mut self, old: [char; 2], new: [char; 2]);
    fn adapt_boundary_ngrams(&mut self, old: [char; 2], new: [char; 2]);
    fn adapt_interior_ngrams(&mut self, old: [char; 2], new: [char; 2]);
    fn adapt_interior_ngram(&mut self, old_idx: usize, old_ng: &[char], new_ng: &[char], acc: &mut Vec<i32>);
}
