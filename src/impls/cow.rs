use std::borrow::{Borrow, Cow};

use no_std_io::io::{Read, Write};

use bitvec::prelude::*;

use crate::reader::Reader;
use crate::writer::Writer;
use crate::{DekuError, DekuReader, DekuWrite, DekuWriter};

impl<'a, T, Ctx> DekuReader<'a, Ctx> for Cow<'a, T>
where
    T: DekuReader<'a, Ctx> + Clone,
    Ctx: Copy,
{
    fn from_reader_with_ctx<R: Read>(
        reader: &mut Reader<R>,
        inner_ctx: Ctx,
    ) -> Result<Self, DekuError> {
        let val = <T>::from_reader_with_ctx(reader, inner_ctx)?;
        Ok(Cow::Owned(val))
    }
}

impl<T, Ctx> DekuWrite<Ctx> for Cow<'_, T>
where
    T: DekuWrite<Ctx> + Clone,
    Ctx: Copy,
{
    /// Write T from Cow<T>
    fn write(&self, output: &mut BitVec<u8, Msb0>, inner_ctx: Ctx) -> Result<(), DekuError> {
        (self.borrow() as &T).write(output, inner_ctx)
    }
}

impl<T, Ctx> DekuWriter<Ctx> for Cow<'_, T>
where
    T: DekuWriter<Ctx> + Clone,
    Ctx: Copy,
{
    /// Write T from Cow<T>
    fn to_writer<W: Write>(&self, writer: &mut Writer<W>, inner_ctx: Ctx) -> Result<(), DekuError> {
        (self.borrow() as &T).to_writer(writer, inner_ctx)
    }
}

#[cfg(test)]
mod tests {
    use no_std_io::io::Cursor;
    use rstest::rstest;

    use super::*;
    use crate::{native_endian, reader::Reader};

    #[rstest(input, expected,
        case(
            &[0xEF, 0xBE],
            Cow::Owned(native_endian!(0xBEEF_u16)),
        ),
    )]
    fn test_cow(input: &[u8], expected: Cow<u16>) {
        let mut cursor = Cursor::new(input);
        let mut reader = Reader::new(&mut cursor);
        let res_read = <Cow<u16>>::from_reader_with_ctx(&mut reader, ()).unwrap();
        assert_eq!(expected, res_read);

        let mut res_write = bitvec![u8, Msb0;];
        res_read.write(&mut res_write, ()).unwrap();
        assert_eq!(input.to_vec(), res_write.into_vec());

        let mut out_buf = vec![];
        let mut writer = Writer::new(&mut out_buf);
        res_read.to_writer(&mut writer, ()).unwrap();
        assert_eq!(input.to_vec(), out_buf.to_vec());
    }
}
