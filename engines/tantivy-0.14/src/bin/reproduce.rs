use tantivy;
use tantivy::query::{Query, TermQuery, QueryParser};
use tantivy::{Term, TERMINATED, Searcher, DocSet, SegmentId, DocId, Score};
use tantivy::schema::IndexRecordOption;
use tantivy::query::Scorer;
use futures::executor::block_on;

// fn test_block_wand_aux(term_query: &TermQuery, searcher: &Searcher) -> tantivy::Result<()> {
//     let term_weight = term_query.specialized_weight(&searcher, true);
//     for (i, reader) in searcher.segment_readers().iter().enumerate() {
//         println!("-------- #{}", i);
//         let mut block_max_scores = vec![];
//         let mut block_max_scores_b = vec![];
//         let mut docs = vec![];
//         {
//             let mut term_scorer = term_weight.specialized_scorer(reader, 1.0f32)?;
//             let num_docs = (0u32..term_scorer.doc_freq() as u32).collect::<Vec<u32>>();
//             for _ in 0..128 {
//                 let mut score = 0.0f32;
//                 docs.push(term_scorer.doc());
//                 for i in 0..128 {
//                     score = score.max(term_scorer.score());
//                     term_scorer.advance();
//                 }
//                 block_max_scores.push(score);
//             }
//         }
//         dbg!(&docs);
//         {
//             for d in docs {
//                 let mut term_scorer = term_weight.specialized_scorer(reader, 1.0f32)?;
//
//                 {
//                 let skip_reader = &term_scorer.postings.block_cursor.skip_reader;
//                 dbg!((skip_reader.last_doc_in_previous_block, skip_reader.last_doc_in_block));
//             }
//                 term_scorer.shallow_seek(d);
//                 block_max_scores_b.push(term_scorer.block_max_score());
//             }
//         }
//
//         for (l, r) in block_max_scores.iter().cloned().zip(block_max_scores_b.iter().cloned()) {
//             dbg!((l, r));
//             assert!(2f32 * (l - r).abs() / (l.abs() + r.abs()) < 0.00001, "{} {}", l, r);
//         }
//     }
//     Ok(())
// }

fn score(query: &dyn Query, searcher: &Searcher, doc: DocId) -> tantivy::Result<Score> {
    let weight = query.weight(&searcher, true)?;
    let mut scorer = weight.scorer(searcher.segment_reader(0), 1.0f32)?;
    assert_eq!(scorer.seek(537_388), 537_388);
    Ok(scorer.score())
}

fn main() -> tantivy::Result<()> {
    let index = tantivy::Index::open_in_dir("idx")?;
    let field = index.schema().get_field("text").unwrap();
    let query_parser = QueryParser::for_index(&index, vec![field]);

    let central_query = TermQuery::new(Term::from_field_text(field, "central"), IndexRecordOption::WithFreqs);
    let community_query = TermQuery::new(Term::from_field_text(field, "community"), IndexRecordOption::WithFreqs);
    let college = TermQuery::new(Term::from_field_text(field, "college"), IndexRecordOption::WithFreqs);


    let mut query = query_parser.parse_query("central community college")?;
    let reader = index.reader()?;
    reader.reload()?;
    let searcher = reader.searcher();

    let doc = 537_388;
    let central_score = score(&central_query, &searcher, doc)? as f64;
    let community_score = score(&community_query, &searcher, doc)? as f64;
    let college_score = score(&college, &searcher, doc)? as f64;

    dbg!(central_score);
    dbg!(community_score);
    dbg!(college_score);

    dbg!(central_score + community_score + college_score);
    dbg!(community_score + college_score + central_score );
    Ok(())
}
