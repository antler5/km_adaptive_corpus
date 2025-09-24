// Copyright 2025 antlers <antlers@illucid.net>
//
// SPDX-License-Identifier: GPL-3.0-only

use crate::CorpusExt;
use crate::adaptive_corpus::*;
use kc::Corpus;

use tracing::debug;

impl GetCount<[char; 5], [char; 4]> for ExpansionStruct<[char; 5], [char; 4]> {
    /// Count pentagrams.
    fn get_count<U: CorpusExt>(&self, corpus: &mut U) -> u32 {
        let idx = corpus.corpus_pentagram(&self.old);
        corpus.get_pentagrams()[idx]
    }
}

impl GetCount<[char; 6], [char; 4]> for ExpansionStruct<[char; 6], [char; 4]> {
    /// "Count" hexagrams.
    fn get_count<U: CorpusExt>(&self, corpus: &mut U) -> u32 {
        #[cfg(feature = "synth-large-ngrams")]
        {
            let prefix = &[self.old[0], self.old[1], self.old[2], self.old[3], self.old[4]];
            let suffix = &[self.old[1], self.old[2], self.old[3], self.old[4], self.old[5]];
            let prefix_idx = corpus.corpus_pentagram(prefix);
            let suffix_idx = corpus.corpus_pentagram(suffix);
            let pgs = corpus.get_pentagrams();

            return (pgs[prefix_idx] + pgs[suffix_idx]) / 2
        }

        #[cfg(not(feature = "synth-large-ngrams"))]
        Default::default()
    }
}

impl<U: CorpusExt> AdaptiveCorpusBase<[char; 4]> for U {
    fn adapt_boundary_ngram<O>(
        &mut self,
        exp: &mut Option<ExpansionStruct<O, [char; 4]>>,
        bcount: u32,
    ) where
        ExpansionStruct<O, [char; 4]>: GetCount<O, [char; 4]>,
        O: std::fmt::Debug,
    {
        // XXX: if let Some(exp) = exp && exp.old.len() < 6 {
        if let Some(exp) = exp {
            exp.set_count(exp.get_count(self) - bcount);

            let idx = self.corpus_quadgram(&exp.new);
            if DEBUG_QUADGRAMS.contains(&exp.new) {
                debug!(?exp, freq_pre = self.get_quadgrams()[idx], bcount);
            }
            self.get_quadgrams()[idx] += exp.read_count();
        }
    }
}

impl Expand<[char; 4], [char; 5], [char; 6]> for [char; 4] {
    fn expand(
        &self,
        old: [char; 2],
        new: [char; 2],
    ) -> Expansions<[char; 4], [char; 5], [char; 6]> {
        let (mut left, mut right, mut both) = (None, None, None);

        let mut qg = self.clone();
        if qg[0] == old[0] && qg[1] == old[1] && qg[2] == old[0] && qg[3] == old[1] {
            // hehe
            qg = [new[0], new[1], new[0], new[1]];
        } else if qg[0] == old[1] && qg[1] == old[0] && qg[2] == old[1] && qg[3] == old[0] {
            // eheh
            qg = [new[1], new[0], new[1], new[0]];
        } else if qg[0] == old[0] && qg[1] == old[1] {
            // he**
            qg = [new[0], new[1], qg[2], qg[3]];
        } else if qg[1] == old[0] && qg[2] == old[1] {
            // *he*
            qg = [qg[0], new[0], new[1], qg[3]];
        } else if qg[2] == old[0] && qg[3] == old[1] {
            // **he
            qg = [qg[0], qg[1], new[0], new[1]];
        }

        // If the quadgram starts with the old bigram suffix, left
        if self[0] == old[1] {
            left = Some(ExpansionStruct::new(
                [old[0], qg[0], qg[1], qg[2], qg[3]],
                [new[1], qg[1], qg[2], qg[3]],
            ));
        }

        // If the quadgram ends with the old bigram prefix, right
        if self[3] == old[0] {
            right = Some(ExpansionStruct::new(
                [qg[0], qg[1], qg[2], qg[3], old[1]],
                [qg[0], qg[1], qg[2], new[0]],
            ));

            // If both, both
            if let Some(ref left) = left {
                both = Some(ExpansionStruct::new(
                    [
                        left.old[0],
                        left.old[1],
                        left.old[2],
                        left.old[3],
                        left.old[4],
                        old[1],
                    ],
                    [left.new[0], left.new[1], left.new[2], new[0]],
                ));
            }
        }

        Expansions { left, right, both }
    }
}

