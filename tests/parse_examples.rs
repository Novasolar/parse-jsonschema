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
#[allow(non_snake_case)]
fn customers_customerNumber_contacts_contactNumber_get_schema_json() {
    let base_schema = parse_and_convert!(
        "tests/economic/customers.customerNumber.contacts.contactNumber.get.schema.json"
    );

    assert!(base_schema.schema.object.unwrap().required.contains("self"))
}

#[test]
fn customers_get_schema_json() {
    let schema = parse_and_convert!("tests/economic/customers.get.schema.json");
    let object = schema.schema.object.unwrap();
    dbg!(&object);
    let collection = object
        .properties
        .get("collection")
        .unwrap()
        .clone()
        .into_object()
        .array
        .unwrap();
    let items = match collection.items.unwrap() {
        schemars::schema::SingleOrVec::Single(items) => items.into_object().object.unwrap(),
        schemars::schema::SingleOrVec::Vec(_) => panic!(),
    };
    assert!(items.required.contains("currency"));
    assert!(items.required.contains("name"));
    assert!(items.required.contains("customerGroup"));
    assert!(items.required.contains("paymentTerms"));
    assert!(items.required.contains("vatZone"));
    assert!(items.required.contains("self"));
    let _meta_data = object.properties.get("metaData").unwrap();
    let _pagination = object.properties.get("pagination").unwrap();
    let _self_ = object.properties.get("self").unwrap();
    assert!(object.required.contains("self"));
}
