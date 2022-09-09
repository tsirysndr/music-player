pub mod metadata {
    pub mod v1alpha1 {
        tonic::include_proto!("metadata.v1alpha1");
    }
}

pub mod objects {
    pub mod v1alpha1 {
        tonic::include_proto!("objects.v1alpha1");
    }
}
pub mod api {
    pub mod v1alpha1 {
     tonic::include_proto!("music.v1alpha1");
    }
}

/*

pub mod metadata {
    pub mod v1alpha1 {
        include!("metadata.v1alpha1.rs");
    }
}
pub mod objects {
    pub mod v1alpha1 {
        include!("objects.v1alpha1.rs");
    }
}
pub mod music {
    pub mod v1alpha1 {
        include!("music.v1alpha1.rs");
    }
}


*/