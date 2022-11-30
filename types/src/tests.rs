use super::types::*;
use tantivy::{
    schema::{Schema, SchemaBuilder, STORED, STRING, TEXT},
    Document,
};

#[test]
fn document_to_artist() {
    let mut schema_builder: SchemaBuilder = Schema::builder();

    let id_field = schema_builder.add_text_field("id", STRING | STORED);
    let name_field = schema_builder.add_text_field("name", TEXT | STORED);

    let schema = schema_builder.build();

    let mut doc = Document::default();
    doc.add_text(id_field, "id");
    doc.add_text(name_field, "name");

    let artist = Artist::from(doc);

    assert_eq!(artist.id, "id");
    assert_eq!(artist.name, "name");
}
