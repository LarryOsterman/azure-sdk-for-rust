// cspell: words amqp

use crate::amqp_client::value::{/*AmqpList,*/ AmqpOrderedMap, AmqpSymbol, AmqpValue};

#[derive(Debug)]
pub struct Fe2o3Symbol(pub fe2o3_amqp_types::primitives::Symbol);
#[derive(Debug)]
pub struct Fe2o3Value(pub fe2o3_amqp_types::primitives::Value);

impl From<AmqpSymbol> for Fe2o3Symbol {
    fn from(s: AmqpSymbol) -> Fe2o3Symbol {
        Fe2o3Symbol(fe2o3_amqp_types::primitives::Symbol(s.0))
    }
}

impl From<fe2o3_amqp_types::primitives::Symbol> for AmqpSymbol {
    fn from(s: fe2o3_amqp_types::primitives::Symbol) -> AmqpSymbol {
        AmqpSymbol(s.to_string())
    }
}

impl From<fe2o3_amqp_types::primitives::Symbol> for Fe2o3Symbol {
    fn from(s: fe2o3_amqp_types::primitives::Symbol) -> Self {
        Fe2o3Symbol(s)
    }
}

impl From<fe2o3_amqp_types::primitives::Value> for Fe2o3Value {
    fn from(value: fe2o3_amqp_types::primitives::Value) -> Self {
        // Implement from trait.
        Fe2o3Value(value)
    }
}

impl From<AmqpValue> for fe2o3_amqp_types::primitives::Value {
    fn from(value: AmqpValue) -> Self {
        match value {
            AmqpValue::Null => fe2o3_amqp_types::primitives::Value::Null,
            AmqpValue::Boolean(b) => fe2o3_amqp_types::primitives::Value::Bool(b),
            AmqpValue::UByte(_) => todo!(),
            AmqpValue::UShort(_) => todo!(),
            AmqpValue::UInt(_) => todo!(),
            AmqpValue::ULong(_) => todo!(),
            AmqpValue::Byte(_) => todo!(),
            AmqpValue::Short(_) => todo!(),
            AmqpValue::Int(_) => todo!(),
            AmqpValue::Long(_) => todo!(),
            AmqpValue::Float(_) => todo!(),
            AmqpValue::Double(_) => todo!(),
            AmqpValue::Char(_) => todo!(),
            AmqpValue::TimeStamp(_) => todo!(),
            AmqpValue::Uuid(_) => todo!(),
            AmqpValue::Binary(_) => todo!(),
            AmqpValue::String(s) => fe2o3_amqp_types::primitives::Value::String(s),
            AmqpValue::Symbol(s) => fe2o3_amqp_types::primitives::Value::Symbol(s.0.into()),
            AmqpValue::List(_) => todo!(),
            AmqpValue::Map(_) => todo!(),
            AmqpValue::Array(_) => todo!(),
            AmqpValue::Described(_, _) => todo!(),
            AmqpValue::Unknown => todo!(),
        }
    }
}

impl From<fe2o3_amqp_types::primitives::Value> for AmqpValue {
    fn from(value: fe2o3_amqp_types::primitives::Value) -> Self {
        match value {
            fe2o3_amqp_types::primitives::Value::Null => AmqpValue::Null,
            fe2o3_amqp_types::primitives::Value::Bool(b) => AmqpValue::Boolean(b),
            fe2o3_amqp_types::primitives::Value::Ubyte(_) => todo!(),
            fe2o3_amqp_types::primitives::Value::Ushort(_) => todo!(),
            fe2o3_amqp_types::primitives::Value::Uint(_) => todo!(),
            fe2o3_amqp_types::primitives::Value::Ulong(_) => todo!(),
            fe2o3_amqp_types::primitives::Value::Byte(_) => todo!(),
            fe2o3_amqp_types::primitives::Value::Short(_) => todo!(),
            fe2o3_amqp_types::primitives::Value::Int(i) => AmqpValue::Int(i),
            fe2o3_amqp_types::primitives::Value::Long(_) => todo!(),
            fe2o3_amqp_types::primitives::Value::Float(_) => todo!(),
            fe2o3_amqp_types::primitives::Value::Double(_) => todo!(),
            fe2o3_amqp_types::primitives::Value::Char(_) => todo!(),
            fe2o3_amqp_types::primitives::Value::Timestamp(t) => {
                AmqpValue::TimeStamp(t.milliseconds())
            }
            fe2o3_amqp_types::primitives::Value::Uuid(_) => todo!(),
            fe2o3_amqp_types::primitives::Value::Binary(_) => todo!(),
            fe2o3_amqp_types::primitives::Value::String(s) => AmqpValue::String(s),
            fe2o3_amqp_types::primitives::Value::Symbol(s) => AmqpValue::Symbol(s.into()),
            fe2o3_amqp_types::primitives::Value::List(_) => todo!(),
            fe2o3_amqp_types::primitives::Value::Map(_) => todo!(),
            fe2o3_amqp_types::primitives::Value::Array(a) => {
                let mut vec = Vec::new();
                for i in a {
                    vec.push(i.into());
                }
                AmqpValue::Array(vec)
            }
            fe2o3_amqp_types::primitives::Value::Described(_) => todo!(),
            fe2o3_amqp_types::primitives::Value::Decimal128(_) => todo!(),
            fe2o3_amqp_types::primitives::Value::Decimal32(_) => todo!(),
            fe2o3_amqp_types::primitives::Value::Decimal64(_) => todo!(),
        }
    }
}

