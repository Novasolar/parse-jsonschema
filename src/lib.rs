/*!
JSON Schema types.
*/

use {
    anyhow::{bail, Context},
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

impl TryInto<schemars::schema::ArrayValidation> for ArrayValidation {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<schemars::schema::ArrayValidation, Self::Error> {
        fn sov_try_into<T, U>(sov: SingleOrVec<T>) -> Result<SingleOrVec<U>, anyhow::Error>
        where
            T: TryInto<U, Error = anyhow::Error>,
        {
            match sov {
                SingleOrVec::Single(bt) => {
                    (*bt).try_into().map(|u| SingleOrVec::Single(Box::new(u)))
                }
                SingleOrVec::Vec(v) => {
                    let (us, errs) = v.into_iter().map(|t| t.try_into()).fold(
                        (Vec::new(), Vec::new()),
                        |(mut us, mut errs), next| {
                            match next {
                                Ok(u) => us.push(u),
                                Err(e) => errs.push(e),
                            }
                            (us, errs)
                        },
                    );
                    for e in errs {
                        return Err(e);
                        // TODO: Return all errors
                    }
                    Ok(SingleOrVec::Vec(us))
                }
            }
        }

        Ok(schemars::schema::ArrayValidation {
            items: self.items.map(sov_try_into).transpose()?,
            additional_items: self
                .additional_items
                .map(|s| (*s).try_into())
                .transpose()?
                .map(Box::new),
            max_items: self.max_items,
            min_items: self.min_items,
            unique_items: self.unique_items,
            contains: self
                .contains
                .map(|s| (*s).try_into())
                .transpose()?
                .map(Box::new),
        })
    }
}

impl TryInto<schemars::schema::SubschemaValidation> for SubschemaValidation {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<schemars::schema::SubschemaValidation, Self::Error> {
        let map_vec_schema = |oms: Option<Vec<Schema>>| -> Result<
            Option<Vec<schemars::schema::Schema>>,
            anyhow::Error,
        > {
            oms.map(|v| {
                let (schemas, errs) = v.into_iter().map(|s| s.try_into()).fold(
                    (Vec::new(), Vec::new()),
                    |(mut schemas, mut errs), next: Result<_, anyhow::Error>| {
                        match next {
                            Ok(s) => schemas.push(s),
                            Err(e) => errs.push(e),
                        };
                        (schemas, errs)
                        // TODO: Propogate indexes if preserve_order is active, or find some other way of signifying which subschema the problem was in
                    },
                );
                for e in errs {
                    bail!(e)
                }
                Ok(schemas)
            })
            .transpose()
        };
        Ok(schemars::schema::SubschemaValidation {
            all_of: map_vec_schema(self.all_of).context("in 'allOf'")?,
            any_of: map_vec_schema(self.any_of).context("in 'anyOf'")?,
            one_of: map_vec_schema(self.one_of).context("in 'oneOf'")?,
            not: self.not.map(|s| (*s).try_into()).transpose()?.map(Box::new),
            if_schema: self
                .if_schema
                .map(|s| (*s).try_into())
                .transpose()?
                .map(Box::new),
            then_schema: self
                .then_schema
                .map(|s| (*s).try_into())
                .transpose()?
                .map(Box::new),
            else_schema: self
                .else_schema
                .map(|s| (*s).try_into())
                .transpose()?
                .map(Box::new),
        })
    }
}

impl TryInto<schemars::schema::SchemaObject> for SchemaObject {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<schemars::schema::SchemaObject, Self::Error> {
        if self.required.is_some() {
            bail!("found illegal \"required\" annotation")
        }
        Ok(schemars::schema::SchemaObject {
            metadata: self.metadata,
            instance_type: self.instance_type,
            format: self.format,
            enum_values: self.enum_values,
            const_value: self.const_value,
            subschemas: self
                .subschemas
                .map(|s| (*s).try_into())
                .transpose()
                .context("in subschemas")?
                .map(Box::new),
            number: self.number,
            string: self.string,
            array: self
                .array
                .map(|a| (*a).try_into())
                .transpose()?
                .map(Box::new),
            object: self
                .object
                .map(|a| (*a).try_into())
                .transpose()?
                .map(Box::new),
            reference: self.reference,
            extensions: self.extensions,
        })
    }
}

impl TryInto<schemars::schema::Schema> for Schema {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<schemars::schema::Schema, Self::Error> {
        Ok(match self {
            Schema::Bool(b) => schemars::schema::Schema::Bool(b),
            Schema::Object(o) => schemars::schema::Schema::Object(o.try_into()?),
        })
    }
}

impl TryInto<schemars::schema::ObjectValidation> for ObjectValidation {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<schemars::schema::ObjectValidation, Self::Error> {
        let process_props =
            |(mut props, mut errs): (Map<_, _>, Vec<_>),
             (k, v): (String, Result<_, anyhow::Error>)| match v {
                Ok(v) => {
                    props.insert(k, v);
                    (props, errs)
                }
                Err(e) => {
                    errs.push((k, e));
                    (props, errs)
                }
            };

        let mut required = self.required;

        let (properties, errs) = self
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
            .map(|(k, v)| (k, v.try_into()))
            .fold(Default::default(), process_props);

        for (k, e) in errs {
            return Err(e.context(format!("in field '{k}'")));
            // TODO: Return the full error tree in a more reasonable error
        }

        let (pattern_properties, errs) = self
            .pattern_properties
            .into_iter()
            .map(|(k, v)| (k, v.try_into()))
            .fold(Default::default(), process_props);

        for (k, e) in errs {
            return Err(e.context(format!("in pattern property '{k}'")));
            // TODO: Return the full error tree in a more reasonable error
        }

        Ok(schemars::schema::ObjectValidation {
            max_properties: self.max_properties,
            min_properties: self.min_properties,
            required,
            properties,
            pattern_properties,
            additional_properties: self
                .additional_properties
                .map(|b| (*b).try_into())
                .transpose()?
                .map(Box::new),
            property_names: self
                .property_names
                .map(|b| (*b).try_into())
                .transpose()?
                .map(Box::new),
        })
    }
}
