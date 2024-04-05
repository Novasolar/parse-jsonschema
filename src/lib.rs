/*!
JSON Schema types.
*/

use {
    schemars::{
        schema::{InstanceType, Metadata, NumberValidation, SingleOrVec, StringValidation},
        Map, Set,
    },
    serde::{Deserialize, Serialize},
    serde_json::Value,
};

/// A JSON Schema.
#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Schema {
    /// A trivial boolean JSON Schema.
    ///
    /// The schema `true` matches everything (always passes validation), whereas the schema `false`
    /// matches nothing (always fails validation).
    Bool(bool),
    /// A JSON Schema object.
    Object(SchemaObject),
}

impl Into<schemars::schema::Schema> for Schema {
    fn into(self) -> schemars::schema::Schema {
        match self {
            Schema::Bool(b) => schemars::schema::Schema::Bool(b),
            Schema::Object(o) => schemars::schema::Schema::Object(o.into()),
        }
    }
}

/// The root object of a JSON Schema document.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct RootSchema {
    /// The `$schema` keyword.
    ///
    /// See [JSON Schema 8.1.1. The "$schema" Keyword](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-8.1.1).
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub meta_schema: Option<String>,
    /// The root schema itself.
    #[serde(flatten)]
    pub schema: SchemaObject,
    /// The `definitions` keyword.
    ///
    /// In JSON Schema draft 2019-09 this was replaced by $defs, but in Schemars this is still
    /// serialized as `definitions` for backward-compatibility.
    ///
    /// See [JSON Schema 8.2.5. Schema Re-Use With "$defs"](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-8.2.5),
    /// and [JSON Schema (draft 07) 9. Schema Re-Use With "definitions"](https://tools.ietf.org/html/draft-handrews-json-schema-validation-01#section-9).
    #[serde(alias = "$defs", skip_serializing_if = "Map::is_empty")]
    pub definitions: Map<String, Schema>,
}

impl Into<schemars::schema::RootSchema> for RootSchema {
    fn into(self) -> schemars::schema::RootSchema {
        schemars::schema::RootSchema {
            meta_schema: self.meta_schema,
            schema: self.schema.into(),
            definitions: self
                .definitions
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

/// A JSON Schema object.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SchemaObject {
    /// Properties which annotate the [`SchemaObject`] which typically have no effect when an object is being validated against the schema.
    #[serde(flatten, deserialize_with = "skip_if_default")]
    pub metadata: Option<Box<Metadata>>,
    /// The `type` keyword.
    ///
    /// See [JSON Schema Validation 6.1.1. "type"](https://tools.ietf.org/html/draft-handrews-json-schema-validation-02#section-6.1.1)
    /// and [JSON Schema 4.2.1. Instance Data Model](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-4.2.1).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub instance_type: Option<SingleOrVec<InstanceType>>,
    /// The `format` keyword.
    ///
    /// See [JSON Schema Validation 7. A Vocabulary for Semantic Content With "format"](https://tools.ietf.org/html/draft-handrews-json-schema-validation-02#section-7).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// The `enum` keyword.
    ///
    /// See [JSON Schema Validation 6.1.2. "enum"](https://tools.ietf.org/html/draft-handrews-json-schema-validation-02#section-6.1.2)
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<Value>>,
    /// The `const` keyword.
    ///
    /// See [JSON Schema Validation 6.1.3. "const"](https://tools.ietf.org/html/draft-handrews-json-schema-validation-02#section-6.1.3)
    #[serde(
        rename = "const",
        skip_serializing_if = "Option::is_none",
        deserialize_with = "allow_null"
    )]
    pub const_value: Option<Value>,
    /// Properties of the [`SchemaObject`] which define validation assertions in terms of other schemas.
    #[serde(flatten, deserialize_with = "skip_if_default")]
    pub subschemas: Option<Box<SubschemaValidation>>,
    /// Properties of the [`SchemaObject`] which define validation assertions for numbers.
    #[serde(flatten, deserialize_with = "skip_if_default")]
    pub number: Option<Box<NumberValidation>>,
    /// Properties of the [`SchemaObject`] which define validation assertions for strings.
    #[serde(flatten, deserialize_with = "skip_if_default")]
    pub string: Option<Box<StringValidation>>,
    /// Properties of the [`SchemaObject`] which define validation assertions for arrays.
    #[serde(flatten, deserialize_with = "skip_if_default")]
    pub array: Option<Box<ArrayValidation>>,
    /// Properties of the [`SchemaObject`] which define validation assertions for objects.
    #[serde(flatten, deserialize_with = "skip_if_default")]
    pub object: Option<Box<ObjectValidation>>,
    /// The `$ref` keyword.
    ///
    /// See [JSON Schema 8.2.4.1. Direct References with "$ref"](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-8.2.4.1).
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    /// The old way of denoting required fields
    ///
    /// See [JSON Schema (03) 5.7. required](https://datatracker.ietf.org/doc/html/draft-zyp-json-schema-03#anchor12)
    /// Arbitrary extra properties which are not part of the JSON Schema specification, or which `schemars` does not support.
    pub required: Option<bool>,
    #[serde(flatten)]
    pub extensions: Map<String, Value>,
}

