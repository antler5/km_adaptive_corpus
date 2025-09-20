// Copyright 2025 antlers <antlers@illucid.net>
//
// SPDX-License-Identifier: GPL-3.0-only

//! Trait CorpusExt is just a wrapper around struct Corpus's fields.

pub(crate) use kc::Corpus;

/// Provides trait implementations on Corpus access to it's struct fields.
pub(crate) trait CorpusExt {
    fn corpus_bigram(&mut self, bigram: &[char; 2]) -> usize;
    fn corpus_trigram(&mut self, trigram: &[char; 3]) -> usize;
    fn corpus_quadgram(&mut self, trigram: &[char; 4]) -> usize;
    fn corpus_pentagram(&mut self, trigram: &[char; 5]) -> usize;
    fn get_bigrams(&mut self) -> &mut Vec<u32>;
    fn get_trigrams(&mut self) -> &mut Vec<u32>;
    fn get_skipgrams(&mut self) -> &mut Vec<u32>;
    fn get_quadgrams(&mut self) -> &mut Vec<u32>;
    fn get_pentagrams(&mut self) -> &mut Vec<u32>;
    #[cfg(test)]
    fn count_char(&self, c: char) -> u32;
    #[cfg(test)]
    fn count_bigram(&self, bg: [char; 2]) -> u32;
    #[cfg(test)]
    fn count_trigram(&self, tg: [char; 3]) -> u32;
    #[cfg(test)]
    fn count_skipgram(&self, sg: [char; 2]) -> u32;
}

impl CorpusExt for Corpus {
    fn corpus_bigram(&mut self, bigram: &[char; 2]) -> usize {
        Corpus::corpus_bigram(self, bigram)
    }
    fn corpus_trigram(&mut self, trigram: &[char; 3]) -> usize {
        Corpus::corpus_trigram(self, trigram)
    }
    fn corpus_quadgram(&mut self, quadgram: &[char; 4]) -> usize {
        Corpus::corpus_quadgram(self, quadgram)
    }
    fn corpus_pentagram(&mut self, pentagram: &[char; 5]) -> usize {
        Corpus::corpus_pentagram(self, pentagram)
    }
    fn get_bigrams(&mut self) -> &mut Vec<u32> {
        &mut self.bigrams
    }
    fn get_trigrams(&mut self) -> &mut Vec<u32> {
        &mut self.trigrams
    }
    fn get_skipgrams(&mut self) -> &mut Vec<u32> {
        &mut self.skipgrams
    }
    fn get_quadgrams(&mut self) -> &mut Vec<u32> {
        &mut self.quadgrams
    }
    fn get_pentagrams(&mut self) -> &mut Vec<u32> {
        &mut self.pentagrams
    }

    #[cfg(test)]
    fn count_char(&self, c: char) -> u32 {
        self.chars[self.corpus_char(c)]
    }

    #[cfg(test)]
    fn count_bigram(&self, bg: [char; 2]) -> u32 {
        self.bigrams[self.corpus_bigram(&bg)]
    }

    #[cfg(test)]
    fn count_trigram(&self, tg: [char; 3]) -> u32 {
        self.trigrams[self.corpus_trigram(&tg)]
    }

    #[cfg(test)]
    fn count_skipgram(&self, sg: [char; 2]) -> u32 {
        self.skipgrams[self.corpus_bigram(&sg)]
    }
}
