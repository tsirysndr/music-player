pub mod addons;
pub mod core;
pub mod history;
pub mod library;
pub mod mixer;
pub mod playback;
pub mod playlist;
pub mod server;
pub mod tracklist;
pub mod event;
pub mod api {
    pub mod v1alpha1 {
        tonic::include_proto!("music.v1alpha1");
    }
}

pub mod objects {
    pub mod v1alpha1 {
        tonic::include_proto!("objects.v1alpha1");
    }
}

pub mod metadata {
    pub mod v1alpha1 {
        tonic::include_proto!("metadata.v1alpha1");
    }
}

