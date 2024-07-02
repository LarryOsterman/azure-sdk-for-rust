// cspell: words amqp

#[derive(Debug, PartialEq, Clone)]
pub struct AmqpSymbol(pub String);

#[derive(Debug, PartialEq, Clone)]
pub struct AmqpList(pub Vec<AmqpValue>);

#[derive(Debug, PartialEq, Clone)]
pub struct AmqpTimestamp(pub std::time::SystemTime);

#[derive(Debug, PartialEq, Clone)]
pub struct AmqpOrderedMap<K, V>
where
    K: PartialEq,
{
    inner: Vec<(K, V)>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AmqpDescriptor {
    Code(u64),
    Name(AmqpSymbol),
}

#[derive(Debug, PartialEq, Clone)]
pub struct AmqpDescribed {
    pub descriptor: AmqpDescriptor,
    pub value: AmqpValue,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AmqpValue {
    Null,
    Boolean(bool),
    UByte(u8),
    UShort(u16),
    UInt(u32),
    ULong(u64),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Char(char),
    TimeStamp(AmqpTimestamp),
    Uuid(uuid::Uuid),
    Binary(Vec<u8>),
    String(String),
    Symbol(AmqpSymbol),
    List(AmqpList),
    Map(AmqpOrderedMap<AmqpValue, AmqpValue>),
    Array(Vec<AmqpValue>),
    Described(Box<AmqpDescribed>),
    Unknown,
}

impl AmqpValue {}

impl<K, V> AmqpOrderedMap<K, V>
where
    K: PartialEq + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.inner.push((key, value));
    }

    pub fn get(&self, key: impl Into<K> + Clone) -> Option<&V> {
        self.inner.iter().find_map(|(k, v)| {
            if *k == key.clone().into() {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let index = self.inner.iter().position(|(k, _)| k == key)?;
        Some(self.inner.remove(index).1)
    }

    pub fn contains_key(&self, key: impl Into<K> + Clone) -> bool {
        self.inner.iter().any(|(k, _)| *k == key.clone().into())
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, V)> + '_ {
        self.inner.iter().map(|(k, v)| (k.clone(), v.clone()))
    }
}

impl<K, V> IntoIterator for AmqpOrderedMap<K, V>
where
    K: PartialEq,
{
    type Item = (K, V);
    type IntoIter = <Vec<(K, V)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

macro_rules! conversions_for_amqp_types {
    ($($t:ty, $field:ident),*) => {
        $(
            impl From<$t> for AmqpValue {
                fn from(v: $t) -> Self {
                    AmqpValue::$field(v)
                }
            }

            impl From<AmqpValue> for $t {
                fn from(v: AmqpValue) -> Self {
                    match v {
                        AmqpValue::$field(v) => v,
                        _ => panic!("Expected a {}", stringify!($t)),
                    }
                }
            }

            impl PartialEq<$t> for AmqpValue {
                fn eq(&self, other: &$t) -> bool {
                    match self {
                        AmqpValue::$field(v) => v == other,
                        _ => false,
                    }
                }
            }
            impl PartialEq<AmqpValue> for $t {
                fn eq(&self, other: &AmqpValue) -> bool {
                    match other {
                        AmqpValue::$field(v) => self == v,
                        _ => false,
                    }
                }
            }
        )*
    }
}

conversions_for_amqp_types!(
    bool,
    Boolean,
    u8,
    UByte,
    u16,
    UShort,
    u32,
    UInt,
    u64,
    ULong,
    i8,
    Byte,
    i16,
    Short,
    i32,
    Int,
    i64,
    Long,
    f32,
    Float,
    f64,
    Double,
    char,
    Char,
    uuid::Uuid,
    Uuid,
    Vec<u8>,
    Binary,
    std::string::String,
    String,
    AmqpSymbol,Symbol,
    AmqpList,List,
    Vec<AmqpValue>,Array,
    AmqpOrderedMap<AmqpValue, AmqpValue>, Map
);
conversions_for_amqp_types!(AmqpTimestamp, TimeStamp);

impl From<()> for AmqpValue {
    fn from(_: ()) -> Self {
        AmqpValue::Null
    }
}

impl From<AmqpValue> for () {
    fn from(v: AmqpValue) -> Self {
        match v {
            AmqpValue::Null => (),
            _ => panic!("Expected a null value"),
        }
    }
}

impl PartialEq<()> for AmqpValue {
    fn eq(&self, _: &()) -> bool {
        matches!(self, AmqpValue::Null)
    }
}

impl PartialEq<AmqpValue> for () {
    fn eq(&self, other: &AmqpValue) -> bool {
        other == self
    }
}

impl From<Box<AmqpDescribed>> for AmqpDescribed {
    fn from(b: Box<AmqpDescribed>) -> Self {
        *b
    }
}

impl From<AmqpValue> for AmqpDescribed {
    fn from(v: AmqpValue) -> Self {
        match v {
            AmqpValue::Described(d) => *d,
            _ => panic!("Expected a described value"),
        }
    }
}

impl From<&str> for AmqpValue {
    fn from(b: &str) -> Self {
        AmqpValue::String(b.to_string())
    }
}

impl From<Box<AmqpValue>> for AmqpValue {
    fn from(b: Box<AmqpValue>) -> Self {
        *b
    }
}

impl From<String> for AmqpSymbol {
    fn from(s: String) -> Self {
        AmqpSymbol(s.into())
    }
}
impl From<AmqpSymbol> for String {
    fn from(s: AmqpSymbol) -> Self {
        s.0
    }
}

impl From<&str> for AmqpSymbol {
    fn from(s: &str) -> Self {
        AmqpSymbol(s.to_string())
    }
}

impl<K, V> From<Vec<(K, V)>> for AmqpOrderedMap<K, V>
where
    K: PartialEq,
{
    fn from(v: Vec<(K, V)>) -> Self {
        AmqpOrderedMap { inner: v }
    }
}

impl<K, V> FromIterator<(K, V)> for AmqpOrderedMap<K, V>
where
    K: PartialEq,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        AmqpOrderedMap {
            inner: iter.into_iter().collect(),
        }
    }
}

impl<V> FromIterator<V> for AmqpList
where
    V: Into<AmqpValue>,
{
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        AmqpList {
            0: iter.into_iter().map(|v| v.into()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_value_create_specific() {
        let uuid = uuid::Uuid::new_v4();
        let timestamp = std::time::SystemTime::now();
        let v1 = AmqpValue::Boolean(true);
        let v2 = AmqpValue::UByte(1);
        let v3 = AmqpValue::UShort(2);
        let v4 = AmqpValue::UInt(3);
        let v5 = AmqpValue::ULong(4);
        let v6 = AmqpValue::Byte(5);
        let v7 = AmqpValue::Short(6);
        let v8 = AmqpValue::Int(7);
        let v9 = AmqpValue::Long(8);
        let v10 = AmqpValue::Float(9.0);
        let v11 = AmqpValue::Double(10.0);
        let v12 = AmqpValue::Char('a');
        let v13 = AmqpValue::TimeStamp(AmqpTimestamp(timestamp));
        let v14 = AmqpValue::Uuid(uuid);
        let v15 = AmqpValue::Binary(vec![1, 2, 3]);
        let v16 = AmqpValue::String("hello".to_string());
        let v17 = AmqpValue::Symbol(AmqpSymbol("hello".to_string()));
        let v18 = AmqpValue::List(AmqpList(vec![AmqpValue::Int(1), AmqpValue::Int(2)]));
        let v19 = AmqpValue::Map(AmqpOrderedMap::new());
        let v20 = AmqpValue::Array(vec![AmqpValue::Int(1), AmqpValue::Int(2)]);
        let v21 = AmqpValue::Described(Box::new(AmqpDescribed {
            descriptor: AmqpDescriptor::Code(23),
            value: AmqpValue::Int(2),
        }));
        let v22 = AmqpValue::Described(Box::new(AmqpDescribed {
            descriptor: AmqpDescriptor::Name(AmqpSymbol("name".to_string())),
            value: AmqpValue::Int(2),
        }));
        let v23 = AmqpValue::Unknown;

        assert_eq!(v1, AmqpValue::Boolean(true));
        assert_eq!(v2, AmqpValue::UByte(1));
        assert_eq!(v3, AmqpValue::UShort(2));
        assert_eq!(v4, AmqpValue::UInt(3));
        assert_eq!(v5, AmqpValue::ULong(4));
        assert_eq!(v6, AmqpValue::Byte(5));
        assert_eq!(v7, AmqpValue::Short(6));
        assert_eq!(v8, AmqpValue::Int(7));
        assert_eq!(v9, AmqpValue::Long(8));
        assert_eq!(v10, AmqpValue::Float(9.0));
        assert_eq!(v11, AmqpValue::Double(10.0));
        assert_eq!(v12, AmqpValue::Char('a'));
        assert_eq!(v13, AmqpValue::TimeStamp(AmqpTimestamp(timestamp)));
        assert_eq!(v14, AmqpValue::Uuid(uuid));
        assert_eq!(v15, AmqpValue::Binary(vec![1, 2, 3]));
        assert_eq!(v16, AmqpValue::String("hello".to_string()));
        assert_eq!(v17, AmqpValue::Symbol(AmqpSymbol("hello".to_string())));
        assert_eq!(
            v18,
            AmqpValue::List(AmqpList(vec![AmqpValue::Int(1), AmqpValue::Int(2)]))
        );
        assert_eq!(v19, AmqpValue::Map(AmqpOrderedMap::new()));
        assert_eq!(
            v20,
            AmqpValue::Array(vec![AmqpValue::Int(1), AmqpValue::Int(2)])
        );
        assert_eq!(
            v21,
            AmqpValue::Described(Box::new(AmqpDescribed {
                descriptor: AmqpDescriptor::Code(23),
                value: AmqpValue::Int(2)
            }))
        );
        assert_eq!(
            v22,
            AmqpValue::Described(Box::new(AmqpDescribed {
                descriptor: AmqpDescriptor::Name("name".to_string().into()),
                value: AmqpValue::Int(2)
            }))
        );
        assert_eq!(v23, AmqpValue::Unknown);
    }

    /// Simple conversion tests for the AmqpValue enum
    /// This macro generates a test for each conversion from a specific type to AmqpValue and back
    /// The test checks that the conversion is correct in both directions
    /// The macro also generates a test for the conversion from the unit type to AmqpValue and back
    macro_rules! test_conversion {
        ($t:ty, $field:ident, $value:expr) => {
            let saved_value = $value;
            let v: AmqpValue = saved_value.clone().into();
            assert_eq!(v, AmqpValue::$field(saved_value.clone()));
            assert_eq!(AmqpValue::$field(saved_value.clone()), v);
            let b: $t = v.into();
            assert_eq!(b, saved_value);
        };
        () => {};
    }

    #[test]
    fn test_value_implicit_conversions() {
        test_conversion!(bool, Boolean, true);
        test_conversion!(u8, UByte, 1u8);
        test_conversion!(u16, UShort, 2u16);
        test_conversion!(u32, UInt, 3u32);
        test_conversion!(u64, ULong, 4u64);
        test_conversion!(i8, Byte, 5i8);
        test_conversion!(i16, Short, 6i16);
        test_conversion!(i32, Int, 7i32);
        test_conversion!(i64, Long, 8i64);
        test_conversion!(f32, Float, 9.0f32);
        test_conversion!(f64, Double, 10.0f64);
        test_conversion!(char, Char, 'a');
        test_conversion!(
            AmqpTimestamp,
            TimeStamp,
            AmqpTimestamp(std::time::SystemTime::now())
        );
        test_conversion!(uuid::Uuid, Uuid, uuid::Uuid::new_v4());
        test_conversion!(Vec<u8>, Binary, vec![1, 2, 3]);
        test_conversion!(String, String, "hello".to_string());
        test_conversion!(AmqpSymbol, Symbol, AmqpSymbol("hello".to_string()));
        test_conversion!(
            AmqpList,
            List,
            AmqpList(vec![AmqpValue::Int(1), AmqpValue::Float(2.75f32)])
        );
        test_conversion!(
            Vec<AmqpValue>,
            Array,
            vec![AmqpValue::Int(1), AmqpValue::Int(2)]
        );
        test_conversion!(
            AmqpOrderedMap<AmqpValue, AmqpValue>,
            Map,
            AmqpOrderedMap::new()
        );

        {
            let described = AmqpDescribed {
                descriptor: AmqpDescriptor::Code(23),
                value: AmqpValue::Int(2),
            };
            let v: AmqpValue = AmqpValue::Described(Box::new(described.clone()));
            assert_eq!(v, AmqpValue::Described(Box::new(described.clone())));
            assert_eq!(AmqpValue::Described(Box::new(described.clone())), v);
            let b: AmqpDescribed = v.into();
            assert_eq!(b, described);
        }

        {
            let v: AmqpValue = AmqpValue::Null;
            assert_eq!(v, AmqpValue::Null);
            assert_eq!(AmqpValue::Null, v);
            let b: () = v.into();
            assert_eq!(b, ());
        }

        {
            let v: AmqpValue = AmqpValue::Unknown;
            assert_eq!(v, AmqpValue::Unknown);
        }
    }

    #[test]
    fn test_amqp_ordered_map() {
        let mut map = AmqpOrderedMap::new();
        map.insert("key1", 1);
        map.insert("key2", 2);
        map.insert("key3", 3);

        assert_eq!(map.get("key1"), Some(&1));
        assert_eq!(map.get("key2"), Some(&2));
        assert_eq!(map.get("key3"), Some(&3));
        assert_eq!(map.get("key4"), None);

        assert_eq!(map.remove(&"key1"), Some(1));
        assert_eq!(map.remove(&"key1"), None);
        assert_eq!(map.get("key1"), None);
    }
}
