// Copyright 2025 antlers <antlers@illucid.net>
//
// SPDX-License-Identifier: GPL-3.0-only

use crate::CorpusExt;
use crate::adaptive_corpus::*;

use kc::Corpus;

use tracing::debug;
use tracing::instrument;

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

impl<U: CorpusExt> AdaptiveCorpusBase<[char; 3]> for U {
    #[instrument(level = "trace", skip(self))]
    fn adapt_boundary_ngram<O>(
        &mut self,
        exp: &mut Option<ExpansionStruct<O, [char; 3]>>,
        bcount: u32,
    ) where
        ExpansionStruct<O, [char; 3]>: GetCount<O, [char; 3]>,
        O: std::fmt::Debug,
    {
        if let Some(exp) = exp {
            exp.set_count(exp.get_count(self) - bcount);

            let idx = self.corpus_trigram(&exp.new);
            if DEBUG_TRIGRAMS.contains(&exp.new) {
                debug!(?exp, freq_pre = self.get_trigrams()[idx], bcount);
            }
            self.get_trigrams()[idx] += exp.read_count();

            // Skipgrams
            // XXX: Half-assed, assumes all corpus chars are valid.
            let new_sg = &[exp.new[0], exp.new[2]];
            let new_sg_idx = self.corpus_bigram(new_sg);
            self.get_skipgrams()[new_sg_idx] += exp.read_count();
        }
    }
}

