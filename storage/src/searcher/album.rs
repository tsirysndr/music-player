use itertools::Itertools;
use music_player_settings::get_application_directory;
use music_player_types::types::Album;
use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    doc,
    query::{FuzzyTermQuery, PhraseQuery},
    schema::{Schema, SchemaBuilder, STORED, STRING, TEXT},
    Document, Index, IndexReader, ReloadPolicy, Term,
};

#[derive(Clone)]
pub struct AlbumSearcher {
    schema: Schema,
    index: Index,
    reader: IndexReader,
}

impl AlbumSearcher {
    pub fn new() -> Self {
        let index_path = format!("{}/albums", get_application_directory());
        let mut schema_builder: SchemaBuilder = Schema::builder();

        schema_builder.add_text_field("id", STRING | STORED);
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("artist", TEXT | STORED);
        schema_builder.add_i64_field("year", STORED);
        schema_builder.add_text_field("cover", STRING | STORED);

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

    pub fn insert(&self, album: Album) -> tantivy::Result<()> {
        let id = self.schema.get_field("id").unwrap();
        let title = self.schema.get_field("title").unwrap();
        let artist = self.schema.get_field("artist").unwrap();
        let year = self.schema.get_field("year").unwrap();
        let cover = self.schema.get_field("cover").unwrap();

        let doc: Document = doc!(
            id => album.id.clone(),
            title => album.title.clone(),
            artist => album.artist.clone(),
            year => i64::try_from(album.year.unwrap_or(0)).unwrap(),
            cover => album.cover.unwrap_or_default().clone()
        );

        let mut writer = self.index.writer_with_num_threads(64, 192_000_000).unwrap();
        writer.add_document(doc)?;
        writer.commit()?;
        Ok(())
    }

    pub fn search(&self, term: &str) -> tantivy::Result<Vec<Album>> {
        let result_by_artist = self.search_by_artist(term).unwrap_or(vec![]);
        let result_by_title = self.search_by_title(term).unwrap_or(vec![]);
        Ok([result_by_artist, result_by_title]
            .concat()
            .into_iter()
            .unique_by(|x| x.id.clone())
            .collect())
    }

    pub fn search_by_title(&self, s: &str) -> tantivy::Result<Vec<Album>> {
        let searcher = self.reader.searcher();
        let title = self.schema.get_field("title").unwrap();

        let top_docs = match s.split(" ").collect::<Vec<&str>>().len() {
            1 => {
                let term = Term::from_field_text(title, s);
                let query = FuzzyTermQuery::new(term, 1, true);
                searcher.search(&query, &TopDocs::with_limit(10))?
            }
            _ => {
                let query = PhraseQuery::new(
                    s.split(" ")
                        .map(|s| Term::from_field_text(title, s))
                        .collect(),
                );
                searcher.search(&query, &TopDocs::with_limit(10))?
            }
        };

        Ok(top_docs
            .clone()
            .into_iter()
            .map(|(_, y)| Into::into(searcher.doc(y).unwrap()))
            .collect::<Vec<Album>>())
    }

    pub fn search_by_artist(&self, s: &str) -> tantivy::Result<Vec<Album>> {
        let searcher = self.reader.searcher();
        let artist = self.schema.get_field("artist").unwrap();

        let top_docs = match s.split(" ").collect::<Vec<&str>>().len() {
            1 => {
                let term = Term::from_field_text(artist, s);
                let query = FuzzyTermQuery::new(term, 1, true);
                searcher.search(&query, &TopDocs::with_limit(10))?
            }
            _ => {
                let query = PhraseQuery::new(
                    s.split(" ")
                        .map(|s| Term::from_field_text(artist, s))
                        .collect(),
                );
                searcher.search(&query, &TopDocs::with_limit(10))?
            }
        };

        Ok(top_docs
            .clone()
            .into_iter()
            .map(|(_, y)| Into::into(searcher.doc(y).unwrap()))
            .collect::<Vec<Album>>())
    }
}
