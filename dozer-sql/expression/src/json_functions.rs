use crate::arg_utils::validate_num_arguments;
use crate::error::Error;
use crate::execution::Expression;

use dozer_types::json_types::JsonValue;
use dozer_types::types::Record;
use dozer_types::types::{Field, Schema};
use jsonpath::{JsonPathFinder, JsonPathInst};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum JsonFunctionType {
    JsonValue,
    JsonQuery,
}

impl Display for JsonFunctionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonFunctionType::JsonValue => f.write_str("JSON_VALUE".to_string().as_str()),
            JsonFunctionType::JsonQuery => f.write_str("JSON_QUERY".to_string().as_str()),
        }
    }
}

impl JsonFunctionType {
    pub(crate) fn new(name: &str) -> Option<JsonFunctionType> {
        match name {
            "json_value" => Some(JsonFunctionType::JsonValue),
            "json_query" => Some(JsonFunctionType::JsonQuery),
            _ => None,
        }
    }

    pub(crate) fn evaluate(
        &self,
        schema: &Schema,
        args: &Vec<Expression>,
        record: &Record,
    ) -> Result<Field, Error> {
        match self {
            JsonFunctionType::JsonValue => self.evaluate_json_value(schema, args, record),
            JsonFunctionType::JsonQuery => self.evaluate_json_query(schema, args, record),
        }
    }

    pub(crate) fn evaluate_json_value(
        &self,
        schema: &Schema,
        args: &Vec<Expression>,
        record: &Record,
    ) -> Result<Field, Error> {
        validate_num_arguments(2..3, args.len(), self)?;
        let json_input = args[0].evaluate(record, schema)?;
        let path = args[1].evaluate(record, schema)?.to_string();

        if let Ok(json_value) = self.evaluate_json(json_input, path) {
            match json_value {
                JsonValue::Object(_) => Ok(Field::Json(JsonValue::Null)),
                JsonValue::Array(_) => Ok(Field::Json(JsonValue::Null)),
                JsonValue::String(val) => Ok(Field::Json(JsonValue::String(val))),
                JsonValue::Bool(val) => Ok(Field::Json(JsonValue::Bool(val))),
                JsonValue::Number(val) => Ok(Field::Json(JsonValue::Number(val))),
                JsonValue::Null => Ok(Field::Json(JsonValue::Null)),
            }
        } else {
            Ok(Field::Null)
        }
    }

    pub(crate) fn evaluate_json_query(
        &self,
        schema: &Schema,
        args: &Vec<Expression>,
        record: &Record,
    ) -> Result<Field, Error> {
        validate_num_arguments(1..3, args.len(), self)?;
        if args.len() == 1 {
            Ok(Field::Json(self.evaluate_json(
                args[0].evaluate(record, schema)?,
                String::from("$"),
            )?))
        } else {
            let json_input = args[0].evaluate(record, schema)?;
            let path = args[1].evaluate(record, schema)?.to_string();

            if let Ok(json_value) = self.evaluate_json(json_input, path) {
                match json_value {
                    JsonValue::Object(val) => Ok(Field::Json(JsonValue::Object(val))),
                    JsonValue::Array(val) => Ok(Field::Json(JsonValue::Array(val))),
                    JsonValue::String(_) => Ok(Field::Json(JsonValue::Null)),
                    JsonValue::Bool(_) => Ok(Field::Json(JsonValue::Null)),
                    JsonValue::Number(_) => Ok(Field::Json(JsonValue::Null)),
                    JsonValue::Null => Ok(Field::Json(JsonValue::Null)),
                }
            } else {
                Ok(Field::Null)
            }
        }
    }

    pub(crate) fn evaluate_json(
        &self,
        json_input: Field,
        path: String,
    ) -> Result<JsonValue, Error> {
        let json_val = match json_input.to_json() {
            Some(json) => json,
            None => JsonValue::Null,
        };

        let finder = JsonPathFinder::new(
            Box::from(json_val),
            Box::from(JsonPathInst::from_str(path.as_str()).map_err(Error::InvalidJsonPath)?),
        );

        match finder.find() {
            JsonValue::Null => Ok(JsonValue::Null),
            JsonValue::Array(mut a) => {
                if a.is_empty() {
                    Ok(JsonValue::Array(vec![]))
                } else if a.len() == 1 {
                    Ok(a.remove(0))
                } else {
                    let mut array_val = vec![];
                    for item in a {
                        array_val.push(item);
                    }
                    Ok(JsonValue::Array(array_val))
                }
            }
            other => Ok(other),
        }
    }
}
