//! Dynamic trigram adjustments for simple magic rules.
//!
//! Right now, "simple" means adaptive-key style bigram rules like `h* -> he`.
//!
//! # Examples
//!
//! ```
//! use kc:Corpus;
//! use km_basic_magic::AdaptiveCorpus;
//!
//! let b = fs::read("./corpora/shai-iweb.corpus").unwrap();
//! let mut corpus: Corpus = rmp_serde::from_slice(&b).unwrap();
//! corpus.adapt_trigrams(['h', 'e'], ['h', '†']);
//! ```

use crate::CorpusExt;
use kc::Corpus;

/// Interface for adapting ngram frequencies to reflect bigram substitutions.
pub trait AdaptiveCorpus {
    fn adapt_trigrams(&mut self, old: [char; 2], new: [char; 2]);
    fn expand_trigram(
        tg: &[char],
        old: [char; 2],
        new: [char; 2],
    ) -> (
        Option<TrigramExpansion<[char; 4]>>,
        Option<TrigramExpansion<[char; 4]>>,
        Option<TrigramExpansion<[char; 5]>>,
    );
    fn adapt_interior_trigram<T, U>(&mut self, exp: &mut Option<T>, bcount: u32)
    where
        T: Expansion<U>;
    fn adapt_interior_trigrams(&mut self, old: [char; 2], new: [char; 2]);
    fn adapt_boundary_trigram(&mut self, old_idx: usize, old_ng: &[char], new_ng: &[char; 3]);
    fn adapt_boundary_trigrams(&mut self, old: [char; 2], new: [char; 2]);
}

/// Methods for adapting ngram frequencies to reflect bigram substitutions.
///
/// # Debugging
/// - See commented code in module `test::si_compare_all_trigrams`
/// ```
/// if tg == &['†', 'a', 'h'] { ... }
/// ```
impl AdaptiveCorpus for Corpus {
    fn adapt_trigrams(&mut self, old: [char; 2], new: [char; 2]) {
        self.adapt_interior_trigrams(old, new);
        self.adapt_boundary_trigrams(old, new);
    }

    fn expand_trigram(
        tg: &[char],
        old: [char; 2],
        new: [char; 2],
    ) -> (
        Option<TrigramExpansion<[char; 4]>>,
        Option<TrigramExpansion<[char; 4]>>,
        Option<TrigramExpansion<[char; 5]>>,
    ) {
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

        (left, right, both)
    }

    fn adapt_interior_trigram<T, U>(&mut self, exp: &mut Option<T>, bcount: u32)
    where
        T: Expansion<U>,
    {
        if let Some(exp) = exp {
            exp.set_count(exp.get_count(&self) - bcount);
            exp.add_count(self)
        }
    }

