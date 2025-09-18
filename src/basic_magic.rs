//! Dynamic trigram adjustments for simple magic rules.
//!
//! Right now, "simple" means bigram rules like `h* -> he`.
//!
//! # Examples
//!
//! ```
//! use km_basic_magic::apply;
//!
//! let b = fs::read("./corpora/shai-iweb.corpus").unwrap();
//! let mut corpus: Corpus = rmp_serde::from_slice(&b).unwrap();
//! corpus = apply(corpus, ['h', 'e'], ['h', '†']);
//! ```

use kc::Corpus;

/// Specialized methods implemented per expansion-length to use generic [Expansion] methods.
trait ExpansionBase<N> {
    fn get_count(&self, corpus: &Corpus) -> u32;
}

/// Generic methods implemented per ngram-length but abstracted over expansion-length via
/// [ExpansionBase].
trait Expansion<N>: ExpansionBase<N> {
    fn new(old: N, new: [char; 3]) -> Self;
    fn add_count(&mut self, corpus: &mut Corpus);
    fn read_count(&self) -> u32;
    fn set_count(&mut self, count: u32);
    fn add_boundary_ngrams(&self, corpus: &mut Corpus, idx: u32, new: [char; 2], old: [char; 2]);
}

/// Expansions derived from trigrams.
#[derive(Debug, Clone)]
struct TrigramExpansion<N> {
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

    /// `TODO`
    fn add_boundary_ngrams(&self, corpus: &mut Corpus, idx: u32, new: [char; 2], old: [char; 2]) {
        todo!();
    }
}

/// Apply simple (bigram) transformation `old -> new` to `corpus`.
#[must_use]
pub fn apply(mut corpus: Corpus, old: [char; 2], new: [char; 2]) -> Corpus {
    corpus = apply_tg(corpus, old, new);
    corpus
}

/// # Debugging
/// - See commented code in test::si_compare_all_trigrams
/// ```
/// if tg == &['†', 'a', 'h'] { ... }
/// ```
fn apply_tg(mut corpus: Corpus, old: [char; 2], new: [char; 2]) -> Corpus {
    let num_trigrams = corpus.trigrams.len();

    // Non-boundary ngrams
    for i in 0..num_trigrams {
        if true {
            let tg = corpus.uncorpus_trigram(i);
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

            macro_rules! sum {
                ($($tg:ident),*) => {
                    [$($tg.as_ref().and_then(|x| Some(x.read_count())).unwrap_or(0)),*].into_iter().sum()
                }
            }

            fn corpus_reflect_expansion<T, U>(exp: &mut Option<T>, corpus: &mut Corpus, offset: u32)
            where
                T: Expansion<U>,
            {
                if let Some(exp) = exp {
                    exp.set_count(exp.get_count(&corpus) - offset);
                    exp.add_count(corpus)
                }
            }

            corpus_reflect_expansion(&mut both, &mut corpus, 0);
            let bcount = sum!(both);
            corpus_reflect_expansion(&mut left, &mut corpus, bcount);
            corpus_reflect_expansion(&mut right, &mut corpus, bcount);

            let sum: u32 = sum!(left, right, both);

            corpus.trigrams[i] -= sum;

            // Skipgrams
            // XXX: Half-assed, assumes all corpus chars were valid.
            let sg = &[tg[0], tg[2]];
            let idx = corpus.corpus_bigram(sg);
            corpus.skipgrams[idx] -= sum;
        }
    }

    // Boundary ngrams
    for i in 0..num_trigrams {
        let tg = corpus.uncorpus_trigram(i);

        // TODO: Would probably like to fold these into the traits?
        // `TrigramExpansion<_>::add_boundary_ngrams(&mut corpus, idx, new, old)`?
        // Uses lexical variables `i`, `tg`, and `corpus`.
        // Call-site uses top-level bigrams `new` and `old`.
        // Will be easier to make this call after trying bigrams.
        macro_rules! add_boundary_tg {
            ($tg:expr) => {
                let freq = corpus.trigrams[i];
                corpus.trigrams[i] -= freq;

                let new_tg = $tg;
                let idx = corpus.corpus_trigram(new_tg);
                corpus.trigrams[idx] += freq;

                // Skipgrams
                // XXX: Half-assed skipgrams, assumes all corpus chars were valid.
                let idx = corpus.corpus_bigram(&[tg[0], tg[2]]);
                corpus.skipgrams[idx] -= freq;
                let sg = &[new_tg[0], new_tg[2]];
                let idx = corpus.corpus_bigram(sg);
                corpus.skipgrams[idx] += freq;
            };
        }

        if tg[0] == old[0] && tg[1] == old[1] {
            add_boundary_tg!(&[new[0], new[1], tg[2]]);
        }
        if tg[1] == old[0] && tg[2] == old[1] {
            add_boundary_tg!(&[tg[0], new[0], new[1]]);
        }
    }

    corpus
}

