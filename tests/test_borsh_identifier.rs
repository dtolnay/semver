#![cfg(feature = "borsh")]

mod util;

use borsh::{BorshDeserialize, BorshSerialize};
use semver::{BuildMetadata, Prerelease};
use util::{build_metadata, prerelease};

fn expected_identifier_encoding(text: &str) -> Vec<u8> {
    let mut buf = Vec::with_capacity(4 + text.len());
    buf.extend_from_slice(&(text.len() as u32).to_le_bytes());
    buf.extend_from_slice(text.as_bytes());
    buf
}

fn serialize_to_vec(value: &impl BorshSerialize) -> Vec<u8> {
    let mut out = Vec::new();
    value.serialize(&mut out).expect("borsh serialize");
    out
}

#[test]
fn prerelease_identifier_serializes_inline_and_heap() {
    for text in ["abcd", "abcdefgh", "stage.alpha.segment9"] {
        let value = prerelease(text);
        let bytes = serialize_to_vec(&value);
        assert_eq!(bytes, expected_identifier_encoding(text));

        let round_trip = Prerelease::try_from_slice(&bytes).expect("deserialize prerelease");
        assert_eq!(round_trip, value);
    }
}

#[test]
fn build_metadata_identifier_serializes_empty_and_long() {
    let empty_bytes = serialize_to_vec(&BuildMetadata::EMPTY);
    assert_eq!(empty_bytes, expected_identifier_encoding(""));
    let decoded_empty =
        BuildMetadata::try_from_slice(&empty_bytes).expect("deserialize empty build metadata");
    assert!(decoded_empty.is_empty());

    let text = "build.20240101.commit.abcdef";
    let metadata = build_metadata(text);
    let bytes = serialize_to_vec(&metadata);
    assert_eq!(bytes, expected_identifier_encoding(text));

    let round_trip =
        BuildMetadata::try_from_slice(&bytes).expect("deserialize populated build metadata");
    assert_eq!(round_trip, metadata);
}
