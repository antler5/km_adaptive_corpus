# km\_adaptive\_corpus

Dynamic bigram substitutions for Keycat, based on a Python POC by ClemenPine @ AKL.

This was a pretty early Learning Project for me, and at this rate I won't even
finish refactoring the interface, which is currently called like this (lol):

```rust
<Corpus as AdaptiveCorpus<[char; 1]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
<Corpus as AdaptiveCorpus<[char; 2]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
<Corpus as AdaptiveCorpus<[char; 3]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
<Corpus as AdaptiveCorpus<[char; 4]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
<Corpus as AdaptiveCorpus<[char; 5]>>::adapt_ngrams(&mut corpus, ['h', 'e'], ['h', '†']);
```

So I almost cleaned that up, and almost refactored the five trait impls into
five macro calls, but I've got what I needed -- might re-write later, with a
sane interface, maybe a flexable slices-based impl, but this'll do for now.

POC achieved! A dozen bigram substitution rules apply up through trigram depth
of a pentagram corpus in 400ms on my machine. I think there is quite a lot of
optimization on the table, but will have to come back to it more prepared.

# License

Code is GPL-3.0-only.

Images and documentation are CC-BY-SA 4.0.

Trivial files may be explictly licensed as CC0-1.0.