impl Expand<[char; 3], [char; 4], [char; 5]> for [char; 3] {
    fn expand(
        &self,
        old: [char; 2],
        new: [char; 2],
    ) -> Expansions<[char; 3], [char; 4], [char; 5]> {
        let (mut left, mut right, mut both) = (None, None, None);

        let mut tg = self.clone();
        if tg[0] == old[0] && tg[1] == old[1] {
            // he*
            tg = [new[0], new[1], tg[2]];
        } else if tg[1] == old[0] && tg[2] == old[1] {
            // *he
            tg = [tg[0], new[0], new[1]];
        }

        // If the trigram starts with the old bigram suffix, left
        if self[0] == old[1] {
            left = Some(ExpansionStruct::new(
                [old[0], self[0], self[1], self[2]],
                [new[1], tg[1], tg[2]],
            ));
        }

        // If the trigram ends with the old bigram prefix, right
        if self[2] == old[0] {
            right = Some(ExpansionStruct::new(
                [self[0], self[1], self[2], old[1]],
                [tg[0], tg[1], new[0]],
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

/// Methods for adapting trigram frequencies to reflect bigram substitutions.
///
/// # Debugging
/// - See commented code in module `test::si_compare_all_trigrams`
/// ```ignore
/// if tg == &['†', 'a', 'h'] { ... }
/// ```
impl AdaptiveCorpus<[char; 3]> for Corpus {
    fn adapt_ngrams(&mut self, old: [char; 2], new: [char; 2]) {
        // self.adapt_interior_ngrams(old, new);
        <Corpus as AdaptiveCorpus<[char; 3]>>::adapt_interior_ngrams(self, old, new);
        // self.adapt_boundary_ngrams(old, new);
        <Corpus as AdaptiveCorpus<[char; 3]>>::adapt_boundary_ngrams(self, old, new);
    }

    #[instrument(level = "trace", skip(self))]
    fn adapt_boundary_ngrams(&mut self, old: [char; 2], new: [char; 2]) {
        let num_trigrams = self.get_trigrams().len();

        for mut i in 0..num_trigrams {
            let mut tg = self.uncorpus_trigram(i);
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

            // TODO: Change method signature to unwrap the Option here
            if exps.both.is_some() {
                self.adapt_boundary_ngram(&mut exps.both, 0)
            };
            let bcount = sum!(exps.both);
            if exps.left.is_some() {
                self.adapt_boundary_ngram(&mut exps.left, bcount)
            };
            if exps.right.is_some() {
                self.adapt_boundary_ngram(&mut exps.right, bcount)
            };

            let sum: u32 = sum!(exps.left, exps.right, exps.both);

            if DEBUG_TRIGRAMS.contains(&[tg[0], tg[1], tg[2]]) {
                debug!(?exps, ?tg, sum, freq_pre = self.trigrams[i]);
            }

            if tg[0] == old[0] && tg[1] == old[1] {
                // he*
                tg = [new[0], new[1], tg[2]].to_vec();
                i = self.corpus_trigram(&[tg[0], tg[1], tg[2]]);
            } else if tg[1] == old[0] && tg[2] == old[1] {
                // *he
                tg = [tg[0], new[0], new[1]].to_vec();
                i = self.corpus_trigram(&[tg[0], tg[1], tg[2]]);
            }

            self.get_trigrams()[i] -= sum;

            if DEBUG_TRIGRAMS.contains(&[tg[0], tg[1], tg[2]]) {
                debug!(?exps, ?tg, sum, freq_post = self.trigrams[i]);
            }

            if DEBUG_TRIGRAMS.contains(&[tg[0], tg[1], tg[2]]) {
                debug!(?exps, ?tg, sum, freq_post = self.trigrams[i]);
            }

            // Skipgrams
            // XXX: Half-assed, assumes all corpus chars were valid.
            let sg = &[tg[0], tg[2]];
            let idx = self.corpus_bigram(sg);
            self.skipgrams[idx] -= sum;
        }
    }

    #[instrument(level = "debug", skip(self))]
    fn adapt_interior_ngrams(&mut self, old: [char; 2], new: [char; 2]) {
        let num_trigrams = self.get_trigrams().len();
        let mut acc = vec![0; num_trigrams];

        for i in 0..num_trigrams {
            if self.trigrams[i] == 0 {
                continue
            }
            let tg = self.uncorpus_trigram(i);
            if tg[0] == old[0] && tg[1] == old[1] {
                // he*
                // self.adapt_interior_ngram(i, &tg[..], &[new[0], new[1], tg[2]]);
                #[rustfmt::skip]
                <Corpus as AdaptiveCorpus<[char; 3]>>::adapt_interior_ngram(self, i, &tg[..], &[new[0], new[1], tg[2]], &mut acc);
            }
            if tg[1] == old[0] && tg[2] == old[1] {
                // *he
                // self.adapt_interior_ngram(i, &tg[..], &[tg[0], new[0], new[1]]);
                #[rustfmt::skip]
                <Corpus as AdaptiveCorpus<[char; 3]>>::adapt_interior_ngram(self, i, &tg[..], &[tg[0], new[0], new[1]], &mut acc);
            }
        }

        for (a, b) in self.get_trigrams().iter_mut().zip(&acc) {
            *a = a.checked_add_signed(*b).expect("Overflow!");
        }
    }

    fn adapt_interior_ngram(&mut self, old_idx: usize, old_ng: &[char], new_ng: &[char], acc: &mut Vec<i32>) {
        let freq = self.get_trigrams()[old_idx];

        if DEBUG_TRIGRAMS.contains(&[old_ng[0], old_ng[1], old_ng[2]])
            || DEBUG_TRIGRAMS.contains(&[new_ng[0], new_ng[1], new_ng[2]])
        {
            debug!(?old_ng, freq_pre = freq);
        }
        acc[old_idx] = acc[old_idx].checked_sub_unsigned(freq).expect("Overflow!");

        let new_idx = self.corpus_trigram(&[new_ng[0], new_ng[1], new_ng[2]]);
        if DEBUG_TRIGRAMS.contains(&[old_ng[0], old_ng[1], old_ng[2]])
            || DEBUG_TRIGRAMS.contains(&[new_ng[0], new_ng[1], new_ng[2]])
        {
            debug!(?new_ng, freq_pre = self.get_trigrams()[new_idx]);
        }
        acc[new_idx] = acc[new_idx].checked_add_unsigned(freq).expect("Overflow!");
        if DEBUG_TRIGRAMS.contains(&[old_ng[0], old_ng[1], old_ng[2]])
            || DEBUG_TRIGRAMS.contains(&[new_ng[0], new_ng[1], new_ng[2]])
        {
            debug!(?new_ng, freq_post = self.get_trigrams()[new_idx]);
        }

        // Skipgrams
        // XXX: Half-assed skipgrams, assumes all corpus chars were valid.
        let old_idx = self.corpus_bigram(&[old_ng[0], old_ng[2]]);
        self.get_skipgrams()[old_idx] -= freq;
        let new_sg = &[new_ng[0], new_ng[2]];
        let new_idx = self.corpus_bigram(new_sg);
        self.get_skipgrams()[new_idx] += freq;
    }
}
