use sea_orm::DatabaseConnection;

pub struct FolderRepository {
    db: DatabaseConnection,
}

impl FolderRepository {
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }

    pub async fn find(&self) {}

    pub async fn find_all(&self) {}
}
