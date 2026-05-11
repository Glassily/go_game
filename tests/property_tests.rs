use go_game::Property;

#[test]
fn test_property_from_str_and_flags() {
    assert_eq!(Property::from_str("B"), Property::B);
    assert_eq!(Property::from_str("SZ"), Property::SZ);
    assert_eq!(Property::from_str("ZZ"), Property::Other("ZZ".to_string()));

    assert!(Property::B.is_move());
    assert!(!Property::SZ.is_move());
    assert!(Property::AB.is_setup());
    assert!(Property::B.has_coord());
    assert!(Property::AB.has_coord());
}
