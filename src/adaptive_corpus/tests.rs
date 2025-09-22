// Copyright 2025 antlers <antlers@illucid.net>
//
// SPDX-License-Identifier: GPL-3.0-only

use super::*;
use std::fs;

use kc::Corpus;

fn verify_corpus_si_pre(corpus: Corpus) {
    // Monograms
    assert_eq!(corpus.count_char('e'), 50497522);
    assert_eq!(corpus.count_char('†'), 0);

    // Bigrams
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
fn verify_corpus_si_he(corpus: Corpus) {
    // Monograms
    assert_eq!(corpus.count_char('e'), 41768210);
    assert_eq!(corpus.count_char('†'), 8729312);

    // Bigrams
    assert_eq!(corpus.count_bigram(['h', 'e']), 0);
    assert_eq!(corpus.count_bigram(['h', '†']), 8729312);
    assert_eq!(corpus.count_bigram(['†', 'e']), 40073);
    assert_eq!(corpus.count_bigram(['†', 'a']), 248245);
    assert_eq!(corpus.count_bigram(['†', 'i']), 253922);
    assert_eq!(corpus.count_bigram(['†', 'o']), 12705);
    assert_eq!(corpus.count_bigram(['†', ' ']), 5421447);

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
    assert_eq!(corpus.count_trigram(['o', 'v', 'e']), 496824);
    assert_eq!(corpus.count_trigram(['v', 'e', 'r']), 934355);
    assert_eq!(corpus.count_trigram(['e', 'r', ' ']), 1580390);
    assert_eq!(corpus.count_skipgram(['o', 'e']), 3660890);

    // Requires adjustments based on pentagrams to handle skips over invalid corpus chars
    // assert_eq!(corpus.count_skipgram(['e', ' ']), 9021099);
}

#[cfg(test)]
fn verify_corpus_si_he_er(corpus: Corpus) {
    // he -> h†
    // er -> r† (whoops)

    // assert_eq!(corpus.count_trigram(['v', 'e', 'r']), 283); // Getting: 0 (seems right?)

    // Monograms
    assert_eq!(corpus.count_char('e'), 36549054);
    assert_eq!(corpus.count_char('†'), 13948468);

    // Bigrams
    assert_eq!(corpus.count_bigram(['h', 'e']), 0);
    assert_eq!(corpus.count_bigram(['h', '†']), 8729312);
    assert_eq!(corpus.count_bigram(['†', 'e']), 470925);
    assert_eq!(corpus.count_bigram(['†', 'a']), 538430);
    assert_eq!(corpus.count_bigram(['†', 'i']), 586723);
    assert_eq!(corpus.count_bigram(['†', 'o']), 51438);
    assert_eq!(corpus.count_bigram(['†', ' ']), 7001837);

    // Trigrams
    assert_eq!(corpus.count_trigram(['t', 'h', 'e']), 0);
    assert_eq!(corpus.count_trigram(['t', 'h', '†']), 6802477);
    assert_eq!(corpus.count_trigram(['h', 'e', ' ']), 0);
    assert_eq!(corpus.count_trigram(['h', '†', ' ']), 5421447);
    assert_eq!(corpus.count_trigram(['e', ' ', 'q']), 21049);
    // assert_eq!(corpus.count_trigram(['†', ' ', 'q']), 22957); // overflow
    assert_eq!(corpus.count_trigram(['e', ' ', 'l']), 210202);
    // assert_eq!(corpus.count_trigram(['†', ' ', 'l']), 258419); // Getting: 258419
    assert_eq!(corpus.count_trigram(['e', 'a', 'h']), 6357);
    assert_eq!(corpus.count_trigram(['†', 'a', 'h']), 60);
    // assert_eq!(corpus.count_trigram(['e', 'h', 'e']), 0); // Getting: 4294966979 (-316)

    // Skipgrams
    assert_eq!(corpus.count_skipgram(['t', 'e']), 960000);
    assert_eq!(corpus.count_skipgram(['t', '†']), 7872503);
    assert_eq!(corpus.count_skipgram(['e', 'q']), 25290);
    assert_eq!(corpus.count_skipgram(['†', 'q']), 23733);
    assert_eq!(corpus.count_skipgram(['e', 'l']), 942739);
    assert_eq!(corpus.count_skipgram(['†', 'l']), 467771);

    // Everything else the same
    assert_eq!(corpus.count_trigram(['o', 'v', 'e']), 222370);
    assert_eq!(corpus.count_trigram(['v', 'e', 'r']), 283);
    assert_eq!(corpus.count_trigram(['e', 'r', ' ']), 0);
    assert_eq!(corpus.count_skipgram(['o', 'e']), 3076975);

    // Requires adjustments based on pentagrams to handle skips over invalid corpus chars
    // assert_eq!(corpus.count_skipgram(['e', ' ']), 9021099);
}

#[cfg(test)]
fn verify_corpus_si_er_he(corpus: Corpus) {
    // he -> h†
    // er -> r† (whoops)

    // assert_eq!(corpus.count_trigram(['v', 'e', 'r']), 283); // Getting: 0 (seems right?)

    // Monograms
    assert_eq!(corpus.count_char('e'), 36549054);
    assert_eq!(corpus.count_char('†'), 13948468);

    // Bigrams
    assert_eq!(corpus.count_bigram(['h', 'e']), 0);
    assert_eq!(corpus.count_bigram(['h', '†']), 7571109);
    assert_eq!(corpus.count_bigram(['†', 'e']), 911133);
    assert_eq!(corpus.count_bigram(['†', 'a']), 548135);
    assert_eq!(corpus.count_bigram(['†', 'i']), 600863);
    assert_eq!(corpus.count_bigram(['†', 'o']), 59519);
    assert_eq!(corpus.count_bigram(['†', ' ']), 7510811);

    // Trigrams
    assert_eq!(corpus.count_trigram(['t', 'h', 'e']), 0);
    assert_eq!(corpus.count_trigram(['t', 'h', '†']), 6099092);
    assert_eq!(corpus.count_trigram(['h', 'e', ' ']), 0);
    assert_eq!(corpus.count_trigram(['h', '†', ' ']), 5421447);
    assert_eq!(corpus.count_trigram(['e', ' ', 'q']), 21049);
    assert_eq!(corpus.count_trigram(['†', ' ', 'q']), 24617); // Getting: 4294942679
    assert_eq!(corpus.count_trigram(['e', ' ', 'l']), 210202);
    assert_eq!(corpus.count_trigram(['†', ' ', 'l']), 273048);
    assert_eq!(corpus.count_trigram(['e', 'a', 'h']), 6357);
    assert_eq!(corpus.count_trigram(['†', 'a', 'h']), 61);
    assert_eq!(corpus.count_trigram(['e', 'h', 'e']), 0); // Getting: 4294966979 (-316)

    // Skipgrams
    assert_eq!(corpus.count_skipgram(['t', 'e']), 960000);
    assert_eq!(corpus.count_skipgram(['t', '†']), 7169118);
    assert_eq!(corpus.count_skipgram(['e', 'q']), 25290);
    assert_eq!(corpus.count_skipgram(['†', 'q']), 25448);
    assert_eq!(corpus.count_skipgram(['e', 'l']), 942739);
    assert_eq!(corpus.count_skipgram(['†', 'l']), 482350);

    // Everything else the same
    // assert_eq!(corpus.count_bigram(['v', 'e']), 2978051);
    // assert_eq!(corpus.count_bigram(['e', 'r']), 5219156);
    // assert_eq!(corpus.count_bigram(['e', 'o']), 252440);
    // assert_eq!(corpus.count_bigram(['e', 'i']), 291015);
    // assert_eq!(corpus.count_bigram(['e', 'h']), 85972);
    assert_eq!(corpus.count_trigram(['o', 'v', 'e']), 222370);
    assert_eq!(corpus.count_trigram(['v', 'e', 'r']), 283);
    assert_eq!(corpus.count_trigram(['e', 'r', ' ']), 0);
    assert_eq!(corpus.count_skipgram(['o', 'e']), 3076975);

    // Requires adjustments based on pentagrams to handle skips over invalid corpus chars
    // assert_eq!(corpus.count_skipgram(['e', ' ']), 9021099);
}

#[test]
#[ignore]
fn si_pre() {
    let b = fs::read("./corpora/shai-iweb.corpus").expect("couldn't read corpus file");
    let corpus: Corpus = rmp_serde::from_slice(&b).expect("couldn't deserialize corpus");
    verify_corpus_si_pre(corpus);
}

#[test]
#[ignore]
fn si_he() {
    let b = fs::read("./corpora/shai-iweb.corpus").expect("couldn't read corpus file");
    let mut corpus: Corpus = rmp_serde::from_slice(&b).expect("couldn't deserialize corpus");
    // corpus.adapt_ngrams(['h', 'e'], ['h', '†']);
    // <Corpus as AdaptiveCorpus<[char; 1]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    // <Corpus as AdaptiveCorpus<[char; 2]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 3]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    verify_corpus_si_he(corpus);
}

#[test]
fn si_he_er() {
    let b = fs::read("./corpora/shai-iweb.corpus").expect("couldn't read corpus file");
    let mut corpus: Corpus = rmp_serde::from_slice(&b).expect("couldn't deserialize corpus");
    <Corpus as AdaptiveCorpus<[char; 1]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 2]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 3]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 4]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);

    <Corpus as AdaptiveCorpus<[char; 1]>>::adapt_ngrams(&mut corpus, ['e', 'r'], ['r', '†']); // XXX
    <Corpus as AdaptiveCorpus<[char; 2]>>::adapt_ngrams(&mut corpus, ['e', 'r'], ['r', '†']); // XXX
    <Corpus as AdaptiveCorpus<[char; 3]>>::adapt_ngrams(&mut corpus, ['e', 'r'], ['r', '†']); // XXX
    <Corpus as AdaptiveCorpus<[char; 4]>>::adapt_ngrams(&mut corpus, ['e', 'r'], ['r', '†']); // XXX
    <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_ngrams(&mut corpus, ['e', 'r'], ['r', '†']); // XXX
    verify_corpus_si_he_er(corpus);
}

