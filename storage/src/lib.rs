use tantivy::{
    Index,
    IndexWriter,
    schema::*,
    IndexReader,
    ReloadPolicy,
    query::RegexQuery,
    collector::TopDocs,
    DocAddress,
    doc
};
use tempfile::TempDir;

use music_player_types::types::Song;
use music_player_settings::{read_settings, Settings};
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct Database {
    pub connection: DatabaseConnection,
    pub searcher: Searcher
}

#[derive(Clone)]
pub struct Searcher {
    pub schema: Schema,
    pub index: Index,
    pub reader: IndexReader
}

impl Searcher {
    pub fn new() -> Searcher {
        let index_path: TempDir = TempDir::new().expect("Error creating Temporal Directory");
        let mut schema_builder: SchemaBuilder = Schema::builder();

        schema_builder.add_text_field("id", TEXT | STORED);
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("artist", TEXT | STORED);
        schema_builder.add_text_field("album", TEXT);
        schema_builder.add_text_field("genre", TEXT);

        let schema: Schema = schema_builder.build();
        let index: Index = Index::create_in_dir(&index_path, schema.clone()).unwrap();
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()
            .expect("Tantivy reader couldn't be created");

        Searcher {
            schema: schema.clone(),
            index,
            reader
        }
    }

    pub fn get_schema(&self) -> Schema {
        return self.schema.clone();
    }

    pub fn get_index(&self) -> Index {
        return self.index.clone();
    }

    pub fn insert_song(&self, song: Song, str_id: String) -> tantivy::Result<()> {
        let mut index_writer: IndexWriter = self.index.writer(50_000_000).unwrap();

        let id = self.schema.get_field("id").unwrap();
        let title = self.schema.get_field("title").unwrap();
        let artist = self.schema.get_field("artist").unwrap();
        let album = self.schema.get_field("album").unwrap();
        let genre = self.schema.get_field("genre").unwrap();

        let doc: Document = doc!(
            id => str_id,
            title => song.title.clone(),
            artist => song.artist.clone(),
            album => song.album.clone(),
            genre => song.genre.clone()
        );

        index_writer.add_document(doc)?;
        index_writer.commit()?;

        Ok(())
    }

    pub fn search_by_title(&self, s: String) -> tantivy::Result<Vec<(f32, DocAddress)>> {
        let searcher = self.reader.searcher();
        let title = self.schema.get_field("title").unwrap();
        let s = format!("[a-zA-Z0-9]*{}[a-zA-Z0-9]*", s);

        let regex_query_title = RegexQuery::from_pattern(&s, title)?;
        let top_docs = searcher.search(&regex_query_title, &TopDocs::with_limit(10))?;

        Ok(top_docs)
    }

    pub fn search_by_artist(&self, s: String) -> tantivy::Result<Vec<(f32, DocAddress)>> {
        let searcher = self.reader.searcher();
        let artist = self.schema.get_field("artist").unwrap();
        let s = format!("[a-zA-Z0-9]*{}[a-zA-Z0-9]*", s);

        let regex_query_artist = RegexQuery::from_pattern(&s, artist)?;
        let top_docs = searcher.search(&regex_query_artist, &TopDocs::with_limit(10))?;

        Ok(top_docs)
    }
    
    pub fn search_by_album(&self, s: String) -> tantivy::Result<Vec<(f32, DocAddress)>> {
        let searcher = self.reader.searcher();
        let album = self.schema.get_field("album").unwrap();
        let s = format!("[a-zA-Z0-9]*{}[a-zA-Z0-9]*", s);

        let regex_query_album = RegexQuery::from_pattern(&s, album)?;
        let top_docs = searcher.search(&regex_query_album, &TopDocs::with_limit(10))?;

        Ok(top_docs)
    }
        
    pub fn search_by_genre(&self, s: String) -> tantivy::Result<Vec<(f32, DocAddress)>> {
        let searcher = self.reader.searcher();
        let genre = self.schema.get_field("genre").unwrap();
        let s = format!("[a-zA-Z0-9]*{}[a-zA-Z0-9]*", s);

        let regex_query_genre = RegexQuery::from_pattern(&s, genre)?;
        let top_docs = searcher.search(&regex_query_genre, &TopDocs::with_limit(10))?;

        Ok(top_docs)
    }
}

impl Database {
    pub async fn new() -> Database {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();

        let connection = sea_orm::Database::connect(settings.database_url)
            .await
            .expect("Could not connect to database");

        let searcher = Searcher::new();

        Database { connection, searcher }
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub fn get_searcher(&self) -> &Searcher {
        &self.searcher
    }
}
