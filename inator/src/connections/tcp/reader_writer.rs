use std::io::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use crate::connections::{BytesOptions, OrderOptions, ReadValue};

pub fn value_to_bytes(value: &ReadValue, order: &OrderOptions) -> Vec<u8> {
    match value {
        // Unsigned
        ReadValue::U8(v) => vec![*v],
        ReadValue::U16(v) => match order {
            OrderOptions::LittleEndian => v.to_le_bytes().to_vec(),
            OrderOptions::BigEndian => v.to_be_bytes().to_vec(),
        },
        ReadValue::U32(v) => match order {
            OrderOptions::LittleEndian => v.to_le_bytes().to_vec(),
            OrderOptions::BigEndian => v.to_be_bytes().to_vec(),
        },
        ReadValue::U64(v) => match order {
            OrderOptions::LittleEndian => v.to_le_bytes().to_vec(),
            OrderOptions::BigEndian => v.to_be_bytes().to_vec(),
        },
        ReadValue::U128(v) => match order {
            OrderOptions::LittleEndian => v.to_le_bytes().to_vec(),
            OrderOptions::BigEndian => v.to_be_bytes().to_vec(),
        },

        // Signed
        ReadValue::I8(v) => vec![*v as u8],
        ReadValue::I16(v) => match order {
            OrderOptions::LittleEndian => v.to_le_bytes().to_vec(),
            OrderOptions::BigEndian => v.to_be_bytes().to_vec(),
        },
        ReadValue::I32(v) => match order {
            OrderOptions::LittleEndian => v.to_le_bytes().to_vec(),
            OrderOptions::BigEndian => v.to_be_bytes().to_vec(),
        },
        ReadValue::I64(v) => match order {
            OrderOptions::LittleEndian => v.to_le_bytes().to_vec(),
            OrderOptions::BigEndian => v.to_be_bytes().to_vec(),
        },
        ReadValue::I128(v) => match order {
            OrderOptions::LittleEndian => v.to_le_bytes().to_vec(),
            OrderOptions::BigEndian => v.to_be_bytes().to_vec(),
        },

        // Floats
        ReadValue::F32(v) => match order {
            OrderOptions::LittleEndian => v.to_le_bytes().to_vec(),
            OrderOptions::BigEndian => v.to_be_bytes().to_vec(),
        },
        ReadValue::F64(v) => match order {
            OrderOptions::LittleEndian => v.to_le_bytes().to_vec(),
            OrderOptions::BigEndian => v.to_be_bytes().to_vec(),
        },
    }
}

pub async fn write_from_settings(
    write_half: &mut OwnedWriteHalf,
    encoded: &[u8],
    bytes: &BytesOptions,
    order: &OrderOptions,
) -> Result<(), Error> {
    let len = encoded.len();
    
    match bytes {
        // Unsigned
        BytesOptions::U8 => write_half.write_u8(len as u8).await?,
        BytesOptions::U16 => match order {
            OrderOptions::LittleEndian => write_half.write_u16_le(len as u16).await?,
            OrderOptions::BigEndian => write_half.write_u16(len as u16).await?,
        },
        BytesOptions::U32 => match order {
            OrderOptions::LittleEndian => write_half.write_u32_le(len as u32).await?,
            OrderOptions::BigEndian => write_half.write_u32(len as u32).await?,
        },
        BytesOptions::U64 => match order {
            OrderOptions::LittleEndian => write_half.write_u64_le(len as u64).await?,
            OrderOptions::BigEndian => write_half.write_u64(len as u64).await?,
        },
        BytesOptions::U128 => match order {
            OrderOptions::LittleEndian => write_half.write_u128_le(len as u128).await?,
            OrderOptions::BigEndian => write_half.write_u128(len as u128).await?,
        },

        // Signed
        BytesOptions::I8 => write_half.write_i8(len as i8).await?,
        BytesOptions::I16 => match order {
            OrderOptions::LittleEndian => write_half.write_i16_le(len as i16).await?,
            OrderOptions::BigEndian => write_half.write_i16(len as i16).await?,
        },
        BytesOptions::I32 => match order {
            OrderOptions::LittleEndian => write_half.write_i32_le(len as i32).await?,
            OrderOptions::BigEndian => write_half.write_i32(len as i32).await?,
        },
        BytesOptions::I64 => match order {
            OrderOptions::LittleEndian => write_half.write_i64_le(len as i64).await?,
            OrderOptions::BigEndian => write_half.write_i64(len as i64).await?,
        },
        BytesOptions::I128 => match order {
            OrderOptions::LittleEndian => write_half.write_i128_le(len as i128).await?,
            OrderOptions::BigEndian => write_half.write_i128(len as i128).await?,
        },

        // Float 
        BytesOptions::F32 => match order {
            OrderOptions::LittleEndian => write_half.write_f32_le(len as f32).await?,
            OrderOptions::BigEndian => write_half.write_f32(len as f32).await?,
        },
        BytesOptions::F64 => match order {
            OrderOptions::LittleEndian => write_half.write_f64_le(len as f64).await?,
            OrderOptions::BigEndian => write_half.write_f64(len as f64).await?,
        },
    }

    Ok(())
}

