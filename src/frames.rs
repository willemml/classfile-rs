use std::{convert::TryFrom, io::Write};

use crate::error::{ParserError, Result};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

#[derive(Clone, Debug, PartialEq)]
pub struct StackMapTableAttribute {
    pub stack_map_frames: Vec<StackMapFrame>,
}

impl StackMapTableAttribute {
    pub fn parse(buf: Vec<u8>) -> Result<Self> {
        let mut buf = buf.as_slice();
        let num_stack_map_frames = buf.read_u16::<BigEndian>()?;
        let mut stack_map_frames = Vec::with_capacity(num_stack_map_frames as usize);
        for _ in 0..num_stack_map_frames {
            stack_map_frames.push(StackMapFrame::parse(buf)?);
        }
        Ok(Self { stack_map_frames })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VerificationTypeInfo {
    Top,
    Integer,
    Float,
    Long,
    Double,
    Null,
    UninitializedThis,
    Object(u16),
    Uninitialized(u16),
}

impl VerificationTypeInfo {
    pub fn parse(mut buf: &[u8]) -> Result<Self> {
        let value = buf.read_u8()?;
        Ok(match value {
            0 => Self::Top,
            1 => Self::Integer,
            2 => Self::Float,
            3 => Self::Long,
            4 => Self::Double,
            5 => Self::Null,
            6 => Self::UninitializedThis,
            7 => Self::Object(buf.read_u16::<BigEndian>()?),
            8 => Self::Uninitialized(buf.read_u16::<BigEndian>()?),
            _ => {
                return Err(ParserError::Unrecognized(
                    "verification type info",
                    value.to_string(),
                ))
            }
        })
    }

    pub fn write<T: Write>(&self, wtr: &mut T) -> Result<()> {
        match self {
            Self::Top => wtr.write_u8(0)?,
            Self::Integer => wtr.write_u8(1)?,
            Self::Float => wtr.write_u8(2)?,
            Self::Long => wtr.write_u8(3)?,
            Self::Double => wtr.write_u8(4)?,
            Self::Null => wtr.write_u8(5)?,
            Self::UninitializedThis => wtr.write_u8(6)?,
            Self::Object(constant_pool_index) => {
                wtr.write_u8(7)?;
                wtr.write_u16::<BigEndian>(*constant_pool_index)?;
            }
            Self::Uninitialized(offset) => {
                wtr.write_u8(7)?;
                wtr.write_u16::<BigEndian>(*offset)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StackMapFrame {
    Same(u8),
    SameLocals1StackItemFrame(u8, VerificationTypeInfo),
    SameLocals1StackItemFrameExtended(u8, u16, VerificationTypeInfo),
    ChopFrame(u8, u16),
    SameFrameExtended(u8, u16),
    AppendFrame(u8, u16, Vec<VerificationTypeInfo>),
    FullFrame(
        u8,
        u16,
        Vec<VerificationTypeInfo>,
        Vec<VerificationTypeInfo>,
    ),
}

impl StackMapFrame {
    pub fn parse(mut buf: &[u8]) -> Result<Self> {
        let frame_type_num = buf.read_u8()?;
        Ok(match frame_type_num {
            0..=63 => StackMapFrame::Same(frame_type_num),
            64..=127 => StackMapFrame::SameLocals1StackItemFrame(
                frame_type_num,
                VerificationTypeInfo::parse(buf)?,
            ),
            128..=246 => {
                return Err(ParserError::Unrecognized(
                    "frame type",
                    frame_type_num.to_string(),
                ))
            }
            247 => StackMapFrame::SameLocals1StackItemFrameExtended(
                frame_type_num,
                buf.read_u16::<BigEndian>()?,
                VerificationTypeInfo::parse(buf)?,
            ),
            248..=250 => StackMapFrame::ChopFrame(frame_type_num, buf.read_u16::<BigEndian>()?),
            251 => StackMapFrame::SameFrameExtended(frame_type_num, buf.read_u16::<BigEndian>()?),
            252..=254 => {
                let offset_delta = buf.read_u16::<BigEndian>()?;
                let mut locals = Vec::new();
                for _ in 0..(frame_type_num - 251) {
                    locals.push(VerificationTypeInfo::parse(buf)?);
                }
                StackMapFrame::AppendFrame(frame_type_num, offset_delta, locals)
            }
            255 => {
                let offset_delta = buf.read_u16::<BigEndian>()?;
                let num_locals = buf.read_u16::<BigEndian>()?;
                let mut locals = Vec::with_capacity(num_locals as usize);
                for _ in 0..num_locals {
                    locals.push(VerificationTypeInfo::parse(buf)?);
                }
                let num_stack = buf.read_u16::<BigEndian>()?;
                let mut stack = Vec::with_capacity(num_stack as usize);
                for _ in 0..num_stack {
                    stack.push(VerificationTypeInfo::parse(buf)?);
                }
                StackMapFrame::FullFrame(frame_type_num, offset_delta, locals, stack)
            }
        })
    }

    pub fn write<T: Write>(&self, wtr: &mut T) -> Result<()> {
        Ok(())
    }
}
