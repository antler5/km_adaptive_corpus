//! Trait CorpusExt is just a wrapper around struct Corpus's fields.

pub(crate) use kc::Corpus;

/// Provides trait implementations on Corpus access to it's struct fields.
pub(crate) trait CorpusExt {
    fn corpus_bigram(&mut self, bigram: &[char; 2]) -> usize;
    fn corpus_trigram(&mut self, trigram: &[char; 3]) -> usize;
    fn get_trigrams(&mut self) -> &mut Vec<u32>;
    fn get_skipgrams(&mut self) -> &mut Vec<u32>;
}

impl CorpusExt for Corpus {
    fn corpus_bigram(&mut self, bigram: &[char; 2]) -> usize {
        Corpus::corpus_bigram(self, bigram)
    }
    fn corpus_trigram(&mut self, trigram: &[char; 3]) -> usize {
        Corpus::corpus_trigram(self, trigram)
    }
    fn get_trigrams(&mut self) -> &mut Vec<u32> {
        &mut self.trigrams
    }
    fn get_skipgrams(&mut self) -> &mut Vec<u32> {
        &mut self.skipgrams
    }
}
