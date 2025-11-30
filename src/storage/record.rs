/// Record format with Varint encoding
/// 
/// Records in DeepSQL use a compact binary format:
/// - Varint encoding for integers (similar to SQLite)
/// - Type tags for values
/// - Efficient space usage

use crate::error::{Error, Result};

/// Variable-length integer encoding (Varint)
/// 
/// Uses 7 bits per byte for data, 1 bit as continuation flag.
/// More space-efficient for small integers.
pub struct Varint;

impl Varint {
    /// Encode a u64 as a varint
    pub fn encode(mut value: u64) -> Vec<u8> {
        let mut result = Vec::new();
        
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            
            if value != 0 {
                byte |= 0x80; // Set continuation bit
            }
            
            result.push(byte);
            
            if value == 0 {
                break;
            }
        }
        
        result
    }
    
    /// Decode a varint from bytes
    /// Returns (value, bytes_consumed)
    pub fn decode(bytes: &[u8]) -> Result<(u64, usize)> {
        let mut value: u64 = 0;
        let mut shift = 0;
        let mut index = 0;
        
        loop {
            if index >= bytes.len() {
                return Err(Error::RecordError("Incomplete varint".to_string()));
            }
            
            if shift >= 64 {
                return Err(Error::RecordError("Varint overflow".to_string()));
            }
            
            let byte = bytes[index];
            index += 1;
            
            value |= ((byte & 0x7F) as u64) << shift;
            shift += 7;
            
            if (byte & 0x80) == 0 {
                break;
            }
        }
        
        Ok((value, index))
    }
    
    /// Encode a signed integer as a varint using zigzag encoding
    pub fn encode_signed(value: i64) -> Vec<u8> {
        let zigzag = ((value << 1) ^ (value >> 63)) as u64;
        Self::encode(zigzag)
    }
    
    /// Decode a signed integer from a varint using zigzag encoding
    pub fn decode_signed(bytes: &[u8]) -> Result<(i64, usize)> {
        let (zigzag, consumed) = Self::decode(bytes)?;
        let value = ((zigzag >> 1) as i64) ^ -((zigzag & 1) as i64);
        Ok((value, consumed))
    }
}

/// Value types supported by DeepSQL
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// NULL value
    Null,
    
    /// Integer value
    Integer(i64),
    
    /// Floating point value
    Real(f64),
    
    /// Text string (UTF-8)
    Text(String),
    
    /// Binary data
    Blob(Vec<u8>),
}

impl Value {
    /// Get type tag for serialization
    fn type_tag(&self) -> u8 {
        match self {
            Value::Null => 0,
            Value::Integer(_) => 1,
            Value::Real(_) => 2,
            Value::Text(_) => 3,
            Value::Blob(_) => 4,
        }
    }
    
    /// Serialize value to bytes
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.type_tag());
        
        match self {
            Value::Null => {},
            Value::Integer(v) => {
                bytes.extend_from_slice(&Varint::encode_signed(*v));
            },
            Value::Real(v) => {
                bytes.extend_from_slice(&v.to_be_bytes());
            },
            Value::Text(s) => {
                let text_bytes = s.as_bytes();
                bytes.extend_from_slice(&Varint::encode(text_bytes.len() as u64));
                bytes.extend_from_slice(text_bytes);
            },
            Value::Blob(b) => {
                bytes.extend_from_slice(&Varint::encode(b.len() as u64));
                bytes.extend_from_slice(b);
            },
        }
        
        bytes
    }
    
    /// Deserialize value from bytes
    /// Returns (value, bytes_consumed)
    pub fn deserialize(bytes: &[u8]) -> Result<(Value, usize)> {
        if bytes.is_empty() {
            return Err(Error::RecordError("Empty value bytes".to_string()));
        }
        
        let type_tag = bytes[0];
        let mut offset = 1;
        
        let value = match type_tag {
            0 => Value::Null,
            1 => {
                let (v, consumed) = Varint::decode_signed(&bytes[offset..])?;
                offset += consumed;
                Value::Integer(v)
            },
            2 => {
                if bytes.len() < offset + 8 {
                    return Err(Error::RecordError("Incomplete real value".to_string()));
                }
                let mut float_bytes = [0u8; 8];
                float_bytes.copy_from_slice(&bytes[offset..offset + 8]);
                offset += 8;
                Value::Real(f64::from_be_bytes(float_bytes))
            },
            3 => {
                let (len, consumed) = Varint::decode(&bytes[offset..])?;
                offset += consumed;
                
                if bytes.len() < offset + len as usize {
                    return Err(Error::RecordError("Incomplete text value".to_string()));
                }
                
                let text_bytes = &bytes[offset..offset + len as usize];
                offset += len as usize;
                
                let text = String::from_utf8(text_bytes.to_vec())
                    .map_err(|e| Error::RecordError(format!("Invalid UTF-8: {}", e)))?;
                
                Value::Text(text)
            },
            4 => {
                let (len, consumed) = Varint::decode(&bytes[offset..])?;
                offset += consumed;
                
                if bytes.len() < offset + len as usize {
                    return Err(Error::RecordError("Incomplete blob value".to_string()));
                }
                
                let blob = bytes[offset..offset + len as usize].to_vec();
                offset += len as usize;
                
                Value::Blob(blob)
            },
            _ => return Err(Error::RecordError(format!("Invalid type tag: {}", type_tag))),
        };
        
        Ok((value, offset))
    }
}

