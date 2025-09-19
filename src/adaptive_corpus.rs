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

/// Specialized methods implemented per expansion-length to use generic [Expansion] methods.
pub trait ExpansionBase<O> {
    fn get_count(&self, corpus: &Corpus) -> u32;
}

/// Generic methods implemented per ngram-length but abstracted over expansion-length via
/// [ExpansionBase].
pub trait Expansion<O, N>: ExpansionBase<N> {
    fn new(old: O, new: N) -> Self;
    fn add_count(&mut self, corpus: &mut Corpus);
    fn read_count(&self) -> u32;
    fn set_count(&mut self, count: u32);
}

/// Expansions derived from a trigram.
pub struct TrigramExpansions {
    left: Option<TrigramExpansion<[char; 4]>>,
    right: Option<TrigramExpansion<[char; 4]>>,
    both: Option<TrigramExpansion<[char; 5]>>,
}

/// Expansion derived from a trigram.
pub struct TrigramExpansion<O> {
    /// Four to five character expansion derived from a modified trigram.
    old: O,
    /// New trigram post-transformation.
    new: [char; 3],
    /// Frequency of `old` in `corpus`.
    count: Option<u32>,
}

impl ExpansionBase<[char; 3]> for TrigramExpansion<[char; 4]> {
    /// Count quadgrams.
    fn get_count(&self, corpus: &Corpus) -> u32 {
        corpus.quadgrams[corpus.corpus_quadgram(&self.old)]
    }
}

impl ExpansionBase<[char; 3]> for TrigramExpansion<[char; 5]> {
    /// Count pentagrams.
    fn get_count(&self, corpus: &Corpus) -> u32 {
        corpus.pentagrams[corpus.corpus_pentagram(&self.old)]
    }
}

/// Generic methods for trigram-expansions abstracted over [ExpansionBase].
impl<O> Expansion<O, [char; 3]> for TrigramExpansion<O>
where
    TrigramExpansion<O>: ExpansionBase<[char; 3]>,
{
    fn new(old: O, new: [char; 3]) -> Self {
        TrigramExpansion {
            old,
            new,
            count: None,
        }
    }

    /// Add `self.count` to trigram `self.new` (and eqiv. skipgram).
    fn add_count(&mut self, corpus: &mut Corpus) {
        let idx = corpus.corpus_trigram(&self.new);
        corpus.trigrams[idx] += self.read_count();

        // Skipgrams
        // XXX: Half-assed, assumes all corpus chars are valid.
        let new_sg = &[self.new[0], self.new[2]];
        let new_sg_idx = corpus.corpus_bigram(new_sg);
        corpus.skipgrams[new_sg_idx] += self.read_count();
    }

    fn read_count(&self) -> u32 {
        self.count.unwrap_or_default()
    }

    fn set_count(&mut self, count: u32) {
        self.count = Some(count);
    }
}

/// Interface for adapting ngram frequencies to reflect bigram substitutions.
pub trait AdaptiveCorpus {
    fn adapt_trigrams(&mut self, old: [char; 2], new: [char; 2]);
    fn expand_trigram(tg: &[char], old: [char; 2], new: [char; 2]) -> TrigramExpansions;
    fn adapt_interior_trigram<T, O, N>(&mut self, exp: &mut Option<T>, bcount: u32)
    where
        T: Expansion<O, N>;
    fn adapt_interior_trigrams(&mut self, old: [char; 2], new: [char; 2]);
    fn adapt_boundary_trigram(&mut self, old_idx: usize, old_ng: &[char], new_ng: &[char; 3]);
    fn adapt_boundary_trigrams(&mut self, old: [char; 2], new: [char; 2]);
}

/// Methods for adapting ngram frequencies to reflect bigram substitutions.
///
/// # Debugging
/// - See commented code in module `test::si_compare_all_trigrams`
/// ```ignore
/// if tg == &['†', 'a', 'h'] { ... }
/// ```
impl AdaptiveCorpus for Corpus {
    fn adapt_trigrams(&mut self, old: [char; 2], new: [char; 2]) {
        self.adapt_interior_trigrams(old, new);
        self.adapt_boundary_trigrams(old, new);
    }

    fn expand_trigram(tg: &[char], old: [char; 2], new: [char; 2]) -> TrigramExpansions {
        let (mut left, mut right, mut both) = (None, None, None);

        // If the trigram starts with the old bigram suffix, left
        if tg[0] == old[1] {
            left = Some(TrigramExpansion::new(
                [old[0], tg[0], tg[1], tg[2]],
                [new[1], tg[1], tg[2]],
            ));
        }

        // If the trigram ends with the old bigram prefix, right
        if tg[2] == old[0] {
            right = Some(TrigramExpansion::new(
                [tg[0], tg[1], tg[2], old[1]],
                [tg[0], tg[1], new[0]],
            ));

            // If both, both
            if let Some(ref left) = left {
                both = Some(TrigramExpansion::new(
                    [left.old[0], left.old[1], left.old[2], left.old[3], old[1]],
                    [left.new[0], left.new[1], new[0]],
                ));
            }
        }

        TrigramExpansions { left, right, both }
    }

    fn adapt_interior_trigram<T, O, N>(&mut self, exp: &mut Option<T>, bcount: u32)
    where
        T: Expansion<O, N>,
    {
        if let Some(exp) = exp {
            exp.set_count(exp.get_count(self) - bcount);
            exp.add_count(self)
        }
    }

    fn adapt_interior_trigrams(&mut self, old: [char; 2], new: [char; 2]) {
        let num_trigrams = self.get_trigrams().len();
        for i in 0..num_trigrams {
            let tg = self.uncorpus_trigram(i);

            let mut exps = Corpus::expand_trigram(&tg[..], old, new);

            macro_rules! sum {
                ($($tg:expr),*) => {
                    [$($tg.as_ref()
                          .and_then(|x| Some(x.read_count()))
                          .unwrap_or(0)
                     ),*
                    ].into_iter().sum()
                }
            }

            self.adapt_interior_trigram(&mut exps.both, 0);
            let bcount = sum!(exps.both);
            self.adapt_interior_trigram(&mut exps.left, bcount);
            self.adapt_interior_trigram(&mut exps.right, bcount);

            let sum: u32 = sum!(exps.left, exps.right, exps.both);

            self.trigrams[i] -= sum;

            // Skipgrams
            // XXX: Half-assed, assumes all corpus chars were valid.
            let sg = &[tg[0], tg[2]];
            let idx = self.corpus_bigram(sg);
            self.skipgrams[idx] -= sum;
        }
    }

    fn adapt_boundary_trigram(&mut self, old_idx: usize, old_ng: &[char], new_ng: &[char; 3]) {
        let freq = self.get_trigrams()[old_idx];
        self.get_trigrams()[old_idx] -= freq;

        let new_idx = self.corpus_trigram(new_ng);
        self.get_trigrams()[new_idx] += freq;

        // Skipgrams
        // XXX: Half-assed skipgrams, assumes all corpus chars were valid.
        let old_idx = self.corpus_bigram(&[old_ng[0], old_ng[2]]);
        self.get_skipgrams()[old_idx] -= freq;
        let new_sg = &[new_ng[0], new_ng[2]];
        let new_idx = self.corpus_bigram(new_sg);
        self.get_skipgrams()[new_idx] += freq;
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
}
