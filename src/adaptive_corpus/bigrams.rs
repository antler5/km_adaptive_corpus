// Copyright 2025 antlers <antlers@illucid.net>
//
// SPDX-License-Identifier: GPL-3.0-only

use crate::CorpusExt;
use crate::adaptive_corpus::*;
use kc::Corpus;

impl GetCount<[char; 3], [char; 2]> for ExpansionStruct<[char; 3], [char; 2]> {
    /// Count trigrams.
    fn get_count<U: CorpusExt>(&self, corpus: &mut U) -> u32 {
        let idx = corpus.corpus_trigram(&self.old);
        corpus.get_trigrams()[idx]
    }
}

impl GetCount<[char; 4], [char; 2]> for ExpansionStruct<[char; 4], [char; 2]> {
    /// Count quadgrams.
    fn get_count<U: CorpusExt>(&self, corpus: &mut U) -> u32 {
        let idx = corpus.corpus_quadgram(&self.old);
        corpus.get_quadgrams()[idx]
    }
}

impl<U: CorpusExt> AdaptiveCorpusBase<[char; 2]> for U {
    fn adapt_boundary_ngram<O>(
        &mut self,
        exp: &mut Option<ExpansionStruct<O, [char; 2]>>,
        bcount: u32,
    ) where
        ExpansionStruct<O, [char; 2]>: GetCount<O, [char; 2]>,
    {
        if let Some(exp) = exp {
            exp.set_count(exp.get_count(self) - bcount);

            let idx = self.corpus_bigram(&exp.new);
            self.get_bigrams()[idx] += exp.read_count();
        }
    }
}

impl Expand<[char; 2], [char; 3], [char; 4]> for [char; 2] {
    fn expand(
        &self,
        old: [char; 2],
        new: [char; 2],
    ) -> Expansions<[char; 2], [char; 3], [char; 4]> {
        let (mut left, mut right, mut both) = (None, None, None);

        // If the bigram starts with the old bigram suffix, left
        if self[0] == old[1] {
            left = Some(ExpansionStruct::new(
                [old[0], self[0], self[1]],
                [new[1], self[1]],
            ));
        }

        // If the bigram ends with the old bigram prefix, right
        if self[1] == old[0] {
            right = Some(ExpansionStruct::new(
                [self[0], self[1], old[1]],
                [self[0], new[0]],
            ));

            // If both, both
            if let Some(ref left) = left {
                both = Some(ExpansionStruct::new(
                    [left.old[0], left.old[1], left.old[2], old[1]],
                    [left.new[0], new[0]],
                ));
            }
        }

        Expansions { left, right, both }
    }
}

/// Methods for adapting bigram frequencies to reflect bigram substitutions.
///
/// # Debugging
/// - See commented code in module `test::si_compare_all_bigrams`
/// ```ignore
/// if tg == &['†', 'a', 'h'] { ... }
/// ```
impl AdaptiveCorpus<[char; 2]> for Corpus {
    fn adapt_ngrams(&mut self, old: [char; 2], new: [char; 2]) {
        // self.adapt_interior_ngrams(old, new);
        <Corpus as AdaptiveCorpus<[char; 2]>>::adapt_interior_ngrams(self, old, new);
        // self.adapt_boundary_ngrams(old, new);
        <Corpus as AdaptiveCorpus<[char; 2]>>::adapt_boundary_ngrams(self, old, new);
    }

    fn adapt_boundary_ngrams(&mut self, old: [char; 2], new: [char; 2]) {
        let num_bigrams = self.get_bigrams().len();
        for i in 0..num_bigrams {
            let bg = self.uncorpus_bigram(i);
            let mut exps = [bg[0], bg[1]].expand(old, new);

            self.adapt_boundary_ngram(&mut exps.both, 0);
            let bcount = exps.sum(&[ExpansionKind::Both]);
            self.adapt_boundary_ngram(&mut exps.left, bcount);
            self.adapt_boundary_ngram(&mut exps.right, bcount);

            let sum = exps.sum(&[ExpansionKind::Left, ExpansionKind::Right, ExpansionKind::Both]);

            self.bigrams[i] -= sum;
        }
    }

    fn adapt_interior_ngrams(&mut self, old: [char; 2], new: [char; 2]) {
        let num_bigrams = self.get_bigrams().len();
        let mut acc = vec![0; num_bigrams];

        for i in 0..num_bigrams {
            if self.bigrams[i] == 0 {
                continue;
            }
            let bg = self.uncorpus_bigram(i);
            if bg[0] == old[0] && bg[1] == old[1] {
                // self.adapt_interior_ngram(i, &bg[..], &[new[0], new[1]]);
                #[rustfmt::skip]
                <Corpus as AdaptiveCorpus<[char; 2]>>::adapt_interior_ngram(self, i, &bg[..], &[new[0], new[1]], &mut acc);
            }
        }

        for (a, b) in self.get_bigrams().iter_mut().zip(&acc) {
            *a = a.checked_add_signed(*b).expect("Overflow!");
        }
    }

    fn adapt_interior_ngram(
        &mut self,
        old_idx: usize,
        old_ng: &[char],
        new_ng: &[char],
        acc: &mut Vec<i32>,
    ) {
        let _ = old_ng;

        let freq = self.get_bigrams()[old_idx];
        acc[old_idx] = acc[old_idx].checked_sub_unsigned(freq).expect("Overflow!");

        let new_idx = self.corpus_bigram(&[new_ng[0], new_ng[1]]);
        acc[new_idx] = acc[new_idx].checked_add_unsigned(freq).expect("Overflow!");
    }
}
