use super::test_data;
use super::*;

#[test]
fn der_pk_decoding_should_match_test_data() {
    for (raw, der) in &[
        (test_data::ED25519_PK_1_HEX, test_data::ED25519_PK_1_DER_HEX),
        (test_data::ED25519_PK_2_HEX, test_data::ED25519_PK_2_DER_HEX),
        (test_data::ED25519_PK_3_HEX, test_data::ED25519_PK_3_DER_HEX),
    ] {
        // Get fixture:
        let test_public_key = internal_types::PublicKey::try_from(
            &hex::decode(raw).expect("Invalid hex in test data")[..],
        )
        .expect("Invalid public key in test data");
        let test_der: Vec<u8> = hex::decode(der).expect("Invalid hex in test data");
        // Test:
        let decoded =
            internal_types::PublicKey::from_der(&test_der).expect("Conversion from der failed");
        assert_eq!(
            test_public_key, decoded,
            "Parsing yielded a different result for test vector:\n    raw:  {}\n    der:  {}",
            raw, der
        );
    }
}

#[test]
fn der_pk_encoding_should_match_test_data() {
    for (raw, der) in &[
        (test_data::ED25519_PK_1_HEX, test_data::ED25519_PK_1_DER_HEX),
        (test_data::ED25519_PK_2_HEX, test_data::ED25519_PK_2_DER_HEX),
        (test_data::ED25519_PK_3_HEX, test_data::ED25519_PK_3_DER_HEX),
    ] {
        // Get fixture:
        let test_public_key = internal_types::PublicKey::try_from(
            &hex::decode(raw).expect("Invalid hex in test data")[..],
        )
        .expect("Invalid public key in test data");
        let test_der: Vec<u8> = hex::decode(der).expect("Invalid hex in test data");
        // Test:
        let encoded = test_public_key.to_der();
        assert_eq!(
            test_der, encoded,
            "Encoding yielded a different result for test vector:\n    raw:  {}\n    der:  {}",
            raw, der
        );
    }
}

#[test]
fn pem_pk_encoding_should_match_test_data() {
    for (raw, pem) in &[
        (test_data::ED25519_PK_1_HEX, test_data::ED25519_PK_1_PEM),
        (test_data::ED25519_PK_2_HEX, test_data::ED25519_PK_2_PEM),
        (test_data::ED25519_PK_3_HEX, test_data::ED25519_PK_3_PEM),
    ] {
        // Get fixture:
        let test_public_key = internal_types::PublicKey::try_from(
            &hex::decode(raw).expect("Invalid hex in test data")[..],
        )
        .expect("Invalid public key in test data");
        // Test:
        let encoded = test_public_key.to_pem();
        assert_eq!(
            *pem, encoded,
            "Encoding yielded a different result for test vector:\n    raw:  {}\n    der:  {}",
            raw, pem
        );
    }
}

#[test]
fn pem_pk_decoding_should_match_test_data() {
    for (raw, pem) in &[
        (test_data::ED25519_PK_1_HEX, test_data::ED25519_PK_1_PEM),
        (test_data::ED25519_PK_2_HEX, test_data::ED25519_PK_2_PEM),
        (test_data::ED25519_PK_3_HEX, test_data::ED25519_PK_3_PEM),
    ] {
        // Get fixture:
        let test_public_key = internal_types::PublicKey::try_from(
            &hex::decode(raw).expect("Invalid hex in test data")[..],
        )
        .expect("Invalid public key in test data");
        // Test:
        let decoded =
            internal_types::PublicKey::from_pem(&pem).expect("Conversion from der failed");
        assert_eq!(
            test_public_key, decoded,
            "Parsing yielded a different result for test vector:\n    raw:  {}\n    der:  {}",
            raw, pem
        );
    }
}

#[test]
fn should_decode_pem_sk() {
    let (_sk, _pk) = internal_types::SecretKey::from_pem(test_data::ED25519_SK_RFC5958_1_PEM)
        .expect("PEM decoding failed");
}

#[test]
fn should_encode_and_decode_pem_sk() {
    let (sk, pk) = internal_types::SecretKey::from_pem(test_data::ED25519_SK_RFC5958_1_PEM)
        .expect("PEM decoding failed");
    let sk_pem = sk.to_pem(&pk);
    let (sk2, pk2) =
        internal_types::SecretKey::from_pem(&sk_pem).expect("PEM decoding of PEM encoding failed");
    assert_eq!(
        (sk, pk),
        (sk2, pk2),
        "PEM secret key encoding-decoding roundtrip failed."
    );
}

#[test]
fn should_fail_decoding_corrupted_pem_sk() {
    let mut corrupted_pem = String::from(test_data::ED25519_SK_RFC5958_1_PEM);
    // Corrupt the beginning of base64-encoding.
    corrupted_pem.replace_range(30..34, "0815");
    let result = internal_types::SecretKey::from_pem(&corrupted_pem);
    assert!(result.is_err());
}

#[test]
fn should_fail_decoding_non_sk_pem() {
    let result = internal_types::SecretKey::from_pem(test_data::ED25519_PK_1_PEM);
    assert!(result.is_err());
}

// TODO(CRP-695): add more tests
