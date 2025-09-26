// Copyright 2025 antlers <antlers@illucid.net>
//
// SPDX-License-Identifier: GPL-3.0-only

use crate::CorpusExt;
use crate::adaptive_corpus::*;
use kc::Corpus;

use tracing::instrument;

impl GetCount<[char; 6], [char; 5]> for ExpansionStruct<[char; 6], [char; 5]> {
    /// "Count" hexagrams.
    fn get_count<U: CorpusExt>(&self, corpus: &mut U) -> u32 {
        #[cfg(feature = "synth-large-ngrams")]
        {
            let prefix = &[self.old[0], self.old[1], self.old[2], self.old[3], self.old[4]];
            let suffix = &[self.old[1], self.old[2], self.old[3], self.old[4], self.old[5]];
            let prefix_idx = corpus.corpus_pentagram(prefix);
            let suffix_idx = corpus.corpus_pentagram(suffix);
            let pgs = corpus.get_pentagrams();

            std::cmp::min(pgs[prefix_idx], pgs[suffix_idx])
        }

        #[cfg(not(feature = "synth-large-ngrams"))]
        Default::default()
    }
}

impl GetCount<[char; 7], [char; 5]> for ExpansionStruct<[char; 7], [char; 5]> {
    /// "Count" septegrams.
    fn get_count<U: CorpusExt>(&self, corpus: &mut U) -> u32 {
        #[cfg(feature = "synth-large-ngrams")]
        {
            let prefix = &[self.old[0], self.old[1], self.old[2], self.old[3], self.old[4]];
            let suffix = &[self.old[2], self.old[3], self.old[4], self.old[5], self.old[6]];
            let prefix_idx = corpus.corpus_pentagram(prefix);
            let suffix_idx = corpus.corpus_pentagram(suffix);
            let sgs = corpus.get_pentagrams();

            std::cmp::min(sgs[prefix_idx], sgs[suffix_idx])
        }

        #[cfg(not(feature = "synth-large-ngrams"))]
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

        let mut pg = self.clone();
        if pg[0] == old[0] && pg[1] == old[1] && pg[2] == old[0] && pg[3] == old[1] {
            // hehe*
            pg = [new[0], new[1], new[0], new[1], pg[4]];
        } else if pg[1] == old[0] && pg[2] == old[1] && pg[3] == old[0] && pg[4] == old[1] {
            // *hehe
            pg = [pg[0], new[0], new[1], new[0], new[1]];
        } else if pg[0] == old[0] && pg[1] == old[1] && pg[3] == old[0] && pg[4] == old[1] {
            // he*he
            pg = [new[0], new[1], pg[2], new[0], new[1]];
        } else if pg[0] == old[0] && pg[1] == old[1] {
            // he***
            pg = [new[0], new[1], pg[2], pg[3], pg[4]];
        } else if pg[1] == old[0] && pg[2] == old[1] {
            // *he**
            pg = [pg[0], new[0], new[1], pg[3], pg[4]];
        } else if pg[2] == old[0] && pg[3] == old[1] {
            // **he*
            pg = [pg[0], pg[1], new[0], new[1], pg[4]];
        } else if pg[3] == old[0] && pg[4] == old[1] {
            // ***he
            pg = [pg[0], pg[1], pg[2], new[0], new[1]];
        }

        // If the pentagram starts with the old bigram suffix, left
        if pg[0] == old[1] {
            left = Some(ExpansionStruct::new(
                [old[0], self[0], self[1], self[2], self[3], self[4]],
                [new[1], pg[1], pg[2], pg[3], pg[4]],
            ));
        }

        // If the pentagram ends with the old bigram prefix, right
        if pg[4] == old[0] {
            right = Some(ExpansionStruct::new(
                [self[0], self[1], self[2], self[3], self[4], old[1]],
                [pg[0], pg[1], pg[2], pg[3], new[0]],
            ));

            // If both, both
            if let Some(ref left) = left {
                both = Some(ExpansionStruct::new(
                    [left.old[0], left.old[1], left.old[2], left.old[3], left.old[4], left.old[5], old[1]],
                    [left.new[0], left.new[1], left.new[2], left.new[3], new[0]],
                ));
            }
        }

        Expansions { left, right, both }
    }
}

/// Methods for adapting pentagram frequencies to reflect bigram substitutions.
impl AdaptiveCorpus<[char; 5]> for Corpus {
    #[instrument(level = "debug", skip(self))]
    fn adapt_ngrams(&mut self, old: [char; 2], new: [char; 2]) {
        // self.adapt_interior_ngrams(old, new);
        <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_interior_ngrams(self, old, new);
    }

