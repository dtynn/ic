use crate::bootstrap_registry_data_provider::INITIAL_REGISTRY_VERSION;
use crate::pb::proto_registry::v1::{ProtoRegistry, ProtoRegistryRecord};
use bytes::{Buf, BufMut};
use ic_interfaces::registry::{RegistryDataProvider, RegistryTransportRecord, RegistryValue};
use ic_registry_transport::pb::v1::RegistryMutation;
use ic_types::{registry::RegistryDataProviderError, RegistryVersion};
use ic_utils::fs::write_atomically;
use std::{
    io::Write,
    path::Path,
    sync::{Arc, RwLock},
};
use thiserror::Error;

#[derive(Clone)]
pub struct ProtoRegistryDataProvider {
    records: Arc<RwLock<Vec<ProtoRegistryRecord>>>,
}

#[derive(Error, Clone, Debug)]
pub enum ProtoRegistryDataProviderError {
    #[error("key {key} already exists at version {version}")]
    KeyAlreadyExists {
        key: String,
        version: RegistryVersion,
    },
}

/// A simple RegistryDataProvider that can be used for tests and loading/storing
/// from/to a file.
impl ProtoRegistryDataProvider {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add<T>(
        &self,
        key: &str,
        version: RegistryVersion,
        value: Option<T>,
    ) -> Result<(), ProtoRegistryDataProviderError>
    where
        T: RegistryValue,
    {
        assert!(version.get() > 0);
        let mut records = self.records.write().unwrap();

        let search_key = &(&version.get(), key);
        match records.binary_search_by_key(search_key, |r| (&r.version, &r.key)) {
            Ok(_) => Err(ProtoRegistryDataProviderError::KeyAlreadyExists {
                key: key.to_string(),
                version,
            }),
            Err(idx) => {
                let mut record = ProtoRegistryRecord::default();
                record.key = key.to_string();
                record.version = version.get();
                record.value = value.map(|v| {
                    let mut buf: Vec<u8> = vec![];
                    v.encode(&mut buf)
                        .expect("can't fail, encoding is infallible");
                    buf
                });
                records.insert(idx, record);
                Ok(())
            }
        }
    }

    /// Writes mutations to the initial Registry
    pub fn add_mutations(
        &self,
        mutations: Vec<RegistryMutation>,
    ) -> Result<(), ProtoRegistryDataProviderError> {
        let mut records = self.records.write().unwrap();
        let version = INITIAL_REGISTRY_VERSION;

        for mutation in mutations {
            let key = std::str::from_utf8(&mutation.key)
                .expect("Expected registry key to be utf8-encoded");

            let search_key = &(&version.get(), key);

            match records.binary_search_by_key(search_key, |r| (&r.version, &r.key)) {
                Ok(_) => {
                    return Err(ProtoRegistryDataProviderError::KeyAlreadyExists {
                        key: key.to_string(),
                        version,
                    })
                }
                Err(idx) => {
                    let mut record = ProtoRegistryRecord::default();
                    record.key = key.to_string();
                    record.version = version.get();
                    record.value = Some(mutation.value);
                    records.insert(idx, record);
                }
            }
        }

        Ok(())
    }

    pub fn decode<B: Buf>(buf: B) -> Self {
        let registry = ProtoRegistry::decode(buf).expect("Could not decode protobuf registry.");

        Self {
            records: Arc::new(RwLock::new(registry.records)),
        }
    }

    pub fn encode<B: BufMut>(&self, buf: &mut B) {
        let mut protobuf_registry = ProtoRegistry::default();

        protobuf_registry.records = self.records.read().unwrap().clone();
        protobuf_registry
            .encode(buf)
            .expect("Could not encode protobuf registry.");
    }

    pub fn load_from_file<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let buf = std::fs::read(path.as_ref()).unwrap_or_else(|e| {
            panic!(format!(
                "Could not read protobuf registry file at {:?}: {}",
                path.as_ref().to_str(),
                e
            ))
        });
        Self::decode(buf.as_ref())
    }

    /// Write the state of this data provider to a file at `path`.
    pub fn write_to_file<P>(&self, path: P)
    where
        P: AsRef<Path>,
    {
        write_atomically(path, |f| {
            let mut buf: Vec<u8> = vec![];
            self.encode(&mut buf);
            f.write_all(buf.as_slice())
        })
        .expect("Could not write to path.");
    }
}

impl Default for ProtoRegistryDataProvider {
    fn default() -> Self {
        Self {
            records: Arc::new(RwLock::new(vec![])),
        }
    }
}

impl RegistryDataProvider for ProtoRegistryDataProvider {
    /// This function only accesses internal state which is assumed to be valid,
    /// so it may neither panic nor return an error.
    fn get_updates_since(
        &self,
        version: RegistryVersion,
    ) -> Result<(Vec<RegistryTransportRecord>, RegistryVersion), RegistryDataProviderError> {
        let records = self.records.read().unwrap();
        let max_version = records
            .iter()
            .max_by_key(|r| r.version)
            .map(|r| r.version)
            .unwrap_or(0);

        let records = records
            .iter()
            .filter(|r| r.version > version.get())
            .map(|r| RegistryTransportRecord {
                key: r.key.clone(),
                version: RegistryVersion::new(r.version),
                value: r.value.to_owned(),
            })
            .collect();

        Ok((records, RegistryVersion::new(max_version)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb::test_protos::v1::TestProto;
    use ic_interfaces::registry::ZERO_REGISTRY_VERSION;

    #[test]
    fn round_trip() {
        let registry = ProtoRegistryDataProvider::new();

        let test_version = RegistryVersion::new(1);

        let mut test_record = TestProto::default();
        test_record.test_value = 1;

        let mut test_record2 = TestProto::default();
        test_record2.test_value = 2;

        let mut bytes1: Vec<u8> = Vec::new();
        let mut bytes2: Vec<u8> = Vec::new();

        test_record.encode(&mut bytes1).expect("encoding failed");
        test_record2.encode(&mut bytes2).expect("encoding failed");

        registry
            .add("A", test_version, Some(test_record))
            .expect("Could not add record to data provider");
        registry
            .add("B", test_version, Some(test_record2))
            .expect("Could not add record to data provider");
        registry
            .add::<TestProto>("C", test_version, None)
            .expect("Could not add record to data provider");

        let mut buf: Vec<u8> = vec![];
        registry.encode(&mut buf);

        let registry = ProtoRegistryDataProvider::decode(buf.as_ref());
        let (records, version) = registry.get_updates_since(ZERO_REGISTRY_VERSION).unwrap();

        assert_eq!(version, test_version);
        let mut records = records
            .iter()
            .map(|r| (r.key.clone(), r.value.to_owned()))
            .collect::<Vec<(String, Option<Vec<u8>>)>>();
        records.sort();

        assert_eq!(
            records,
            vec![
                ("A".to_string(), Some(bytes1)),
                ("B".to_string(), Some(bytes2)),
                ("C".to_string(), None)
            ]
        );
    }
}
