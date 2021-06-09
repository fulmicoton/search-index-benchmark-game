
# Welcome to Search Benchmark, the Game!

This repository is standardized benchmark for comparing the speed of various
aspects of search engine technologies.

The results are available [here](https://tantivy-search.github.io/bench/).

This benchmark is both
- **for users** to make it easy for users to compare different libraries
- **for library** developers to identify optimization opportunities by comparing
their implementation to other implementations.

Currently, the benchmark only includes Lucene and tantivy.
It is reasonably simple to add another engine.

You are free to communicate about the results of this benchmark **in
a reasonable manner**.
For instance, twisting this benchmark in marketing material to claim that your search engine is 31x faster than Lucene,
because your product was 31x on one of the test is not tolerated. If this happens, the benchmark will publicly
host a wall of shame.
Bullshit claims about performance are a plague in the database world.


## The benchmark

Different search engine implementation are benched over different real-life tests.
The corpus used is the English wikipedia. Stemming is disabled. Queries have been derived
 from the [AOL query dataset](https://en.wikipedia.org/wiki/AOL_search_data_leak)
 (but do not contain any personal information).

Out of a random sample of query, we filtered queries that had at least two terms and yield at least 1 hit when searches as
a phrase query.

For each of these query, we then run them as :
- `intersection`
- `unions`
- `phrase queries`

with the following collection options :
- `COUNT` only count documents, no need to score them
- `TOP 10` : Identify the 10 documents with the best BM25 score.
- `TOP 10 + COUNT`: Identify the 10  documents with the best BM25 score, and count the matching documents.

We also reintroduced artificially a couple of term queries with different term frequencies.

All tests are run once in order to make sure that
- all of the data is loaded and in page cache
- Java's JIT already kicked in.

Test are run in a single thread.
Out of 5 runs, we only retain the best score, so Garbage Collection likely does not matter.


## Engine specific detail

### Lucene

- Query cache is disabled.
- Maxscore is not yet used in Top 10. It should give a nice boost once Lucene 8 is released.
- GC should not influence the results as we pick the best out of 5 runs.
- JVM used was openjdk 10.0.1 2018-04-17

### Tantivy

- Tantivy returns slightly more results because its tokenizer handles apostrophes differently.
- Tantivy and Lucene both use BM25 and should return almost identical scores.


# Reproducing

These instructions will get you a copy of the project up and running on your local machine.

### Prerequisites

The lucene benchmarks requires java and Gradle. This can be installed from [the Gradle website](https://gradle.org/).
The tantivy benchmarks and benchmark driver code requires Cargo. This can be installed using [rustup](https://www.rustup.rs/).

### Installing

Clone this repo.

```
git clone git@github.com:tantivy-search/search-benchmark-game.git
```

## Running

Checkout the [Makefile](Makefile) for all available commands. You can adjust the `ENGINES` parameter for a different set of engines.

Run `make corpus` to download and unzip the corpus used in the benchmark.
```
make corpus
```

Run `make index` to create the indices for the engines.

```
make index
```

Run `make compile`, to build the search binaries, and then `make bench`, to build the different project and run the benches.
This command may take more than 30mn.

```
make bench
```

The results are outputted in a `results.json` file.

You can then check your results out by running:

```
make serve
```

And open the following in your browser: [http://localhost:8000/](http://localhost:8000/)


# Adding another search engine

See `CONTRIBUTE.md`.
