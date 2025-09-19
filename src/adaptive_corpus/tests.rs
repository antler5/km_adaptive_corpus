// Copyright 2025 antlers <antlers@illucid.net>
//
// SPDX-License-Identifier: GPL-3.0-only

use super::*;
use std::fs;

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
