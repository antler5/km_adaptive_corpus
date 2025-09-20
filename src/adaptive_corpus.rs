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
//! corpus.adapt_trigrams(['h', 'e'], ['h', '†']);
//! ```

use crate::CorpusExt;
use kc::Corpus;

#[cfg(test)]
mod tests;

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

trait GetCount<O, N> {
    fn get_count<U: CorpusExt>(&self, corpus: &mut U) -> u32;
}

pub struct Expansions<N, S, L>
{
    left: Option<ExpansionStruct<S, N>>,
    right: Option<ExpansionStruct<S, N>>,
    both: Option<ExpansionStruct<L, N>>,
}

impl GetCount<[char; 4], [char; 3]> for ExpansionStruct<[char; 4], [char; 3]> {
    /// Count quadgrams.
    fn get_count<U: CorpusExt>(&self, corpus: &mut U) -> u32 {
        let idx = corpus.corpus_quadgram(&self.old);
        corpus.get_quadgrams()[idx]
    }
}
impl GetCount<[char; 5], [char; 3]> for ExpansionStruct<[char; 5], [char; 3]> {
    /// Count pentagrams.
    fn get_count<U: CorpusExt>(&self, corpus: &mut U) -> u32 {
        let idx = corpus.corpus_pentagram(&self.old);
        corpus.get_pentagrams()[idx]
    }
}

/// Generic methods for adapting ngram frequencies to reflect bigram substitutions.
pub trait AdaptiveCorpusBase<N> {
    fn adapt_interior_ngram<O>(&mut self, exp: &mut Option<ExpansionStruct<O, N>>, bcount: u32)
    where
        ExpansionStruct<O, N>: GetCount<O, N>;
}

impl<U: CorpusExt> AdaptiveCorpusBase<[char;3]> for U {
    fn adapt_interior_ngram<O>(&mut self, exp: &mut Option<ExpansionStruct<O, [char; 3]>>, bcount: u32)
    where
        ExpansionStruct<O, [char; 3]>: GetCount<O, [char; 3]>,
    {
        if let Some(exp) = exp {
            exp.set_count(exp.get_count(self) - bcount);

            let idx = self.corpus_trigram(&exp.new);
            self.get_trigrams()[idx] += exp.read_count();

            // Skipgrams
            // XXX: Half-assed, assumes all corpus chars are valid.
            let new_sg = &[exp.new[0], exp.new[2]];
            let new_sg_idx = self.corpus_bigram(new_sg);
            self.get_skipgrams()[new_sg_idx] += exp.read_count();
        }
    }
}

trait Expand<N, S, L> {
    fn expand(&self, old: [char; 2], new: [char; 2]) -> Expansions<N, S, L>;
}

impl Expand<[char; 3], [char; 4], [char; 5]> for [char; 3] {
    fn expand(&self, old: [char; 2], new: [char; 2]) -> Expansions<[char; 3], [char; 4], [char; 5]> {
        let (mut left, mut right, mut both) = (None, None, None);

        // If the trigram starts with the old bigram suffix, left
        if self[0] == old[1] {
            left = Some(ExpansionStruct::new(
                [old[0], self[0], self[1], self[2]],
                [new[1], self[1], self[2]],
            ));
        }

        // If the trigram ends with the old bigram prefix, right
        if self[2] == old[0] {
            right = Some(ExpansionStruct::new(
                [self[0], self[1], self[2], old[1]],
                [self[0], self[1], new[0]],
            ));

            // If both, both
            if let Some(ref left) = left {
                both = Some(ExpansionStruct::new(
                    [left.old[0], left.old[1], left.old[2], left.old[3], old[1]],
                    [left.new[0], left.new[1], new[0]],
                ));
            }
        }

        Expansions { left, right, both }
    }
}

/// Specialized methods for adapting ngram frequencies to reflect bigram substitutions.
pub trait AdaptiveCorpus: CorpusExt {
    fn adapt_trigrams(&mut self, old: [char; 2], new: [char; 2]);
    fn adapt_interior_trigrams(&mut self, old: [char; 2], new: [char; 2]);
    fn adapt_boundary_trigrams(&mut self, old: [char; 2], new: [char; 2]);
    fn adapt_boundary_trigram(&mut self, old_idx: usize, old_tg: &[char], new_tg: &[char; 3]);
}

/// Methods for adapting ngram frequencies to reflect bigram substitutions.
///
/// # Debugging
/// - See commented code in module `test::si_compare_all_trigrams`
/// ```ignore
/// if tg == &['†', 'a', 'h'] { ... }
/// ```
impl AdaptiveCorpus for Corpus
{
    fn adapt_trigrams(&mut self, old: [char; 2], new: [char; 2]) {
        self.adapt_interior_trigrams(old, new);
        self.adapt_boundary_trigrams(old, new);
    }

    fn adapt_interior_trigrams(&mut self, old: [char; 2], new: [char; 2]) {
        let num_trigrams = self.get_trigrams().len();
        for i in 0..num_trigrams {
            let tg = self.uncorpus_trigram(i);

            let mut exps = [tg[0], tg[1], tg[2]].expand(old, new);

            macro_rules! sum {
                ($($tg:expr),*) => {
                    [$($tg.as_ref()
                          .and_then(|x| Some(x.read_count()))
                          .unwrap_or(0)
                     ),*
                    ].into_iter().sum()
                }
            }

            self.adapt_interior_ngram(&mut exps.both, 0);
            let bcount = sum!(exps.both);
            self.adapt_interior_ngram(&mut exps.left, bcount);
            self.adapt_interior_ngram(&mut exps.right, bcount);

            let sum: u32 = sum!(exps.left, exps.right, exps.both);

            self.trigrams[i] -= sum;

            // Skipgrams
            // XXX: Half-assed, assumes all corpus chars were valid.
            let sg = &[tg[0], tg[2]];
            let idx = self.corpus_bigram(sg);
            self.skipgrams[idx] -= sum;
        }
    }

    fn adapt_boundary_trigrams(&mut self, old: [char; 2], new: [char; 2]) {
        let num_trigrams = self.get_trigrams().len();
        for i in 0..num_trigrams {
            let tg = self.uncorpus_trigram(i);
            if tg[0] == old[0] && tg[1] == old[1] {
                self.adapt_boundary_trigram(i, &tg[..], &[new[0], new[1], tg[2]]);
            }
            if tg[1] == old[0] && tg[2] == old[1] {
                self.adapt_boundary_trigram(i, &tg[..], &[tg[0], new[0], new[1]]);
            }
        }
    }

    fn adapt_boundary_trigram(&mut self, old_idx: usize, old_tg: &[char], new_tg: &[char; 3]) {
        let freq = self.get_trigrams()[old_idx];
        self.get_trigrams()[old_idx] -= freq;

        let new_idx = self.corpus_trigram(new_tg);
        self.get_trigrams()[new_idx] += freq;

        // Skipgrams
        // XXX: Half-assed skipgrams, assumes all corpus chars were valid.
        let old_idx = self.corpus_bigram(&[old_tg[0], old_tg[2]]);
        self.get_skipgrams()[old_idx] -= freq;
        let new_sg = &[new_tg[0], new_tg[2]];
        let new_idx = self.corpus_bigram(new_sg);
        self.get_skipgrams()[new_idx] += freq;
    }
}
