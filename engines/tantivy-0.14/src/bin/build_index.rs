use tantivy::schema::{Schema, STORED, TEXT};
use tantivy::Index;
use std::env;
use std::io::BufRead;
use std::path::Path;
use futures::executor::block_on;

fn main() {
    let args: Vec<String> = env::args().collect();
    main_inner(&Path::new(&args[1])).unwrap();
}

fn create_schema() -> Schema {
  let mut schema_builder = Schema::builder();
  schema_builder.add_text_field("id", STORED);
  schema_builder.add_text_field("text", TEXT);
  schema_builder.build()
}

fn main_inner(output_dir: &Path) -> tantivy::Result<()> {
    env_logger::init();

    let schema = create_schema();
    let index = Index::create_in_dir(output_dir, schema.clone()).expect("failed to create index");

    let mut i = 0;
    {
        let mut index_writer = index.writer_with_num_threads(4, 2_000_000_000).expect("failed to create index writer");
        let stdin = std::io::stdin();

        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            i += 1;
            if i % 100_000 == 0 {
                println!("{}", i);
            }
            let doc = schema.parse_document(&line)?;
            index_writer.add_document(doc);
        }

        index_writer.commit()?;
        index_writer.wait_merging_threads()?;
    }
        let segment_ids = index.searchable_segment_ids()?;
        let mut index_writer = index.writer(1_500_000_000).expect("failed to create index writer");
        block_on(index_writer.merge(&segment_ids))?;
        block_on(index_writer.garbage_collect_files())?;
    Ok(())
}