#[cfg(test)]
fn count_char(corpus: &Corpus, c: char) -> u32 {
    corpus.chars[corpus.corpus_char(c)]
}

#[cfg(test)]
fn count_bigram(corpus: &Corpus, bg: [char; 2]) -> u32 {
    corpus.bigrams[corpus.corpus_bigram(&bg)]
}

#[cfg(test)]
fn count_trigram(corpus: &Corpus, tg: [char; 3]) -> u32 {
    corpus.trigrams[corpus.corpus_trigram(&tg)]
}

#[cfg(test)]
fn count_skipgram(corpus: &Corpus, sg: [char; 2]) -> u32 {
    corpus.skipgrams[corpus.corpus_bigram(&sg)]
}

#[cfg(test)]
fn verify_corpus_si_pre(corpus: Corpus) {
    // Monograms
    assert_eq!(count_char(&corpus, 'e'), 50497522);
    assert_eq!(count_char(&corpus, '†'), 0);

    // // Bigrams
    assert_eq!(count_bigram(&corpus, ['h', 'e']), 8729312);
    assert_eq!(count_bigram(&corpus, ['h', '†']), 0);

    // Trigrams
    assert_eq!(count_trigram(&corpus, ['t', 'h', 'e']), 6802477);
    assert_eq!(count_trigram(&corpus, ['t', 'h', '†']), 0);
    assert_eq!(count_trigram(&corpus, ['h', 'e', ' ']), 5421447);
    assert_eq!(count_trigram(&corpus, ['h', '†', ' ']), 0);
    assert_eq!(count_trigram(&corpus, ['e', ' ', 'q']), 40503);
    assert_eq!(count_trigram(&corpus, ['†', ' ', 'q']), 0);
    assert_eq!(count_trigram(&corpus, ['e', ' ', 'l']), 438196);
    assert_eq!(count_trigram(&corpus, ['†', ' ', 'l']), 0);

    // Skipgrams
    assert_eq!(count_skipgram(&corpus, ['t', 'e']), 7955798);
    assert_eq!(count_skipgram(&corpus, ['t', '†']), 0);
    assert_eq!(count_skipgram(&corpus, ['e', 'q']), 45074);
    assert_eq!(count_skipgram(&corpus, ['†', 'q']), 0);
    assert_eq!(count_skipgram(&corpus, ['e', 'l']), 1300716);
    assert_eq!(count_skipgram(&corpus, ['†', 'l']), 0);

    // Everything else the same
    assert_eq!(count_bigram(&corpus, ['v', 'e']), 2978051);
    assert_eq!(count_bigram(&corpus, ['e', 'r']), 6377359);
    assert_eq!(count_trigram(&corpus, ['o', 'v', 'e']), 496824);
    assert_eq!(count_trigram(&corpus, ['v', 'e', 'r']), 934355);
    assert_eq!(count_trigram(&corpus, ['e', 'r', ' ']), 2089364);
    assert_eq!(count_skipgram(&corpus, ['o', 'e']), 3663662);
    assert_eq!(count_skipgram(&corpus, ['e', ' ']), 10441564);
}

