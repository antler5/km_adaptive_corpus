// Copyright 2025 antlers <antlers@illucid.net>
//
// SPDX-License-Identifier: GPL-3.0-only

//! Dynamic trigram adjustments for simple magic rules.
//!
//! Right now, "simple" means adaptive-key style bigram rules like `h* -> he`.
//!
//! # Examples
//!
//! ```
//! use std::fs;
//! use kc::Corpus;
//! use km_adaptive_corpus::AdaptiveCorpus;
//!
//! let b = fs::read("./corpora/shai-iweb.corpus").unwrap();
//! let mut corpus: Corpus = rmp_serde::from_slice(&b).unwrap();
//! corpus.adapt_ngrams(['h', 'e'], ['h', '†']);
//! ```

use crate::CorpusExt;

pub mod trigrams;
pub mod bigrams;

#[cfg(test)]
mod tests;

// # Generics

pub struct ExpansionStruct<O, N> {
    old: O,
    new: N,
    count: Option<u32>,
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

pub trait GetCount<O, N> {
    fn get_count<U: CorpusExt>(&self, corpus: &mut U) -> u32;
}

pub struct Expansions<N, S, L> {
    left: Option<ExpansionStruct<S, N>>,
    right: Option<ExpansionStruct<S, N>>,
    both: Option<ExpansionStruct<L, N>>,
}

trait Expand<N, S, L> {
    fn expand(&self, old: [char; 2], new: [char; 2]) -> Expansions<N, S, L>;
}

pub trait AdaptiveCorpusBase<N>: CorpusExt {
    fn adapt_boundary_ngram<O>(&mut self, exp: &mut Option<ExpansionStruct<O, N>>, bcount: u32)
    where
        ExpansionStruct<O, N>: GetCount<O, N>;
}

pub trait AdaptiveCorpus<N>: AdaptiveCorpusBase<N> {
    fn adapt_ngrams(&mut self, old: [char; 2], new: [char; 2]);
    fn adapt_boundary_ngrams(&mut self, old: [char; 2], new: [char; 2]);
    fn adapt_interior_ngrams(&mut self, old: [char; 2], new: [char; 2]);
    fn adapt_interior_ngram(&mut self, old_idx: usize, old_ng: &[char], new_ng: &[char]);
}
