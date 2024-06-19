// cspell: words amqp

#[derive(Debug, PartialEq, Clone)]
pub struct AmqpSymbol(pub String);

#[derive(Debug, PartialEq, Clone)]
pub struct AmqpList(pub Vec<AmqpValue>);

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
    TimeStamp(std::time::SystemTime),
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

    pub fn get(&self, key: K) -> Option<&V> {
        self.inner
            .iter()
            .find_map(|(k, v)| if *k == key { Some(v) } else { None })
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let index = self.inner.iter().position(|(k, _)| k == key)?;
        Some(self.inner.remove(index).1)
    }

    pub fn contains_key(&self, key: K) -> bool {
        self.inner.iter().any(|(k, _)| *k == key)
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
conversions_for_amqp_types!(std::time::SystemTime, TimeStamp);

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
        let v13 = AmqpValue::TimeStamp(timestamp);
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
        assert_eq!(v13, AmqpValue::TimeStamp(timestamp));
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

    #[test]
    fn test_value_implicit() {
        {
            let v: AmqpValue = true.into();
            assert_eq!(v, true);
            assert_eq!(true, v);
            let b: bool = v.into();
            assert_eq!(b, true);
        }
        {
            let v: AmqpValue = 1u8.into();
            assert_eq!(v, 1u8);
            assert_eq!(1u8, v);
            let b: u8 = v.into();
            assert_eq!(b, 1u8);
        }
        {
            let v: AmqpValue = 2u16.into();
            assert_eq!(v, 2u16);
            assert_eq!(2u16, v);
            let b: u16 = v.into();
            assert_eq!(b, 2u16);
        }
        {
            let v: AmqpValue = 3u32.into();
            assert_eq!(v, 3u32);
            assert_eq!(3u32, v);
            let b: u32 = v.into();
            assert_eq!(b, 3u32);
        }
        {
            let v: AmqpValue = 4u64.into();
            assert_eq!(v, 4u64);
            assert_eq!(4u64, v);
            let b: u64 = v.into();
            assert_eq!(b, 4u64);
        }
        {
            let v: AmqpValue = 5i8.into();
            assert_eq!(v, 5i8);
            assert_eq!(5i8, v);
            let b: i8 = v.into();
            assert_eq!(b, 5i8);
        }
        {
            let v: AmqpValue = 6i16.into();
            assert_eq!(v, 6i16);
            assert_eq!(6i16, v);
            let b: i16 = v.into();
            assert_eq!(b, 6i16);
        }
        {
            let v: AmqpValue = 7i32.into();
            assert_eq!(v, 7i32);
            assert_eq!(7i32, v);
            let b: i32 = v.into();
            assert_eq!(b, 7i32);
        }
        {
            let v: AmqpValue = 8i64.into();
            assert_eq!(v, 8i64);
            assert_eq!(8i64, v);
            let b: i64 = v.into();
            assert_eq!(b, 8i64);
        }
        {
            let v: AmqpValue = 9.0f32.into();
            assert_eq!(v, 9.0f32);
            assert_eq!(9.0f32, v);
            let b: f32 = v.into();
            assert_eq!(b, 9.0f32);
        }
        {
            let v: AmqpValue = 10.0f64.into();
            assert_eq!(v, 10.0f64);
            assert_eq!(10.0f64, v);
            let b: f64 = v.into();
            assert_eq!(b, 10.0f64);
        }
        {
            let v: AmqpValue = 'a'.into();
            assert_eq!(v, 'a');
            assert_eq!('a', v);
            let b: char = v.into();
            assert_eq!(b, 'a');
        }
        {
            let timestamp = std::time::SystemTime::now();
            let v: AmqpValue = timestamp.into();
            assert_eq!(v, timestamp);
            assert_eq!(timestamp, v);
            let b: std::time::SystemTime = v.into();
            assert_eq!(b, timestamp);
        }

        {
            let uuid = uuid::Uuid::new_v4();
            let v: AmqpValue = uuid.into();
            assert_eq!(v, uuid);
            assert_eq!(uuid, v);
            let b: uuid::Uuid = v.into();
            assert_eq!(b, uuid);
        }
        {
            let v: AmqpValue = vec![1, 2, 3].into();
            assert_eq!(v, vec![1, 2, 3]);
            assert_eq!(vec![1, 2, 3], v);
            let b: Vec<u8> = v.into();
            assert_eq!(b, vec![1, 2, 3]);
        }
        {
            let v: AmqpValue = "hello".into();
            assert_eq!(v, "hello".to_string());
            assert_eq!("hello".to_string(), v);
            let b: String = v.into();
            assert_eq!(b, "hello".to_string());
        }
        {
            let v: AmqpValue = AmqpSymbol("hello".to_string()).into();
            assert_eq!(v, AmqpSymbol("hello".to_string()));
            assert_eq!(AmqpSymbol("hello".to_string()), v);
            let b: AmqpSymbol = v.into();
            assert_eq!(b, AmqpSymbol("hello".to_string()));
        }
        {
            let v: AmqpValue = AmqpList(vec![AmqpValue::Int(1), AmqpValue::Int(2)]).into();
            assert_eq!(v, AmqpList(vec![AmqpValue::Int(1), AmqpValue::Int(2)]));
            assert_eq!(AmqpList(vec![AmqpValue::Int(1), AmqpValue::Int(2)]), v);
            let b: AmqpList = v.into();
            assert_eq!(b, AmqpList(vec![AmqpValue::Int(1), AmqpValue::Int(2)]));
        }
        {
            let v: AmqpValue = AmqpOrderedMap::new().into();
            assert_eq!(v, AmqpOrderedMap::new());
            assert_eq!(AmqpOrderedMap::new(), v);
            let b: AmqpOrderedMap<AmqpValue, AmqpValue> = v.into();
            assert_eq!(b, AmqpOrderedMap::new());
        }
        {
            let v: AmqpValue = vec![AmqpValue::Int(1), AmqpValue::Int(2)].into();
            assert_eq!(v, vec![AmqpValue::Int(1), AmqpValue::Int(2)]);
            assert_eq!(vec![AmqpValue::Int(1), AmqpValue::Int(2)], v);
            let b: Vec<AmqpValue> = v.into();
            assert_eq!(b, vec![AmqpValue::Int(1), AmqpValue::Int(2)]);
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