#[cfg(test)]
fn verify_corpus_si_post(corpus: Corpus) {
    // // Monograms
    // assert_eq!(count_char(&corpus, 'e'), 41768210);
    // assert_eq!(count_char(&corpus, '†'), 8729312);

    // // // Bigrams
    // assert_eq!(count_bigram(&corpus, ['h', 'e']), 0);
    // assert_eq!(count_bigram(&corpus, ['h', '†']), 8729312);
    // assert_eq!(count_bigram(&corpus, ['†', 'e']), 40073);
    // assert_eq!(count_bigram(&corpus, ['†', 'a']), 248245);
    // assert_eq!(count_bigram(&corpus, ['†', 'i']), 253922);
    // assert_eq!(count_bigram(&corpus, ['†', 'o']), 12705);
    // assert_eq!(count_bigram(&corpus, ['†', ' ']), 5421447);

    // Trigrams
    assert_eq!(count_trigram(&corpus, ['t', 'h', 'e']), 0);
    assert_eq!(count_trigram(&corpus, ['t', 'h', '†']), 6802477);
    assert_eq!(count_trigram(&corpus, ['h', 'e', ' ']), 0);
    assert_eq!(count_trigram(&corpus, ['h', '†', ' ']), 5421447);
    assert_eq!(count_trigram(&corpus, ['e', ' ', 'q']), 21049);
    assert_eq!(count_trigram(&corpus, ['†', ' ', 'q']), 19454);
    assert_eq!(count_trigram(&corpus, ['e', ' ', 'l']), 210202);
    assert_eq!(count_trigram(&corpus, ['†', ' ', 'l']), 227994);
    assert_eq!(count_trigram(&corpus, ['e', 'a', 'h']), 6357);
    assert_eq!(count_trigram(&corpus, ['†', 'a', 'h']), 24);

    // Skipgrams
    assert_eq!(count_skipgram(&corpus, ['t', 'e']), 1153321);
    assert_eq!(count_skipgram(&corpus, ['t', '†']), 6802477);
    assert_eq!(count_skipgram(&corpus, ['e', 'q']), 25582);
    assert_eq!(count_skipgram(&corpus, ['†', 'q']), 19492);
    assert_eq!(count_skipgram(&corpus, ['e', 'l']), 977172);
    assert_eq!(count_skipgram(&corpus, ['†', 'l']), 323544);

    // Everything else the same
    // assert_eq!(count_bigram(&corpus, ['v', 'e']), 2978051);
    // assert_eq!(count_bigram(&corpus, ['e', 'r']), 5219156);
    // assert_eq!(count_bigram(&corpus, ['e', 'o']), 252440);
    // assert_eq!(count_bigram(&corpus, ['e', 'i']), 291015);
    // assert_eq!(count_bigram(&corpus, ['e', 'h']), 85972);
    assert_eq!(count_trigram(&corpus, ['o', 'v', 'e']), 496824);
    assert_eq!(count_trigram(&corpus, ['v', 'e', 'r']), 934355);
    assert_eq!(count_trigram(&corpus, ['e', 'r', ' ']), 1580390);
    assert_eq!(count_skipgram(&corpus, ['o', 'e']), 3660890);

    // Requires adjustments based on pentagrams to handle skips over invalid corpus chars
    // assert_eq!(count_skipgram(&corpus, ['e', ' ']), 9021099);
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
        corpus = apply(corpus, ['h', 'e'], ['h', '†']);
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
        corpus = apply(corpus, ['h', 'e'], ['h', '†']);

        let b = fs::read("./corpora/shai-iweb-he.corpus").expect("couldn't read corpus file");
        let ref_corpus: Corpus = rmp_serde::from_slice(&b).expect("couldn't deserialize corpus");

        // assert_eq!(ref_corpus.trigrams, corpus.trigrams);

        let num_trigrams = corpus.trigrams.len();
        assert_eq!(ref_corpus.trigrams.len(), num_trigrams);
        for i in 0..num_trigrams {
            let tg = corpus.uncorpus_trigram(i);
            let ref_tg_idx = ref_corpus.corpus_trigram(&[tg[0], tg[1], tg[2]]);
            println!("{:?}", tg);
            assert_eq!(corpus.trigrams[i], ref_corpus.trigrams[ref_tg_idx]);
        }
    }
}
