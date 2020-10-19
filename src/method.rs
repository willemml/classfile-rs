use crate::access::MethodAccessFlags;
use crate::attributes::{Attribute, Attributes, AttributeSource};
use crate::version::ClassVersion;
use crate::constantpool::ConstantPool;
use std::io::{Seek, Read, Write};
use crate::Serializable;
use byteorder::{BigEndian, ReadBytesExt};
use crate::error::Result;
use crate::utils::mut_retain;

#[allow(non_snake_case)]
pub mod Methods {
	use std::io::{Seek, Read, Write};
	use crate::method::Method;
	use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
	use crate::version::ClassVersion;
	use crate::constantpool::ConstantPool;
	
	pub fn parse<T: Seek + Read>(rdr: &mut T, version: &ClassVersion, constant_pool: &ConstantPool) -> crate::Result<Vec<Method>> {
		let num_fields = rdr.read_u16::<BigEndian>()? as usize;
		let mut fields: Vec<Method> = Vec::with_capacity(num_fields);
		for _ in 0..num_fields {
			fields.push(Method::parse(rdr, version, constant_pool)?);
		}
		Ok(fields)
	}
	
	pub fn write<T: Seek + Write>(wtr: &mut T, fields: &Vec<Method>, constant_pool: &ConstantPool) -> crate::Result<()> {
		wtr.write_u16::<BigEndian>(fields.len() as u16)?;
		for field in fields.iter() {
			field.write(wtr, constant_pool)?;
		}
		Ok(())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Method {
	access_flags: MethodAccessFlags,
	name: String,
	descriptor: String,
	signature: Option<String>,
	exceptions: Vec<String>,
	attributes: Vec<Attribute>
}

impl Method {
	pub fn parse<R: Seek + Read>(rdr: &mut R, version: &ClassVersion, constant_pool: &ConstantPool) -> Result<Self> {
		let access_flags = MethodAccessFlags::parse(rdr)?;
		let name = constant_pool.utf8(rdr.read_u16::<BigEndian>()?)?.str.clone();
		let descriptor = constant_pool.utf8(rdr.read_u16::<BigEndian>()?)?.str.clone();
		let mut signature: Option<String> = None;
		let mut exceptions: Vec<String> = Vec::new();
		let mut attributes = Attributes::parse(rdr, AttributeSource::Method, version, constant_pool)?;
		
		mut_retain(&mut attributes, |attribute| {
			match attribute {
				Attribute::Signature(signature_attr) => {
					// The attribute will be dropped, so instead of cloning we can swap an empty string for the signature
					let mut rep = String::new();
					std::mem::swap(&mut rep, &mut signature_attr.signature);
					signature = Some(rep);
					false
				},
				Attribute::Exceptions(exceptions_attr) => {
					std::mem::swap(&mut exceptions, &mut exceptions_attr.exceptions);
					false
				},
				_ => true
			}
		});
		
		Ok(Method {
			access_flags,
			name,
			descriptor,
			signature,
			exceptions,
			attributes
		})
	}
	
	pub fn write<W: Seek + Write>(&self, wtr: &mut W, constant_pool: &ConstantPool) -> Result<()> {
		self.access_flags.write(wtr)?;
		Attributes::write(wtr, &self.attributes, constant_pool)?;
		Ok(())
	}
}