/// Methods for adapting quadgram frequencies to reflect bigram substitutions.
impl AdaptiveCorpus<[char; 4]> for Corpus {
    fn adapt_ngrams(&mut self, old: [char; 2], new: [char; 2]) {
        // self.adapt_interior_ngrams(old, new);
        <Corpus as AdaptiveCorpus<[char; 4]>>::adapt_interior_ngrams(self, old, new);
        // self.adapt_boundary_ngrams(old, new);
        <Corpus as AdaptiveCorpus<[char; 4]>>::adapt_boundary_ngrams(self, old, new);
    }

    fn adapt_boundary_ngrams(&mut self, old: [char; 2], new: [char; 2]) {
        let num_quadgrams = self.get_quadgrams().len();
        for i in 0..num_quadgrams {
            let qg = self.uncorpus_quadgram(i);
            let mut exps = [qg[0], qg[1], qg[2], qg[3]].expand(old, new);

            macro_rules! sum {
                ($($qg:expr),*) => {
                    [$($qg.as_ref()
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

            if DEBUG_QUADGRAMS.contains(&[qg[0], qg[1], qg[2], qg[3]]) {
                debug!(?exps, sum, ?qg, freq_pre = self.quadgrams[i]);
            }

            self.quadgrams[i] -= sum;

            if DEBUG_QUADGRAMS.contains(&[qg[0], qg[1], qg[2], qg[3]]) {
                debug!(?exps, sum, ?qg, freq_post = self.quadgrams[i]);
            }
        }
    }

    fn adapt_interior_ngrams(&mut self, old: [char; 2], new: [char; 2]) {
        let num_quadgrams = self.get_quadgrams().len();
        for i in 0..num_quadgrams {
            let qg = self.uncorpus_quadgram(i);

            // XXX: Probably not correct for replacing repeats
            if qg[0] == old[0] && qg[1] == old[1] && qg[2] == old[0] && qg[3] == old[1] {
                #[rustfmt::skip]
                <Corpus as AdaptiveCorpus<[char; 4]>>::adapt_interior_ngram(self, i, &qg[..], &[new[0], new[1], new[0], new[1]]);
            } else {
                if qg[0] == old[0] && qg[1] == old[1] {
                    #[rustfmt::skip]
                    <Corpus as AdaptiveCorpus<[char; 4]>>::adapt_interior_ngram(self, i, &qg[..], &[new[0], new[1], qg[2], qg[3]]);
                }
                if qg[1] == old[0] && qg[2] == old[1] {
                    #[rustfmt::skip]
                    <Corpus as AdaptiveCorpus<[char; 4]>>::adapt_interior_ngram(self, i, &qg[..], &[qg[0], new[0], new[1], qg[3]]);
                }
                if qg[2] == old[0] && qg[3] == old[1] {
                    #[rustfmt::skip]
                    <Corpus as AdaptiveCorpus<[char; 4]>>::adapt_interior_ngram(self, i, &qg[..], &[qg[0], qg[1], new[0], new[1]]);
                }
            }
        }
    }

    fn adapt_interior_ngram(&mut self, old_idx: usize, old_ng: &[char], new_ng: &[char]) {
        let freq = self.get_quadgrams()[old_idx];
        if DEBUG_QUADGRAMS.contains(&[old_ng[0], old_ng[1], old_ng[2], old_ng[3]])
            || DEBUG_QUADGRAMS.contains(&[new_ng[0], new_ng[1], new_ng[2], new_ng[3]])
        {
            debug!(?old_ng, freq_pre = freq);
        }
        self.get_quadgrams()[old_idx] -= freq;

        let new_idx = self.corpus_quadgram(&[new_ng[0], new_ng[1], new_ng[2], new_ng[3]]);
        if DEBUG_QUADGRAMS.contains(&[old_ng[0], old_ng[1], old_ng[2], old_ng[3]])
            || DEBUG_QUADGRAMS.contains(&[new_ng[0], new_ng[1], new_ng[2], new_ng[3]])
        {
            debug!(?new_ng, freq_pre = self.get_quadgrams()[new_idx]);
        }
        self.get_quadgrams()[new_idx] += freq;
        if DEBUG_QUADGRAMS.contains(&[old_ng[0], old_ng[1], old_ng[2], old_ng[3]])
            || DEBUG_QUADGRAMS.contains(&[new_ng[0], new_ng[1], new_ng[2], new_ng[3]])
        {
            debug!(?new_ng, freq_post = self.get_quadgrams()[new_idx]);
        }
    }
}
