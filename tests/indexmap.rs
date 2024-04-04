use parse_jsonschema::Map;

#[test]
fn keeps_ordering() {
    const INPUTS: [(&str, [(&str, usize); 3]); 6] = [
        (
            r#"{
    "a": 3,
    "b": 2,
    "c": 1
}"#,
            [("a", 3), ("b", 2), ("c", 1)],
        ),
        (
            r#"{
    "a": 3,
    "c": 1,
    "b": 2
}"#,
            [("a", 3), ("c", 1), ("b", 2)],
        ),
        (
            r#"{
    "b": 2,
    "c": 1,
    "a": 3
}"#,
            [("b", 2), ("c", 1), ("a", 3)],
        ),
        (
            r#"{
    "b": 2,
    "a": 3,
    "c": 1
}"#,
            [("b", 2), ("a", 3), ("c", 1)],
        ),
        (
            r#"{
    "c": 1,
    "a": 3,
    "b": 2
}"#,
            [("c", 1), ("a", 3), ("b", 2)],
        ),
        (
            r#"{
    "c": 1,
    "b": 2,
    "a": 3
}"#,
            [("c", 1), ("b", 2), ("a", 3)],
        ),
    ];
    for (i, (json, expected)) in INPUTS.into_iter().enumerate() {
        let from_json = serde_json::from_str::<Map<String, usize>>(json)
            .expect(&format!("to be able to parse {i}"));
        let mut res_iter = from_json.into_iter();
        let mut expected_iter = expected.into_iter();
        while let Some((key, val)) = res_iter.next() {
            let (e_key, e_val) = expected_iter.next().unwrap();
            assert_eq!(key, e_key, "failed {i}");
            assert_eq!(val, e_val, "failed {i}");
        }
        assert!(expected_iter.next().is_none())
    }
}
