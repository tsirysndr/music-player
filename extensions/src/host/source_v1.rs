//! Nonblocking legacy dispatch for the shared source scheduler. The public
//! optional legacy API is retained for existing callers. Neither path may
//! reinterpret a loaded v2 instance as v1.
use super::{Extension, SourceInstanceBusy, SourceProtocol, TryLockError, SOURCE_V1_EXPORTS};
use anyhow::{anyhow, Error};
use serde::{de::DeserializeOwned, Serialize};

impl Extension {
    pub(crate) fn call_source_v1<I: Serialize, O: DeserializeOwned>(
        &self,
        function: &str,
        input: &I,
    ) -> Result<Option<O>, Error> {
        if self.source_protocol != Some(SourceProtocol::V1)
            || !SOURCE_V1_EXPORTS.contains(&function)
        {
            return Err(anyhow!("source v1 export was not selected at load"));
        }
        let body =
            serde_json::to_vec(input).map_err(|_| anyhow!("could not encode source v1 request"))?;
        let mut plugin = match self.plugin.try_lock() {
            Ok(plugin) => plugin,
            Err(TryLockError::WouldBlock) => return Err(SourceInstanceBusy.into()),
            Err(TryLockError::Poisoned(_)) => {
                return Err(anyhow!("source v1 instance lock is poisoned"));
            }
        };
        // A declaration alone is insufficient. Lists remain optional because
        // a source need not expose albums, artists AND playlists.
        for required in ["source_info", "get_stream_url"] {
            if !plugin.function_exists(required) {
                return Err(anyhow!("source v1 is missing required export {required}"));
            }
        }
        if !plugin.function_exists(function) {
            return Ok(None);
        }
        let raw = plugin
            .call::<&[u8], &[u8]>(function, &body)
            .map_err(|_| anyhow!("source v1 export failed"))?;
        if raw.is_empty() {
            return Err(anyhow!("source v1 export returned an empty response"));
        }
        serde_json::from_slice(raw)
            .map(Some)
            .map_err(|_| anyhow!("source v1 export returned invalid JSON"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::HostContext;
    use serde_json::{json, Value};
    use std::{
        collections::BTreeMap,
        sync::{mpsc, Arc},
        time::Duration,
    };

    fn fixture() -> (tempfile::TempDir, Extension) {
        let dir = tempfile::tempdir().unwrap();
        // No output is deliberate: a called-but-empty export is an error,
        // unlike an optional list that does not exist.
        std::fs::write(
            dir.path().join("plugin.wat"),
            r#"(module
            (func (export "source_info") (result i32) (i32.const 0))
            (func (export "get_stream_url") (result i32) (i32.const 0)))"#,
        )
        .unwrap();
        std::fs::write(dir.path().join("plugin.json"), json!({
            "id":"legacy.host", "name":"Legacy host", "capabilities":["source"], "entry":"plugin.wat"
        }).to_string()).unwrap();
        let extension =
            Extension::load(dir.path(), &BTreeMap::new(), HostContext::default()).unwrap();
        (dir, extension)
    }

    #[test]
    fn legacy_busy_admission_does_not_wait_for_the_instance_lock() {
        let (_dir, extension) = fixture();
        let extension = Arc::new(extension);
        let held = extension.plugin.lock().unwrap();
        let caller = extension.clone();
        let (sent, received) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            let _ = sent.send(caller.call_source_v1::<_, Value>("source_info", &()));
        });
        let observed = received.recv_timeout(Duration::from_secs(2));
        drop(held);
        worker.join().unwrap();
        let error = observed
            .expect("legacy admission waited on busy instance")
            .unwrap_err();
        assert!(error.downcast_ref::<SourceInstanceBusy>().is_some());
    }

    #[test]
    fn legacy_optional_list_and_empty_export_are_distinct() {
        let (_dir, mut extension) = fixture();
        assert!(extension
            .call_source_v1::<_, Value>("list_albums", &())
            .unwrap()
            .is_none());
        assert!(extension
            .call_source_v1::<_, Value>("source_info", &())
            .is_err());
        assert!(extension
            .call_source_v1::<_, Value>("get_stream_url", &())
            .is_err());
        assert!(extension
            .call_source_v1::<_, Value>("resolve_audio_v2", &())
            .is_err());
        // Runtime routing uses the version fixed at load, never mutable metadata.
        extension.manifest.source_api_version = Some(2);
        assert!(extension
            .call_source_v1::<_, Value>("list_albums", &())
            .unwrap()
            .is_none());
    }
}
