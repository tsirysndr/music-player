use music_player_entity::folder::Model;

use crate::schema::objects::folder::Folder;

#[test]
fn model_to_folder() {
    let model = Model {
        id: "id".to_string(),
        name: "name".to_string(),
        ..Default::default()
    };
    let folder: Folder = model.into();
    assert_eq!(folder.id, "id");
    assert_eq!(folder.name, "name");
}