impl From<AmqpValue> for Fe2o3Value {
    fn from(value: AmqpValue) -> Self {
        match value {
            AmqpValue::Null => Fe2o3Value(fe2o3_amqp_types::primitives::Value::Null),
            AmqpValue::Boolean(b) => Fe2o3Value(fe2o3_amqp_types::primitives::Value::Bool(b)),
            AmqpValue::UByte(_) => todo!(),
            AmqpValue::UShort(_) => todo!(),
            AmqpValue::UInt(_) => todo!(),
            AmqpValue::ULong(_) => todo!(),
            AmqpValue::Byte(_) => todo!(),
            AmqpValue::Short(_) => todo!(),
            AmqpValue::Int(_) => todo!(),
            AmqpValue::Long(_) => todo!(),
            AmqpValue::Float(_) => todo!(),
            AmqpValue::Double(_) => todo!(),
            AmqpValue::Char(_) => todo!(),
            AmqpValue::TimeStamp(_) => todo!(),
            AmqpValue::Uuid(_) => todo!(),
            AmqpValue::Binary(_) => todo!(),
            AmqpValue::String(s) => Fe2o3Value(fe2o3_amqp_types::primitives::Value::String(s)),
            AmqpValue::Symbol(s) => Fe2o3Value(fe2o3_amqp_types::primitives::Value::Symbol(
                fe2o3_amqp_types::primitives::Symbol(s.0),
            )),
            AmqpValue::List(_) => todo!(),
            AmqpValue::Map(_) => todo!(),
            AmqpValue::Array(_) => todo!(),
            AmqpValue::Described(_, _) => todo!(),
            AmqpValue::Unknown => todo!(),
        }
    }
}

impl From<Fe2o3Value> for AmqpValue {
    fn from(value: Fe2o3Value) -> Self {
        AmqpValue::from(value.0)
    }
}

impl<K, V> From<fe2o3_amqp_types::definitions::Fields> for AmqpOrderedMap<K, V>
where
    K: PartialEq + From<fe2o3_amqp_types::primitives::Symbol> + Clone,
    V: From<fe2o3_amqp_types::primitives::Value> + Clone,
{
    fn from(fields: fe2o3_amqp_types::definitions::Fields) -> Self {
        let mut map = AmqpOrderedMap::new();
        for (k, v) in fields {
            map.insert(k.into(), v.into());
        }
        map
    }
}

impl
    From<
        fe2o3_amqp_types::primitives::OrderedMap<
            std::string::String,
            fe2o3_amqp_types::primitives::Value,
        >,
    > for AmqpOrderedMap<std::string::String, AmqpValue>
{
    fn from(
        value: fe2o3_amqp_types::primitives::OrderedMap<
            std::string::String,
            fe2o3_amqp_types::primitives::Value,
        >,
    ) -> Self {
        // Convert the OrderedMap to AmqpOrderedMap
        let mut amqp_ordered_map = AmqpOrderedMap::new();
        for (key, value) in value.into_iter() {
            let fe2o3value: Fe2o3Value = value.into();
            amqp_ordered_map.insert(key, fe2o3value.into());
        }
        amqp_ordered_map
    }
}