pub async fn read_from_settings(
    read_half: &mut OwnedReadHalf,
    bytes: &BytesOptions,
    order: &OrderOptions,
) -> Result<ReadValue, tokio::io::Error> {
    let value = match bytes {
        BytesOptions::U8 => ReadValue::U8(read_half.read_u8().await?),
        BytesOptions::U16 => ReadValue::U16(match order {
            OrderOptions::LittleEndian => read_half.read_u16_le().await?,
            OrderOptions::BigEndian => read_half.read_u16().await?,
        }),
        BytesOptions::U32 => ReadValue::U32(match order {
            OrderOptions::LittleEndian => read_half.read_u32_le().await?,
            OrderOptions::BigEndian => read_half.read_u32().await?,
        }),
        BytesOptions::U64 => ReadValue::U64(match order {
            OrderOptions::LittleEndian => read_half.read_u64_le().await?,
            OrderOptions::BigEndian => read_half.read_u64().await?,
        }),
        BytesOptions::U128 => ReadValue::U128(match order {
            OrderOptions::LittleEndian => read_half.read_u128_le().await?,
            OrderOptions::BigEndian => read_half.read_u128().await?,
        }),

        BytesOptions::I8 => ReadValue::I8(read_half.read_i8().await?),
        BytesOptions::I16 => ReadValue::I16(match order {
            OrderOptions::LittleEndian => read_half.read_i16_le().await?,
            OrderOptions::BigEndian => read_half.read_i16().await?,
        }),
        BytesOptions::I32 => ReadValue::I32(match order {
            OrderOptions::LittleEndian => read_half.read_i32_le().await?,
            OrderOptions::BigEndian => read_half.read_i32().await?,
        }),
        BytesOptions::I64 => ReadValue::I64(match order {
            OrderOptions::LittleEndian => read_half.read_i64_le().await?,
            OrderOptions::BigEndian => read_half.read_i64().await?,
        }),
        BytesOptions::I128 => ReadValue::I128(match order {
            OrderOptions::LittleEndian => read_half.read_i128_le().await?,
            OrderOptions::BigEndian => read_half.read_i128().await?,
        }),

        BytesOptions::F32 => ReadValue::F32(match order {
            OrderOptions::LittleEndian => read_half.read_f32_le().await?,
            OrderOptions::BigEndian => read_half.read_f32().await?,
        }),
        BytesOptions::F64 => ReadValue::F64(match order {
            OrderOptions::LittleEndian => read_half.read_f64_le().await?,
            OrderOptions::BigEndian => read_half.read_f64().await?,
        }),
    };

    Ok(value)
}
