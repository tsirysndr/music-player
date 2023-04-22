use itertools::Itertools;
use music_player_settings::get_application_directory;
use music_player_types::types::Artist;
use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    doc,
    query::{FuzzyTermQuery, PhraseQuery, TermQuery},
    schema::{IndexRecordOption, Schema, SchemaBuilder, STORED, STRING, TEXT},
    Document, Index, IndexReader, ReloadPolicy, Term,
};
#[derive(Clone)]
pub struct ArtistSearcher {
    schema: Schema,
    index: Index,
    reader: IndexReader,
}

impl ArtistSearcher {
    pub fn new() -> Self {
        let index_path = format!("{}/artists", get_application_directory());
        let mut schema_builder: SchemaBuilder = Schema::builder();

        schema_builder.add_text_field("id", STRING | STORED);
        schema_builder.add_text_field("name", TEXT | STORED);

        let schema: Schema = schema_builder.build();
        let dir = MmapDirectory::open(&index_path).unwrap();
        let index: Index = Index::open_or_create(dir, schema.clone()).unwrap();
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()
            .expect("Tantivy reader couldn't be created");

        Self {
            schema: schema.clone(),
            index,
            reader,
        }
    }

    pub fn get_schema(&self) -> Schema {
        return self.schema.clone();
    }

    pub fn get_index(&self) -> Index {
        return self.index.clone();
    }

    pub fn insert(&self, artist: Artist) -> tantivy::Result<()> {
        let id = self.schema.get_field("id").unwrap();
        let name = self.schema.get_field("name").unwrap();

        // Check if the artist already exists
        let searcher = self.reader.searcher();
        let query = TermQuery::new(
            Term::from_field_text(id, &artist.id),
            IndexRecordOption::Basic,
        );
        let top_docs = searcher.search(&query, &TopDocs::with_limit(1))?;
        if top_docs.len() > 0 {
            return Ok(());
        }

        let doc: Document = doc!(
            id => artist.id.clone(),
            name => artist.name.clone(),
        );
        let mut writer = self.index.writer_with_num_threads(64, 192_000_000)?;
        writer.add_document(doc)?;
        writer.commit()?;
        Ok(())
    }

    pub fn search(&self, term: &str) -> tantivy::Result<Vec<Artist>> {
        self.search_by_name(term)
    }

    pub fn search_by_name(&self, s: &str) -> tantivy::Result<Vec<Artist>> {
        let searcher = self.reader.searcher();
        let name = self.schema.get_field("name").unwrap();

        let top_docs = match s.split(" ").collect::<Vec<&str>>().len() {
            1 => {
                let term = Term::from_field_text(name, s);
                let query = FuzzyTermQuery::new(term, 1, true);
                searcher.search(&query, &TopDocs::with_limit(10))?
            }
            _ => {
                let query = PhraseQuery::new(
                    s.split(" ")
                        .map(|s| Term::from_field_text(name, s))
                        .collect(),
                );
                searcher.search(&query, &TopDocs::with_limit(10))?
            }
        };
        let results: Vec<Artist> = top_docs
            .into_iter()
            .map(|(_, y)| Into::into(searcher.doc(y).unwrap()))
            .collect();
        Ok(results.into_iter().unique_by(|x| x.id.clone()).collect())
    }
}