#[test]
fn si_er_he() {
    let b = fs::read("./corpora/shai-iweb.corpus").expect("couldn't read corpus file");
    let mut corpus: Corpus = rmp_serde::from_slice(&b).expect("couldn't deserialize corpus");
    <Corpus as AdaptiveCorpus<[char; 1]>>::adapt_ngrams(&mut corpus, ['e', 'r'], ['r', '†']); // XXX
    <Corpus as AdaptiveCorpus<[char; 2]>>::adapt_ngrams(&mut corpus, ['e', 'r'], ['r', '†']); // XXX
    <Corpus as AdaptiveCorpus<[char; 3]>>::adapt_ngrams(&mut corpus, ['e', 'r'], ['r', '†']); // XXX
    <Corpus as AdaptiveCorpus<[char; 4]>>::adapt_ngrams(&mut corpus, ['e', 'r'], ['r', '†']); // XXX
    <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_ngrams(&mut corpus, ['e', 'r'], ['r', '†']); // XXX

    <Corpus as AdaptiveCorpus<[char; 1]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 2]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 3]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 4]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    verify_corpus_si_er_he(corpus);
}

#[test]
fn si_he_ref() {
    let b = fs::read("./corpora/shai-iweb-he.corpus").expect("couldn't read corpus file");
    let corpus: Corpus = rmp_serde::from_slice(&b).expect("couldn't deserialize corpus");
    verify_corpus_si_he(corpus);
}

