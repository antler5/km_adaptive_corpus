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

pub mod bigrams;
pub mod monograms;
pub mod pentagrams;
pub mod quadgrams;
pub mod trigrams;

use crate::CorpusExt;

#[cfg(test)]
mod tests;

use std::fmt::Debug;

// # Generics

// XXX: There being two uses of "old" is confusing.

/// # Generics
/// - `O`: Old, the length of the source ngram.
/// - `N`: New, the length of the replacement ngram.
#[derive(Debug)]
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

#[derive(PartialEq)]
enum ExpansionKind {
    Left,
    Right,
    Both,
}

/// # Generics
/// - `N`: New, the length of the replacement ngram.
/// - `S`: Short, the length of a `left` or `right` expansion.
/// - `L`: Long, the length of a `both` expansion.
#[derive(Debug)]
pub struct Expansions<N, S, L> {
    left: Option<ExpansionStruct<S, N>>,
    right: Option<ExpansionStruct<S, N>>,
    both: Option<ExpansionStruct<L, N>>,
}

impl<O, N> From<&ExpansionStruct<O, N>> for u32 {
    fn from(exp: &ExpansionStruct<O, N>) -> Self {
        exp.read_count()
    }
}

impl<N, S, L> Expansions<N, S, L> {
    fn sum(&self, kinds: &[ExpansionKind]) -> u32 {
        fn unpack<O, N>(x: &Option<ExpansionStruct<O, N>>) -> Option<u32> {
            x.as_ref().map_or(Some(0), |x| Some(x.into()))
        }
        kinds
            .iter()
            .filter_map(|k| -> Option<u32> {
                match k {
                    ExpansionKind::Left => unpack(&self.left),
                    ExpansionKind::Right => unpack(&self.right),
                    ExpansionKind::Both => unpack(&self.both),
                }
            })
            .sum()
    }
}

/// # Generics
/// - `N`: New, the length of the replacement ngram.
/// - `S`: Short, the length of a `left` or `right` expansion.
/// - `L`: Long, the length of a `both` expansion.
trait Expand<N, S, L> {
    fn expand(&self, old: [char; 2], new: [char; 2]) -> Expansions<N, S, L>;
}

pub trait GetCount<O, N> {
    fn get_count<U: CorpusExt>(&self, corpus: &mut U) -> u32;
}

pub trait AdaptiveCorpusBase<N>: CorpusExt {
    fn adapt_boundary_ngram<O: Debug>(
        &mut self,
        exp: &mut Option<ExpansionStruct<O, N>>,
        bcount: u32,
    ) where
        ExpansionStruct<O, N>: GetCount<O, N>;
}

pub trait AdaptiveCorpus<N>: AdaptiveCorpusBase<N> {
    fn adapt_ngrams(&mut self, old: [char; 2], new: [char; 2]);
    fn adapt_boundary_ngrams(&mut self, old: [char; 2], new: [char; 2]);
    fn adapt_interior_ngrams(&mut self, old: [char; 2], new: [char; 2]);
    fn adapt_interior_ngram(
        &mut self,
        old_idx: usize,
        old_ng: &[char],
        new_ng: &[char],
        acc: &mut Vec<i32>,
    );
}
