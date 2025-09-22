// Copyright 2025 antlers <antlers@illucid.net>
//
// SPDX-License-Identifier: GPL-3.0-only

use crate::CorpusExt;
use crate::adaptive_corpus::*;
use kc::Corpus;

impl GetCount<[char; 6], [char; 5]> for ExpansionStruct<[char; 6], [char; 5]> {
    /// "Count" hexagrams.
    fn get_count<U: CorpusExt>(&self, corpus: &mut U) -> u32 {
        Default::default()
    }
}

impl GetCount<[char; 7], [char; 5]> for ExpansionStruct<[char; 7], [char; 5]> {
    /// "Count" septegrams.
    fn get_count<U: CorpusExt>(&self, corpus: &mut U) -> u32 {
        Default::default()
    }
}

impl<U: CorpusExt> AdaptiveCorpusBase<[char; 5]> for U {
    fn adapt_boundary_ngram<O>(
        &mut self,
        exp: &mut Option<ExpansionStruct<O, [char; 5]>>,
        bcount: u32,
    ) where
        ExpansionStruct<O, [char; 5]>: GetCount<O, [char; 5]>,
    {
        // XXX: if let Some(exp) = exp && exp.old.len() < 6 {
        if let Some(exp) = exp {
            exp.set_count(exp.get_count(self) - bcount);

            let idx = self.corpus_pentagram(&exp.new);
            self.get_pentagrams()[idx] += exp.read_count();
        }
    }
}

impl Expand<[char; 5], [char; 6], [char; 7]> for [char; 5] {
    fn expand(
        &self,
        old: [char; 2],
        new: [char; 2],
    ) -> Expansions<[char; 5], [char; 6], [char; 7]> {
        let (mut left, mut right, mut both) = (None, None, None);

        // If the pentagram starts with the old bigram suffix, left
        if self[0] == old[1] {
            left = Some(ExpansionStruct::new(
                [0 as char; 6],
                [new[1], self[1], self[2], self[3], self[4]],
            ));
        }

        // If the pentagram ends with the old bigram prefix, right
        if self[3] == old[0] {
            right = Some(ExpansionStruct::new(
                [0 as char; 6],
                [self[0], self[1], self[2], self[3], new[0]],
            ));

            // If both, both
            if let Some(ref left) = left {
                both = Some(ExpansionStruct::new(
                    [0 as char; 7],
                    [left.new[0], left.new[1], left.new[2], left.new[3], new[0]],
                ));
            }
        }

        Expansions { left, right, both }
    }
}

/// Methods for adapting pentagram frequencies to reflect bigram substitutions.
impl AdaptiveCorpus<[char; 5]> for Corpus {
    fn adapt_ngrams(&mut self, old: [char; 2], new: [char; 2]) {
        // self.adapt_interior_ngrams(old, new);
        <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_interior_ngrams(self, old, new);
    }

    fn adapt_boundary_ngrams(&mut self, old: [char; 2], new: [char; 2]) {
        let num_pentagrams = self.get_pentagrams().len();
        for i in 0..num_pentagrams {
            let pg = self.uncorpus_pentagram(i);

            let mut exps = [pg[0], pg[1], pg[2], pg[3], pg[4]].expand(old, new);

            macro_rules! sum {
                ($($pg:expr),*) => {
                    [$($pg.as_ref()
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

            self.pentagrams[i] -= sum;
        }
    }

    fn adapt_interior_ngrams(&mut self, old: [char; 2], new: [char; 2]) {
        let num_pentagrams = self.get_pentagrams().len();
        for i in 0..num_pentagrams {
            let pg = self.uncorpus_pentagram(i);

            // XXX: Probably not correct for replacing repeats
            if pg[0] == old[0] && pg[1] == old[1] && pg[2] == old[0] && pg[3] == old[1] {
                <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_interior_ngram(
                    self,
                    i,
                    &pg[..],
                    &[new[0], new[1], new[0], new[1], pg[4]],
                );
            } else if pg[1] == old[0] && pg[2] == old[1] && pg[3] == old[0] && pg[4] == old[1] {
                <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_interior_ngram(
                    self,
                    i,
                    &pg[..],
                    &[pg[0], new[0], new[1], new[0], new[1]],
                );
            } else {
                if pg[0] == old[0] && pg[1] == old[1] {
                    <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_interior_ngram(
                        self,
                        i,
                        &pg[..],
                        &[new[0], new[1], pg[2], pg[3], pg[4]],
                    );
                }
                if pg[1] == old[0] && pg[2] == old[1] {
                    <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_interior_ngram(
                        self,
                        i,
                        &pg[..],
                        &[pg[0], new[0], new[1], pg[3], pg[4]],
                    );
                }
                if pg[2] == old[0] && pg[3] == old[1] {
                    <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_interior_ngram(
                        self,
                        i,
                        &pg[..],
                        &[pg[0], pg[1], new[0], new[1], pg[4]],
                    );
                }
                if pg[3] == old[0] && pg[4] == old[1] {
                    <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_interior_ngram(
                        self,
                        i,
                        &pg[..],
                        &[pg[0], pg[1], pg[2], new[0], new[1]],
                    );
                }
            }
        }
    }

    fn adapt_interior_ngram(&mut self, old_idx: usize, old_ng: &[char], new_ng: &[char]) {
        let freq = self.get_pentagrams()[old_idx];
        self.get_pentagrams()[old_idx] -= freq;

        let new_idx =
            self.corpus_pentagram(&[new_ng[0], new_ng[1], new_ng[2], new_ng[3], new_ng[4]]);
        self.get_pentagrams()[new_idx] += freq;
    }
}
