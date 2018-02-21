use ordermap::OrderMap;
use std::hash::Hash;

use parser::Spanning;
use ast::{InputValue, ToInputValue};

/// Serializable value returned from query and field execution.
///
/// Used by the execution engine and resolvers to build up the response
/// structure. Similar to the `Json` type found in the serialize crate.
///
/// It is also similar to the `InputValue` type, but can not contain enum
/// values or variables. Also, lists and objects do not contain any location
/// information since they are generated by resolving fields and values rather
/// than parsing a source query.
#[derive(Debug, PartialEq)]
#[allow(missing_docs)]
pub enum Value {
    Null,
    Int(i32),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Value>),
    Object(OrderMap<String, Value>),
}

impl Value {
    // CONSTRUCTORS

    /// Construct a null value.
    pub fn null() -> Value {
        Value::Null
    }

    /// Construct an integer value.
    pub fn int(i: i32) -> Value {
        Value::Int(i)
    }

    /// Construct a floating point value.
    pub fn float(f: f64) -> Value {
        Value::Float(f)
    }

    /// Construct a string value.
    pub fn string<T: AsRef<str>>(s: T) -> Value {
        Value::String(s.as_ref().to_owned())
    }

    /// Construct a boolean value.
    pub fn boolean(b: bool) -> Value {
        Value::Boolean(b)
    }

    /// Construct a list value.
    pub fn list(l: Vec<Value>) -> Value {
        Value::List(l)
    }

    /// Construct an object value.
    pub fn object<K>(o: OrderMap<K, Value>) -> Value
    where
        K: Into<String> + Eq + Hash,
    {
        Value::Object(o.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }

    // DISCRIMINATORS

    /// Does this value represent null?
    pub fn is_null(&self) -> bool {
        match *self {
            Value::Null => true,
            _ => false,
        }
    }

    /// View the underlying float value, if present.
    pub fn as_float_value(&self) -> Option<&f64> {
        match *self {
            Value::Float(ref f) => Some(f),
            _ => None,
        }
    }

    /// View the underlying object value, if present.
    pub fn as_object_value(&self) -> Option<&OrderMap<String, Value>> {
        match *self {
            Value::Object(ref o) => Some(o),
            _ => None,
        }
    }

    /// Mutable view into the underlying object value, if present.
    pub fn as_mut_object_value(&mut self) -> Option<&mut OrderMap<String, Value>> {
        match *self {
            Value::Object(ref mut o) => Some(o),
            _ => None,
        }
    }

    /// View the underlying list value, if present.
    pub fn as_list_value(&self) -> Option<&Vec<Value>> {
        match *self {
            Value::List(ref l) => Some(l),
            _ => None,
        }
    }

    /// View the underlying string value, if present.
    pub fn as_string_value(&self) -> Option<&str> {
        match *self {
            Value::String(ref s) => Some(s),
            _ => None,
        }
    }
}

impl ToInputValue for Value {
    fn to_input_value(&self) -> InputValue {
        match *self {
            Value::Null => InputValue::Null,
            Value::Int(i) => InputValue::Int(i),
            Value::Float(f) => InputValue::Float(f),
            Value::String(ref s) => InputValue::String(s.clone()),
            Value::Boolean(b) => InputValue::Boolean(b),
            Value::List(ref l) => InputValue::List(
                l.iter()
                    .map(|x| Spanning::unlocated(x.to_input_value()))
                    .collect(),
            ),
            Value::Object(ref o) => InputValue::Object(
                o.iter()
                    .map(|(k, v)| {
                        (
                            Spanning::unlocated(k.clone()),
                            Spanning::unlocated(v.to_input_value()),
                        )
                    })
                    .collect(),
            ),
        }
    }
}

impl<'a> From<&'a str> for Value {
    fn from(s: &'a str) -> Value {
        Value::string(s)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Value {
        Value::string(s)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Value {
        Value::boolean(b)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Value {
        Value::int(i)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Value {
        Value::float(f)
    }
}

impl<T> From<Option<T>> for Value
where
    Value: From<T>,
{
    fn from(v: Option<T>) -> Value {
        match v {
            Some(v) => Value::from(v),
            None => Value::null(),
        }
    }
}

/// Construct JSON-like values by using JSON syntax
///
/// This macro can be used to create `Value` instances using a JSON syntax.
/// Value objects are used mostly when creating custom errors from fields.
///
/// Here are some examples; the resulting JSON will look just like what you
/// passed in.
/// ```rust
/// #[macro_use] extern crate juniper;
///
/// # fn main() {
/// graphql_value!(1234);
/// graphql_value!("test");
/// graphql_value!([ 1234, "test", true ]);
/// graphql_value!({ "key": "value", "foo": 1234 });
/// # }
/// ```
#[macro_export]
macro_rules! graphql_value {
    ([ $($arg:tt),* $(,)* ]) => {
        $crate::Value::list(vec![
            $( graphql_value!($arg), )*
        ])
    };
    ({ $($key:tt : $val:tt ),* $(,)* }) => {
        $crate::Value::object(vec![
            $( ($key, graphql_value!($val)), )*
        ].into_iter().collect())
    };
    (None) => ($crate::Value::null());
    ($e:expr) => ($crate::Value::from($e))
}

#[cfg(test)]
mod tests {
    use super::Value;

    #[test]
    fn value_macro_string() {
        assert_eq!(graphql_value!("test"), Value::string("test"));
    }

    #[test]
    fn value_macro_int() {
        assert_eq!(graphql_value!(123), Value::int(123));
    }

    #[test]
    fn value_macro_float() {
        assert_eq!(graphql_value!(123.5), Value::float(123.5));
    }

    #[test]
    fn value_macro_boolean() {
        assert_eq!(graphql_value!(false), Value::boolean(false));
    }

    #[test]
    fn value_macro_option() {
        assert_eq!(graphql_value!(Some("test")), Value::string("test"));
        assert_eq!(graphql_value!(None), Value::null());
    }

    #[test]
    fn value_macro_list() {
        assert_eq!(
            graphql_value!([123, "Test", false]),
            Value::list(vec![
                Value::int(123),
                Value::string("Test"),
                Value::boolean(false),
            ])
        );
        assert_eq!(
            graphql_value!([123, [456], 789]),
            Value::list(vec![
                Value::int(123),
                Value::list(vec![Value::int(456)]),
                Value::int(789),
            ])
        );
    }

    #[test]
    fn value_macro_object() {
        assert_eq!(
            graphql_value!({ "key": 123, "next": true }),
            Value::object(
                vec![("key", Value::int(123)), ("next", Value::boolean(true))]
                    .into_iter()
                    .collect()
            )
        );
    }
}