impl Into<schemars::schema::SchemaObject> for SchemaObject {
    fn into(self) -> schemars::schema::SchemaObject {
        schemars::schema::SchemaObject {
            metadata: self.metadata,
            instance_type: self.instance_type,
            format: self.format,
            enum_values: self.enum_values,
            const_value: self.const_value,
            subschemas: self.subschemas.map(|s| Box::new((*s).into())),
            number: self.number,
            string: self.string,
            array: self.array.map(|a| Box::new((*a).into())),
            object: self.object.map(|a| Box::new((*a).into())),
            reference: self.reference,
            extensions: self.extensions,
        }
    }
}

// Deserializing "null" to `Option<Value>` directly results in `None`,
// this function instead makes it deserialize to `Some(Value::Null)`.
fn allow_null<'de, D>(de: D) -> Result<Option<Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Value::deserialize(de).map(Option::Some)
}

fn skip_if_default<'de, D, T>(deserializer: D) -> Result<Option<Box<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de> + Default + PartialEq,
{
    let value = T::deserialize(deserializer)?;
    if value == T::default() {
        Ok(None)
    } else {
        Ok(Some(Box::new(value)))
    }
}

/// Properties of a [`SchemaObject`] which define validation assertions in terms of other schemas.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SubschemaValidation {
    /// The `allOf` keyword.
    ///
    /// See [JSON Schema 9.2.1.1. "allOf"](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-9.2.1.1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<Schema>>,
    /// The `anyOf` keyword.
    ///
    /// See [JSON Schema 9.2.1.2. "anyOf"](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-9.2.1.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<Schema>>,
    /// The `oneOf` keyword.
    ///
    /// See [JSON Schema 9.2.1.3. "oneOf"](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-9.2.1.3).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<Schema>>,
    /// The `not` keyword.
    ///
    /// See [JSON Schema 9.2.1.4. "not"](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-9.2.1.4).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<Box<Schema>>,
    /// The `if` keyword.
    ///
    /// See [JSON Schema 9.2.2.1. "if"](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-9.2.2.1).
    #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
    pub if_schema: Option<Box<Schema>>,
    /// The `then` keyword.
    ///
    /// See [JSON Schema 9.2.2.2. "then"](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-9.2.2.2).
    #[serde(rename = "then", skip_serializing_if = "Option::is_none")]
    pub then_schema: Option<Box<Schema>>,
    /// The `else` keyword.
    ///
    /// See [JSON Schema 9.2.2.3. "else"](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-9.2.2.3).
    #[serde(rename = "else", skip_serializing_if = "Option::is_none")]
    pub else_schema: Option<Box<Schema>>,
}

impl Into<schemars::schema::SubschemaValidation> for SubschemaValidation {
    fn into(self) -> schemars::schema::SubschemaValidation {
        schemars::schema::SubschemaValidation {
            all_of: self
                .all_of
                .map(|v| v.into_iter().map(|e| e.into()).collect()),
            any_of: self
                .any_of
                .map(|v| v.into_iter().map(|e| e.into()).collect()),
            one_of: self
                .one_of
                .map(|v| v.into_iter().map(|e| e.into()).collect()),
            not: self.not.map(|s| Box::new((*s).into())),
            if_schema: self.if_schema.map(|s| Box::new((*s).into())),
            then_schema: self.then_schema.map(|s| Box::new((*s).into())),
            else_schema: self.else_schema.map(|s| Box::new((*s).into())),
        }
    }
}

