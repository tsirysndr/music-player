use itertools::Itertools;
use music_player_settings::get_application_directory;
use music_player_types::types::{SimplifiedSong, Song};
use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    doc,
    query::{FuzzyTermQuery, PhraseQuery},
    schema::{Schema, SchemaBuilder, STORED, STRING, TEXT},
    Document, Index, IndexReader, IndexWriter, ReloadPolicy, Term,
};
#[derive(Clone)]
pub struct TrackSearcher {
    pub schema: Schema,
    pub index: Index,
    pub reader: IndexReader,
}

impl TrackSearcher {
    pub fn new() -> Self {
        let index_path = format!("{}/tracks", get_application_directory());

        let mut schema_builder: SchemaBuilder = Schema::builder();

        schema_builder.add_text_field("id", STRING | STORED);
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("artist", TEXT | STORED);
        schema_builder.add_text_field("album", TEXT | STORED);
        schema_builder.add_text_field("genre", TEXT);
        schema_builder.add_text_field("cover", STRING | STORED);
        schema_builder.add_i64_field("duration", STORED);

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

    pub fn insert(&self, song: Song, str_id: &str) -> tantivy::Result<()> {
        let mut index_writer: IndexWriter = self.index.writer(50_000_000).unwrap();

        let id = self.schema.get_field("id").unwrap();
        let title = self.schema.get_field("title").unwrap();
        let artist = self.schema.get_field("artist").unwrap();
        let album = self.schema.get_field("album").unwrap();
        let genre = self.schema.get_field("genre").unwrap();
        let cover = self.schema.get_field("cover").unwrap();
        let duration = self.schema.get_field("duration").unwrap();

        let time = song.duration.as_secs_f32() as i64;

        let doc: Document = doc!(
            id => str_id,
            title => song.title.clone(),
            artist => song.artist.clone(),
            album => song.album.clone(),
            genre => song.genre.clone(),
            cover => song.cover.unwrap_or_default().clone(),
            duration => time,
        );

        index_writer.add_document(doc)?;
        index_writer.commit()?;

        Ok(())
    }

    pub fn search(&self, term: &str) -> tantivy::Result<Vec<SimplifiedSong>> {
        let result_by_title = self.search_by_title(term)?;
        let result_by_artist = self.search_by_artist(term)?;
        let result_by_album = self.search_by_album(term)?;
        let result_by_genre = self.search_by_genre(term)?;
        Ok([
            result_by_title,
            result_by_artist,
            result_by_album,
            result_by_genre,
        ]
        .concat()
        .into_iter()
        .unique_by(|x| x.id.clone())
        .collect())
    }

    pub fn search_by_title(&self, s: &str) -> tantivy::Result<Vec<SimplifiedSong>> {
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
            .into_iter()
            .map(|(_, x)| Into::into(searcher.doc(x).unwrap()))
            .collect())
    }

    pub fn search_by_artist(&self, s: &str) -> tantivy::Result<Vec<SimplifiedSong>> {
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
            .into_iter()
            .map(|(_, x)| Into::into(searcher.doc(x).unwrap()))
            .collect())
    }

    pub fn search_by_album(&self, s: &str) -> tantivy::Result<Vec<SimplifiedSong>> {
        let searcher = self.reader.searcher();
        let album = self.schema.get_field("album").unwrap();

        let top_docs = match s.split(" ").collect::<Vec<&str>>().len() {
            1 => {
                let term = Term::from_field_text(album, s);
                let query = FuzzyTermQuery::new(term, 1, true);
                searcher.search(&query, &TopDocs::with_limit(10))?
            }
            _ => {
                let query = PhraseQuery::new(
                    s.split(" ")
                        .map(|s| Term::from_field_text(album, s))
                        .collect(),
                );
                searcher.search(&query, &TopDocs::with_limit(10))?
            }
        };

        Ok(top_docs
            .into_iter()
            .map(|(_, x)| Into::into(searcher.doc(x).unwrap()))
            .collect())
    }

    pub fn search_by_genre(&self, s: &str) -> tantivy::Result<Vec<SimplifiedSong>> {
        let searcher = self.reader.searcher();
        let genre = self.schema.get_field("genre").unwrap();

        let top_docs = match s.split(" ").collect::<Vec<&str>>().len() {
            1 => {
                let term = Term::from_field_text(genre, s);
                let query = FuzzyTermQuery::new(term, 1, true);
                searcher.search(&query, &TopDocs::with_limit(10))?
            }
            _ => {
                let query = PhraseQuery::new(
                    s.split(" ")
                        .map(|s| Term::from_field_text(genre, s))
                        .collect(),
                );
                searcher.search(&query, &TopDocs::with_limit(10))?
            }
        };

        Ok(top_docs
            .into_iter()
            .map(|(_, x)| Into::into(searcher.doc(x).unwrap()))
            .collect())
    }
}
