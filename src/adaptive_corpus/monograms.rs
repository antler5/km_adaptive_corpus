// Copyright 2025 antlers <antlers@illucid.net>
//
// SPDX-License-Identifier: GPL-3.0-only

use crate::CorpusExt;
use crate::adaptive_corpus::*;
use kc::Corpus;

impl GetCount<[char; 2], [char; 1]> for ExpansionStruct<[char; 2], [char; 1]> {
    /// Count bigrams.
    fn get_count<U: CorpusExt>(&self, corpus: &mut U) -> u32 {
        let idx = corpus.corpus_bigram(&self.old);
        corpus.get_bigrams()[idx]
    }
}

impl GetCount<[char; 3], [char; 1]> for ExpansionStruct<[char; 3], [char; 1]> {
    /// Count trigrams.
    fn get_count<U: CorpusExt>(&self, corpus: &mut U) -> u32 {
        let idx = corpus.corpus_trigram(&self.old);
        corpus.get_trigrams()[idx]
    }
}

impl<U: CorpusExt> AdaptiveCorpusBase<[char; 1]> for U {
    fn adapt_boundary_ngram<O>(
        &mut self,
        exp: &mut Option<ExpansionStruct<O, [char; 1]>>,
        bcount: u32,
    ) where
        ExpansionStruct<O, [char; 1]>: GetCount<O, [char; 1]>,
    {
        if let Some(exp) = exp {
            exp.set_count(exp.get_count(self) - bcount);

            let idx = self.corpus_char(&exp.new);
            self.get_chars()[idx] += exp.read_count();
        }
    }
}

impl Expand<[char; 1], [char; 2], [char; 3]> for [char; 1] {
    fn expand(
        &self,
        old: [char; 2],
        new: [char; 2],
    ) -> Expansions<[char; 1], [char; 2], [char; 3]> {
        let (mut left, mut right, mut both) = (None, None, None);

        // If the char starts with the old char suffix, left
        if self[0] == old[1] {
            left = Some(ExpansionStruct::new([old[0], self[0]], [new[1]]));
        }

        // If the char ends with the old char prefix, right
        if self[0] == old[0] {
            right = Some(ExpansionStruct::new([self[0], old[1]], [new[0]]));

            // If both, both
            if let Some(ref left) = left {
                both = Some(ExpansionStruct::new(
                    [left.old[0], left.old[1], old[1]],
                    [new[0]],
                ));
            }
        }

        Expansions { left, right, both }
    }
}

/// Methods for adapting char frequencies to reflect char substitutions.
///
/// # Debugging
/// - See commented code in module `test::si_compare_all_chars`
/// ```ignore
/// if tg == &['†', 'a', 'h'] { ... }
/// ```
impl AdaptiveCorpus<[char; 1]> for Corpus {
    fn adapt_ngrams(&mut self, old: [char; 2], new: [char; 2]) {
        // self.adapt_boundary_ngrams(old, new);
        <Corpus as AdaptiveCorpus<[char; 1]>>::adapt_boundary_ngrams(self, old, new);
    }

    fn adapt_boundary_ngrams(&mut self, old: [char; 2], new: [char; 2]) {
        let num_chars = self.get_chars().len();
        for i in 0..num_chars {
            let c = self.uncorpus_unigram(i);
            let mut exps = [c].expand(old, new);

            macro_rules! sum {
                ($($c:expr),*) => {
                    [$($c.as_ref()
                         .and_then(|x| Some(x.read_count()))
                         .unwrap_or(0)
                     ),*
                    ].into_iter().sum()
                }
            }

            self.adapt_boundary_ngram(&mut exps.both, 0);
            let bcount = sum!(exps.both);
            self.adapt_boundary_ngram(&mut exps.left, bcount);
            self.adapt_boundary_ngram(&mut exps.right, bcount);

            let sum: u32 = sum!(exps.left, exps.right, exps.both);

            self.chars[i] -= sum;
        }
    }

    fn adapt_interior_ngrams(&mut self, old: [char; 2], new: [char; 2]) {}
    fn adapt_interior_ngram(
        &mut self,
        old_idx: usize,
        old_ng: &[char],
        new_ng: &[char],
        acc: &mut Vec<i32>,
    ) {
    }
}