    fn adapt_boundary_ngrams(&mut self, old: [char; 2], new: [char; 2]) {
        let num_pentagrams = self.get_pentagrams().len();

        for mut i in 0..num_pentagrams {
            let mut pg = self.uncorpus_pentagram(i);
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

            if pg[0] == old[0] && pg[1] == old[1] && pg[2] == old[0] && pg[3] == old[1] {
                // hehe*
                pg = [new[0], new[1], new[0], new[1], pg[4]].to_vec();
                i = self.corpus_pentagram(&[pg[0], pg[1], pg[2], pg[3], pg[4]]);
            } else if pg[1] == old[0] && pg[2] == old[1] && pg[3] == old[0] && pg[4] == old[1] {
                // *hehe
                i = self.corpus_pentagram(&[pg[0], pg[1], pg[2], pg[3], pg[4]]);
            } else if pg[0] == old[0] && pg[1] == old[1] && pg[3] == old[0] && pg[4] == old[1] {
                // he*he
                pg = [new[0], new[1], pg[2], new[0], new[1]].to_vec();
                i = self.corpus_pentagram(&[pg[0], pg[1], pg[2], pg[3], pg[4]]);
            } else if pg[0] == old[0] && pg[1] == old[1] {
                // he***
                pg = [new[0], new[1], pg[2], pg[3], pg[4]].to_vec();
                i = self.corpus_pentagram(&[pg[0], pg[1], pg[2], pg[3], pg[4]]);
            } else if pg[1] == old[0] && pg[2] == old[1] {
                // *he**
                pg = [pg[0], new[0], new[1], pg[3], pg[4]].to_vec();
                i = self.corpus_pentagram(&[pg[0], pg[1], pg[2], pg[3], pg[4]]);
            } else if pg[2] == old[0] && pg[3] == old[1] {
                // **he*
                pg = [pg[0], pg[1], new[0], new[1], pg[4]].to_vec();
                i = self.corpus_pentagram(&[pg[0], pg[1], pg[2], pg[3], pg[4]]);
            } else if pg[3] == old[0] && pg[4] == old[1] {
                // ***he
                pg = [pg[0], pg[1], pg[2], new[0], new[1]].to_vec();
                i = self.corpus_pentagram(&[pg[0], pg[1], pg[2], pg[3], pg[4]]);
            }

            self.get_pentagrams()[i] -= sum;
        }
    }

    #[instrument(level = "debug", skip(self))]
    fn adapt_interior_ngrams(&mut self, old: [char; 2], new: [char; 2]) {
        let num_pentagrams = self.get_pentagrams().len();
        let mut acc = vec![0; num_pentagrams];

        for i in 0..num_pentagrams {
            if self.pentagrams[i] == 0 {
                continue
            }
            let pg = self.uncorpus_pentagram(i);

            // XXX: Probably not correct for replacing repeats
            if pg[0] == old[0] && pg[1] == old[1] && pg[2] == old[0] && pg[3] == old[1] {
                // hehe*
                #[rustfmt::skip]
                <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_interior_ngram(self, i, &pg[..], &[new[0], new[1], new[0], new[1], pg[4]], &mut acc);
            } else if pg[1] == old[0] && pg[2] == old[1] && pg[3] == old[0] && pg[4] == old[1] {
                // *hehe
                #[rustfmt::skip]
                <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_interior_ngram(self, i, &pg[..], &[pg[0], new[0], new[1], new[0], new[1]], &mut acc);
            } else if pg[0] == old[0] && pg[1] == old[1] && pg[3] == old[0] && pg[4] == old[1] {
                // he*he
                #[rustfmt::skip]
                <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_interior_ngram(self, i, &pg[..], &[new[0], new[1], pg[2], new[0], new[1]], &mut acc);
            } else if pg[0] == old[0] && pg[1] == old[1] {
                // he***
                #[rustfmt::skip]
                <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_interior_ngram(self, i, &pg[..], &[new[0], new[1], pg[2], pg[3], pg[4]], &mut acc);
            } else if pg[1] == old[0] && pg[2] == old[1] {
                // *he**
                #[rustfmt::skip]
                <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_interior_ngram(self, i, &pg[..], &[pg[0], new[0], new[1], pg[3], pg[4]], &mut acc);
            } else if pg[2] == old[0] && pg[3] == old[1] {
                // **he*
                #[rustfmt::skip]
                <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_interior_ngram(self, i, &pg[..], &[pg[0], pg[1], new[0], new[1], pg[4]], &mut acc);
            } else if pg[3] == old[0] && pg[4] == old[1] {
                // ***he
                #[rustfmt::skip]
                <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_interior_ngram(self, i, &pg[..], &[pg[0], pg[1], pg[2], new[0], new[1]], &mut acc);
            }
        }

        for (a, b) in self.get_pentagrams().iter_mut().zip(&acc) {
            *a = a.checked_add_signed(*b).expect("Overflow!");
        }
    }

    fn adapt_interior_ngram(&mut self, old_idx: usize, old_ng: &[char], new_ng: &[char], acc: &mut Vec<i32>) {
        let freq = self.get_pentagrams()[old_idx];
        acc[old_idx] = acc[old_idx].checked_sub_unsigned(freq).unwrap();

        let new_idx =
            self.corpus_pentagram(&[new_ng[0], new_ng[1], new_ng[2], new_ng[3], new_ng[4]]);
        acc[new_idx] = acc[new_idx].checked_add_unsigned(freq).unwrap();
    }
}