#[test]
fn si_he_er_ref() {
    let b = fs::read("./corpora/shai-iweb-he-er.corpus").expect("couldn't read corpus file");
    let corpus: Corpus = rmp_serde::from_slice(&b).expect("couldn't deserialize corpus");
    verify_corpus_si_he_er(corpus);
}

#[test]
fn si_er_he_ref() {
    let b = fs::read("./corpora/shai-iweb-er-he.corpus").expect("couldn't read corpus file");
    let corpus: Corpus = rmp_serde::from_slice(&b).expect("couldn't deserialize corpus");
    verify_corpus_si_er_he(corpus);
}

/// XXX: Can OOM in release-mode.
#[test]
#[ignore]
fn si_compare_all_ngrams() {
    let b = fs::read("./corpora/shai-iweb.corpus").expect("couldn't read corpus file");
    let mut corpus: Corpus = rmp_serde::from_slice(&b).expect("couldn't deserialize corpus");
    // corpus.adapt_ngrams(['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 1]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 2]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 3]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 4]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);

    let b = fs::read("./corpora/shai-iweb-he.corpus").expect("couldn't read corpus file");
    let ref_corpus: Corpus = rmp_serde::from_slice(&b).expect("couldn't deserialize corpus");

    // assert_eq!(ref_corpus.chars, corpus.chars);
    // assert_eq!(ref_corpus.bigrams, corpus.bigrams);
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

/// XXX: Can OOM in release-mode.
#[test]
fn si_he_er_compare_all_ngrams() {
    let b = fs::read("./corpora/shai-iweb.corpus").expect("couldn't read corpus file");
    let mut corpus: Corpus = rmp_serde::from_slice(&b).expect("couldn't deserialize corpus");
    // corpus.adapt_ngrams(['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 1]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 2]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 3]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 4]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
    <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);

    <Corpus as AdaptiveCorpus<[char; 1]>>::adapt_ngrams(&mut corpus, ['e', 'r'], ['r', '†']);
    <Corpus as AdaptiveCorpus<[char; 2]>>::adapt_ngrams(&mut corpus, ['e', 'r'], ['r', '†']);
    <Corpus as AdaptiveCorpus<[char; 3]>>::adapt_ngrams(&mut corpus, ['e', 'r'], ['r', '†']);
    <Corpus as AdaptiveCorpus<[char; 4]>>::adapt_ngrams(&mut corpus, ['e', 'r'], ['r', '†']);
    <Corpus as AdaptiveCorpus<[char; 5]>>::adapt_ngrams(&mut corpus, ['e', 'r'], ['r', '†']);

    let b = fs::read("./corpora/shai-iweb-he-er.corpus").expect("couldn't read corpus file");
    let ref_corpus: Corpus = rmp_serde::from_slice(&b).expect("couldn't deserialize corpus");

    // assert_eq!(ref_corpus.chars, corpus.chars);
    // assert_eq!(ref_corpus.bigrams, corpus.bigrams);
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