/// A record consists of a key and a list of values
#[derive(Debug, Clone)]
pub struct Record {
    /// Record key (used for B+Tree ordering)
    pub key: Vec<u8>,
    
    /// Record values
    pub values: Vec<Value>,
}

impl Record {
    /// Create a new record
    pub fn new(key: Vec<u8>, values: Vec<Value>) -> Self {
        Record { key, values }
    }
    
    /// Serialize record to bytes
    /// Format: key_len (varint) | key | value_count (varint) | values...
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Serialize key
        bytes.extend_from_slice(&Varint::encode(self.key.len() as u64));
        bytes.extend_from_slice(&self.key);
        
        // Serialize value count
        bytes.extend_from_slice(&Varint::encode(self.values.len() as u64));
        
        // Serialize values
        for value in &self.values {
            bytes.extend_from_slice(&value.serialize());
        }
        
        bytes
    }
    
    /// Deserialize record from bytes
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        let mut offset = 0;
        
        // Deserialize key
        let (key_len, consumed) = Varint::decode(&bytes[offset..])?;
        offset += consumed;
        
        if bytes.len() < offset + key_len as usize {
            return Err(Error::RecordError("Incomplete record key".to_string()));
        }
        
        let key = bytes[offset..offset + key_len as usize].to_vec();
        offset += key_len as usize;
        
        // Deserialize value count
        let (value_count, consumed) = Varint::decode(&bytes[offset..])?;
        offset += consumed;
        
        // Deserialize values
        let mut values = Vec::with_capacity(value_count as usize);
        for _ in 0..value_count {
            let (value, consumed) = Value::deserialize(&bytes[offset..])?;
            offset += consumed;
            values.push(value);
        }
        
        Ok(Record { key, values })
    }
    
    /// Get the total serialized size of this record
    pub fn serialized_size(&self) -> usize {
        self.serialize().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_varint_encode_decode() {
        let test_cases = vec![0, 1, 127, 128, 16383, 16384, u64::MAX];
        
        for value in test_cases {
            let encoded = Varint::encode(value);
            let (decoded, _) = Varint::decode(&encoded).unwrap();
            assert_eq!(value, decoded);
        }
    }
    
    #[test]
    fn test_varint_signed() {
        let test_cases = vec![0, 1, -1, 127, -127, i64::MAX, i64::MIN];
        
        for value in test_cases {
            let encoded = Varint::encode_signed(value);
            let (decoded, _) = Varint::decode_signed(&encoded).unwrap();
            assert_eq!(value, decoded);
        }
    }
    
    #[test]
    fn test_value_serialization() {
        let test_cases = vec![
            Value::Null,
            Value::Integer(42),
            Value::Integer(-100),
            Value::Real(3.14159),
            Value::Text("Hello, World!".to_string()),
            Value::Blob(vec![1, 2, 3, 4, 5]),
        ];
        
        for value in test_cases {
            let serialized = value.serialize();
            let (deserialized, _) = Value::deserialize(&serialized).unwrap();
            assert_eq!(value, deserialized);
        }
    }
    
    #[test]
    fn test_record_serialization() {
        let record = Record::new(
            vec![1, 2, 3],
            vec![
                Value::Integer(42),
                Value::Text("test".to_string()),
                Value::Null,
            ],
        );
        
        let serialized = record.serialize();
        let deserialized = Record::deserialize(&serialized).unwrap();
        
        assert_eq!(record.key, deserialized.key);
        assert_eq!(record.values.len(), deserialized.values.len());
        
        for (v1, v2) in record.values.iter().zip(deserialized.values.iter()) {
            assert_eq!(v1, v2);
        }
    }
}

