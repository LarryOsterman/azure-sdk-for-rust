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
    TimeStamp(i64),
    Uuid(uuid::Uuid),
    Binary(Vec<u8>),
    String(String),
    Symbol(AmqpSymbol),
    List(AmqpList),
    Map(AmqpOrderedMap<AmqpValue, AmqpValue>),
    Array(Vec<AmqpValue>),
    Described(Box<AmqpValue>, Box<AmqpValue>),
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

impl From<bool> for AmqpValue {
    fn from(b: bool) -> Self {
        AmqpValue::Boolean(b)
    }
}

impl From<u8> for AmqpValue {
    fn from(b: u8) -> Self {
        AmqpValue::UByte(b)
    }
}

impl From<u16> for AmqpValue {
    fn from(b: u16) -> Self {
        AmqpValue::UShort(b)
    }
}

impl From<u32> for AmqpValue {
    fn from(b: u32) -> Self {
        AmqpValue::UInt(b)
    }
}

impl From<u64> for AmqpValue {
    fn from(b: u64) -> Self {
        AmqpValue::ULong(b)
    }
}

impl From<i8> for AmqpValue {
    fn from(b: i8) -> Self {
        AmqpValue::Byte(b)
    }
}

impl From<i16> for AmqpValue {
    fn from(b: i16) -> Self {
        AmqpValue::Short(b)
    }
}

impl From<i32> for AmqpValue {
    fn from(b: i32) -> Self {
        AmqpValue::Int(b)
    }
}

impl Into<i32> for AmqpValue {
    fn into(self) -> i32 {
        match self {
            AmqpValue::Int(i) => i,
            _ => panic!("Expected an int, found: {:?}", self),
        }
    }
}

impl From<i64> for AmqpValue {
    fn from(b: i64) -> Self {
        AmqpValue::Long(b)
    }
}

impl Into<i64> for AmqpValue {
    fn into(self) -> i64 {
        match self {
            AmqpValue::Long(l) => l,
            _ => panic!("Expected a long"),
        }
    }
}

impl From<f32> for AmqpValue {
    fn from(b: f32) -> Self {
        AmqpValue::Float(b)
    }
}

impl From<f64> for AmqpValue {
    fn from(b: f64) -> Self {
        AmqpValue::Double(b)
    }
}

impl From<char> for AmqpValue {
    fn from(b: char) -> Self {
        AmqpValue::Char(b)
    }
}

impl From<uuid::Uuid> for AmqpValue {
    fn from(b: uuid::Uuid) -> Self {
        AmqpValue::Uuid(b)
    }
}

impl From<Vec<u8>> for AmqpValue {
    fn from(b: Vec<u8>) -> Self {
        AmqpValue::Binary(b)
    }
}

impl From<String> for AmqpValue {
    fn from(b: String) -> Self {
        AmqpValue::String(b)
    }
}

impl Into<String> for AmqpValue {
    fn into(self) -> String {
        match self {
            AmqpValue::String(s) => s,
            _ => panic!("Expected a string"),
        }
    }
}

impl From<&str> for AmqpValue {
    fn from(b: &str) -> Self {
        AmqpValue::String(b.to_string())
    }
}

impl From<AmqpList> for AmqpValue {
    fn from(l: AmqpList) -> Self {
        AmqpValue::List(l)
    }
}

impl From<AmqpOrderedMap<AmqpValue, AmqpValue>> for AmqpValue {
    fn from(m: AmqpOrderedMap<AmqpValue, AmqpValue>) -> Self {
        AmqpValue::Map(m)
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

impl From<AmqpSymbol> for AmqpValue {
    fn from(s: AmqpSymbol) -> Self {
        AmqpValue::Symbol(s)
    }
}

impl From<std::time::SystemTime> for AmqpValue {
    fn from(t: std::time::SystemTime) -> Self {
        let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap();
        AmqpValue::TimeStamp(duration.as_secs() as i64)
    }
}

impl Into<std::time::SystemTime> for AmqpValue {
    fn into(self) -> std::time::SystemTime {
        match self {
            AmqpValue::TimeStamp(t) => {
                std::time::UNIX_EPOCH + std::time::Duration::from_millis(t as u64)
            }
            _ => panic!("Expected a timestamp"),
        }
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