    fn adapt_interior_trigrams(&mut self, old: [char; 2], new: [char; 2]) {
        let num_trigrams = self.get_trigrams().len();
        for i in 0..num_trigrams {
            let tg = self.uncorpus_trigram(i);

            let (mut left, mut right, mut both) = Corpus::expand_trigram(&tg[..], old, new);

            macro_rules! sum {
                ($($tg:ident),*) => {
                    [$($tg.as_ref()
                          .and_then(|x| Some(x.read_count()))
                          .unwrap_or(0)
                     ),*
                    ].into_iter().sum()
                }
            }

            self.adapt_interior_trigram(&mut both, 0);
            let bcount = sum!(both);
            self.adapt_interior_trigram(&mut left, bcount);
            self.adapt_interior_trigram(&mut right, bcount);

            let sum: u32 = sum!(left, right, both);

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

/// Specialized methods implemented per expansion-length to use generic [Expansion] methods.
pub trait ExpansionBase<N> {
    fn get_count(&self, corpus: &Corpus) -> u32;
}

/// Generic methods implemented per ngram-length but abstracted over expansion-length via
/// [ExpansionBase].
pub trait Expansion<N>: ExpansionBase<N> {
    fn new(old: N, new: [char; 3]) -> Self;
    fn add_count(&mut self, corpus: &mut Corpus);
    fn read_count(&self) -> u32;
    fn set_count(&mut self, count: u32);
}

/// Expansions derived from trigrams.
#[derive(Debug, Clone)]
pub struct TrigramExpansion<N> {
    /// Four to five character expansion derived from a modified trigram.
    old: N,
    /// New trigram post-transformation.
    new: [char; 3],
    /// Frequency of `old` in `corpus`.
    count: Option<u32>,
}

impl ExpansionBase<[char; 4]> for TrigramExpansion<[char; 4]> {
    /// Count quadgrams.
    fn get_count(&self, corpus: &Corpus) -> u32 {
        corpus.quadgrams[corpus.corpus_quadgram(&self.old)]
    }
}

impl ExpansionBase<[char; 5]> for TrigramExpansion<[char; 5]> {
    /// Count pentagrams.
    fn get_count(&self, corpus: &Corpus) -> u32 {
        corpus.pentagrams[corpus.corpus_pentagram(&self.old)]
    }
}

/// Generic methods for trigram-expansions abstracted over [ExpansionBase].
impl<N> Expansion<N> for TrigramExpansion<N>
where
    TrigramExpansion<N>: ExpansionBase<N>,
{
    fn new(old: N, new: [char; 3]) -> Self {
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

#[cfg(test)]
fn verify_corpus_si_pre(corpus: Corpus) {
    // Monograms
    assert_eq!(corpus.count_char('e'), 50497522);
    assert_eq!(corpus.count_char('†'), 0);

    // // Bigrams
    assert_eq!(corpus.count_bigram(['h', 'e']), 8729312);
    assert_eq!(corpus.count_bigram(['h', '†']), 0);

    // Trigrams
    assert_eq!(corpus.count_trigram(['t', 'h', 'e']), 6802477);
    assert_eq!(corpus.count_trigram(['t', 'h', '†']), 0);
    assert_eq!(corpus.count_trigram(['h', 'e', ' ']), 5421447);
    assert_eq!(corpus.count_trigram(['h', '†', ' ']), 0);
    assert_eq!(corpus.count_trigram(['e', ' ', 'q']), 40503);
    assert_eq!(corpus.count_trigram(['†', ' ', 'q']), 0);
    assert_eq!(corpus.count_trigram(['e', ' ', 'l']), 438196);
    assert_eq!(corpus.count_trigram(['†', ' ', 'l']), 0);

    // Skipgrams
    assert_eq!(corpus.count_skipgram(['t', 'e']), 7955798);
    assert_eq!(corpus.count_skipgram(['t', '†']), 0);
    assert_eq!(corpus.count_skipgram(['e', 'q']), 45074);
    assert_eq!(corpus.count_skipgram(['†', 'q']), 0);
    assert_eq!(corpus.count_skipgram(['e', 'l']), 1300716);
    assert_eq!(corpus.count_skipgram(['†', 'l']), 0);

    // Everything else the same
    assert_eq!(corpus.count_bigram(['v', 'e']), 2978051);
    assert_eq!(corpus.count_bigram(['e', 'r']), 6377359);
    assert_eq!(corpus.count_trigram(['o', 'v', 'e']), 496824);
    assert_eq!(corpus.count_trigram(['v', 'e', 'r']), 934355);
    assert_eq!(corpus.count_trigram(['e', 'r', ' ']), 2089364);
    assert_eq!(corpus.count_skipgram(['o', 'e']), 3663662);
    assert_eq!(corpus.count_skipgram(['e', ' ']), 10441564);
}

#[cfg(test)]
fn verify_corpus_si_post(corpus: Corpus) {
    // // Monograms
    // assert_eq!(corpus.count_char('e'), 41768210);
    // assert_eq!(corpus.count_char('†'), 8729312);

    // // // Bigrams
    // assert_eq!(corpus.count_bigram(['h', 'e']), 0);
    // assert_eq!(corpus.count_bigram(['h', '†']), 8729312);
    // assert_eq!(corpus.count_bigram(['†', 'e']), 40073);
    // assert_eq!(corpus.count_bigram(['†', 'a']), 248245);
    // assert_eq!(corpus.count_bigram(['†', 'i']), 253922);
    // assert_eq!(corpus.count_bigram(['†', 'o']), 12705);
    // assert_eq!(corpus.count_bigram(['†', ' ']), 5421447);

    // Trigrams
    assert_eq!(corpus.count_trigram(['t', 'h', 'e']), 0);
    assert_eq!(corpus.count_trigram(['t', 'h', '†']), 6802477);
    assert_eq!(corpus.count_trigram(['h', 'e', ' ']), 0);
    assert_eq!(corpus.count_trigram(['h', '†', ' ']), 5421447);
    assert_eq!(corpus.count_trigram(['e', ' ', 'q']), 21049);
    assert_eq!(corpus.count_trigram(['†', ' ', 'q']), 19454);
    assert_eq!(corpus.count_trigram(['e', ' ', 'l']), 210202);
    assert_eq!(corpus.count_trigram(['†', ' ', 'l']), 227994);
    assert_eq!(corpus.count_trigram(['e', 'a', 'h']), 6357);
    assert_eq!(corpus.count_trigram(['†', 'a', 'h']), 24);

    // Skipgrams
    assert_eq!(corpus.count_skipgram(['t', 'e']), 1153321);
    assert_eq!(corpus.count_skipgram(['t', '†']), 6802477);
    assert_eq!(corpus.count_skipgram(['e', 'q']), 25582);
    assert_eq!(corpus.count_skipgram(['†', 'q']), 19492);
    assert_eq!(corpus.count_skipgram(['e', 'l']), 977172);
    assert_eq!(corpus.count_skipgram(['†', 'l']), 323544);

    // Everything else the same
    // assert_eq!(corpus.count_bigram(['v', 'e']), 2978051);
    // assert_eq!(corpus.count_bigram(['e', 'r']), 5219156);
    // assert_eq!(corpus.count_bigram(['e', 'o']), 252440);
    // assert_eq!(corpus.count_bigram(['e', 'i']), 291015);
    // assert_eq!(corpus.count_bigram(['e', 'h']), 85972);
    assert_eq!(corpus.count_trigram(['o', 'v', 'e']), 496824);
    assert_eq!(corpus.count_trigram(['v', 'e', 'r']), 934355);
    assert_eq!(corpus.count_trigram(['e', 'r', ' ']), 1580390);
    assert_eq!(corpus.count_skipgram(['o', 'e']), 3660890);

    // Requires adjustments based on pentagrams to handle skips over invalid corpus chars
    // assert_eq!(corpus.count_skipgram(['e', ' ']), 9021099);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn si_pre() {
        let b = fs::read("./corpora/shai-iweb.corpus").expect("couldn't read corpus file");
        let corpus: Corpus = rmp_serde::from_slice(&b).expect("couldn't deserialize corpus");
        verify_corpus_si_pre(corpus);
    }

    #[test]
    fn si_post() {
        let b = fs::read("./corpora/shai-iweb.corpus").expect("couldn't read corpus file");
        let mut corpus: Corpus = rmp_serde::from_slice(&b).expect("couldn't deserialize corpus");
        corpus.adapt_trigrams(['h', 'e'], ['h', '†']);
        verify_corpus_si_post(corpus);
    }

    #[test]
    fn si_ref() {
        let b = fs::read("./corpora/shai-iweb-he.corpus").expect("couldn't read corpus file");
        let corpus: Corpus = rmp_serde::from_slice(&b).expect("couldn't deserialize corpus");
        verify_corpus_si_post(corpus);
    }

    /// XXX: Can OOM in release-mode.
    #[ignore]
    #[test]
    fn si_compare_all_trigrams() {
        let b = fs::read("./corpora/shai-iweb.corpus").expect("couldn't read corpus file");
        let mut corpus: Corpus = rmp_serde::from_slice(&b).expect("couldn't deserialize corpus");
        corpus.adapt_trigrams(['h', 'e'], ['h', '†']);

        let b = fs::read("./corpora/shai-iweb-he.corpus").expect("couldn't read corpus file");
        let ref_corpus: Corpus = rmp_serde::from_slice(&b).expect("couldn't deserialize corpus");

        assert_eq!(ref_corpus.trigrams, corpus.trigrams);

        // let num_trigrams = corpus.trigrams.len();
        // assert_eq!(ref_corpus.trigrams.len(), num_trigrams);
        // for i in 0..num_trigrams {
        //     let tg = corpus.uncorpus_trigram(i);
        //     let ref_tg_idx = ref_corpus.corpus_trigram(&[tg[0], tg[1], tg[2]]);
        //     // println!("{:?}", tg);
        //     assert_eq!(corpus.trigrams[i], ref_corpus.trigrams[ref_tg_idx]);
        // }
    }
}
