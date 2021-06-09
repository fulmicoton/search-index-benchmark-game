#![macro_use]
extern crate tantivy;

use tantivy::{Index, SegmentReader, Score, DocId, TERMINATED};
use tantivy::query::{QueryParser, Weight};
use tantivy::tokenizer::TokenizerManager;
use tantivy::collector::{Count, TopDocs};

use std::env;
use std::io::BufRead;
use std::path::Path;
use std::collections::BinaryHeap;

fn main() {
    let args: Vec<String> = env::args().collect();
    main_inner(&Path::new(&args[1])).unwrap()
}

struct Float(Score);

use std::cmp::Ordering;


impl Eq for Float {
}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(&other) == Ordering::Equal
    }
}

impl PartialOrd for Float {
   fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
   }
}

impl Ord for Float {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
    }
}

fn checkpoints_pruning(weight: &dyn Weight, reader: &SegmentReader, n: usize) -> tantivy::Result<Vec<(DocId, Score, Score)>> {
    let mut heap: BinaryHeap<Float> = BinaryHeap::with_capacity(n);
    let mut checkpoints: Vec<(DocId, Score, Score)> = Vec::new();
    let mut limit: Score = 0.0;
    weight.for_each_pruning(Score::MIN, reader, &mut |doc, score| {
        checkpoints.push((doc,score, score));
        // println!("pruning doc={} score={} limit={}", doc, score, limit);
        heap.push(Float(score));
        if heap.len() > n {
            heap.pop().unwrap();
        }
        limit = heap.peek().unwrap().0;
        return limit;
    })?;
    Ok(checkpoints)
}

fn checkpoints_no_pruning(weight: &dyn Weight, reader: &SegmentReader, n: usize) -> tantivy::Result<Vec<(DocId, Score, Score)>> {
    let mut heap: BinaryHeap<Float> = BinaryHeap::with_capacity(n);
    let mut checkpoints: Vec<(DocId, Score, Score)> = Vec::new();
    let mut scorer = weight.scorer(reader, 1.0)?;
    let mut limit = Score::MIN;
    loop {
        if scorer.doc() == TERMINATED {
            break;
        }
        let doc = scorer.doc();
        let score = scorer.score();
        if score > limit {
            // println!("nopruning doc={} score={} limit={}", doc, score, limit);
            checkpoints.push((doc, limit, score));
            heap.push(Float(score));
            if heap.len() > n {
                heap.pop().unwrap();
            }
            limit = heap.peek().unwrap().0;
        }
        scorer.advance();
    }
    Ok(checkpoints)
}

fn assert_nearly_equals(left: Score, right: Score) -> bool {
    (left - right).abs() * 2.0 / (left + right).abs() < 0.000001
}

fn main_inner(index_dir: &Path) -> tantivy::Result<()> {
    let index = Index::open_in_dir(index_dir).expect("failed to open index");
    let text_field = index.schema().get_field("text").expect("no all field?!");
    let query_parser = QueryParser::new(
        index.schema(),
        vec![text_field],
        TokenizerManager::default());
    let reader = index.reader()?;
    let searcher = reader.searcher();

    let stdin = std::io::stdin();
    for line_res in stdin.lock().lines() {
        let line = line_res?;
        let fields: Vec<&str> = line.split("\t").collect();
        assert_eq!(fields.len(), 2, "Expected a line in the format <COMMAND> query.");
        let command = fields[0];
        let query = query_parser.parse_query(fields[1])?;
        let count;
        match command {
            "COUNT" => {
                count = query.count(&searcher)?;
            }
            "TOP_10" => {
                let _top_k = searcher.search(&query, &TopDocs::with_limit(10))?;
                count = 1;
            }
            "TOP_10_COUNT" => {
                let (_top_k, count_) = searcher.search(&query, &(TopDocs::with_limit(10), Count))?;
                count = count_;
            }
            "DEBUG_TOP_10" => {
                let weight = query.weight(&searcher, true)?;
                for reader in searcher.segment_readers() {
                    let checkpoints_left =
                        checkpoints_no_pruning(&*weight, reader, 10)?;
                    let checkpoints_right =
                        checkpoints_pruning(&*weight, reader, 10)?;
               }
               count = 0;
            }
            _ => {
                println!("UNSUPPORTED");
                continue;
            }
        }
        println!("{}", count);
    }

    Ok(())
}
