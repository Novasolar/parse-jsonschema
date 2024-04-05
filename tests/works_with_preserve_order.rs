macro_rules! parse_and_convert {
    ($file_name:expr) => {{
        let file_contents = ::std::fs::read_to_string($file_name).unwrap();
        let parsed: ::parse_jsonschema::RootSchema =
            ::serde_json::from_str(&file_contents).unwrap();
        let schemars: ::schemars::schema::RootSchema = parsed.try_into().unwrap();
        schemars
    }};
}

#[test]
fn smoke() {
    let base_schema = parse_and_convert!(
        "tests/economic/customers.customerNumber.contacts.contactNumber.get.schema.json"
    );

    assert!(base_schema.schema.object.unwrap().required.contains("self"))
}