/// Properties of a [`SchemaObject`] which define validation assertions for arrays.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ArrayValidation {
    /// The `items` keyword.
    ///
    /// See [JSON Schema 9.3.1.1. "items"](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-9.3.1.1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<SingleOrVec<Schema>>,
    /// The `additionalItems` keyword.
    ///
    /// See [JSON Schema 9.3.1.2. "additionalItems"](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-9.3.1.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_items: Option<Box<Schema>>,
    /// The `maxItems` keyword.
    ///
    /// See [JSON Schema Validation 6.4.1. "maxItems"](https://tools.ietf.org/html/draft-handrews-json-schema-validation-02#section-6.4.1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u32>,
    /// The `minItems` keyword.
    ///
    /// See [JSON Schema Validation 6.4.2. "minItems"](https://tools.ietf.org/html/draft-handrews-json-schema-validation-02#section-6.4.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<u32>,
    /// The `uniqueItems` keyword.
    ///
    /// See [JSON Schema Validation 6.4.3. "uniqueItems"](https://tools.ietf.org/html/draft-handrews-json-schema-validation-02#section-6.4.3).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,
    /// The `contains` keyword.
    ///
    /// See [JSON Schema 9.3.1.4. "contains"](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-9.3.1.4).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contains: Option<Box<Schema>>,
}

impl Into<schemars::schema::ArrayValidation> for ArrayValidation {
    fn into(self) -> schemars::schema::ArrayValidation {
        fn sov_into<T, U>(sov: SingleOrVec<T>) -> SingleOrVec<U>
        where
            T: Into<U>,
        {
            match sov {
                SingleOrVec::Single(bt) => SingleOrVec::Single(Box::new((*bt).into())),
                SingleOrVec::Vec(v) => SingleOrVec::Vec(v.into_iter().map(|v| v.into()).collect()),
            }
        }

        schemars::schema::ArrayValidation {
            items: self.items.map(sov_into),
            additional_items: self.additional_items.map(|s| Box::new((*s).into())),
            max_items: self.max_items,
            min_items: self.min_items,
            unique_items: self.unique_items,
            contains: self.contains.map(|s| Box::new((*s).into())),
        }
    }
}

/// Properties of a [`SchemaObject`] which define validation assertions for objects.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ObjectValidation {
    /// The `maxProperties` keyword.
    ///
    /// See [JSON Schema Validation 6.5.1. "maxProperties"](https://tools.ietf.org/html/draft-handrews-json-schema-validation-02#section-6.5.1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<u32>,
    /// The `minProperties` keyword.
    ///
    /// See [JSON Schema Validation 6.5.2. "minProperties"](https://tools.ietf.org/html/draft-handrews-json-schema-validation-02#section-6.5.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<u32>,
    /// The `required` keyword.
    ///
    /// See [JSON Schema Validation 6.5.3. "required"](https://tools.ietf.org/html/draft-handrews-json-schema-validation-02#section-6.5.3).
    #[serde(skip_serializing_if = "Set::is_empty")]
    pub required: Set<String>,
    /// The `properties` keyword.
    ///
    /// See [JSON Schema 9.3.2.1. "properties"](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-9.3.2.1).
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub properties: Map<String, Schema>,
    /// The `patternProperties` keyword.
    ///
    /// See [JSON Schema 9.3.2.2. "patternProperties"](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-9.3.2.2).
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub pattern_properties: Map<String, Schema>,
    /// The `additionalProperties` keyword.
    ///
    /// See [JSON Schema 9.3.2.3. "additionalProperties"](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-9.3.2.3).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<Box<Schema>>,
    /// The `propertyNames` keyword.
    ///
    /// See [JSON Schema 9.3.2.5. "propertyNames"](https://tools.ietf.org/html/draft-handrews-json-schema-02#section-9.3.2.5).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_names: Option<Box<Schema>>,
}

impl Into<schemars::schema::ObjectValidation> for ObjectValidation {
    fn into(self) -> schemars::schema::ObjectValidation {
        let mut required = self.required;

        let properties = self
            .properties
            .into_iter()
            .map(|(k, v)| {
                let mut o = match v {
                    Schema::Bool(b) => return (k, Schema::Bool(b)),
                    Schema::Object(o) => o,
                };
                let req = o.required.take();
                if let Some(true) = req {
                    required.insert(k.clone());
                };
                (k, Schema::Object(o))
            })
            .map(|(k, v)| (k, v.into()))
            .collect();

        let pattern_properties = self
            .pattern_properties
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect();

        schemars::schema::ObjectValidation {
            max_properties: self.max_properties,
            min_properties: self.min_properties,
            required,
            properties,
            pattern_properties,
            additional_properties: self.additional_properties.map(|b| Box::new((*b).into())),
            property_names: self.property_names.map(|b| Box::new((*b).into())),
        }
    }
}
