use std::io::Read;
use std::io;

pub trait KRead {
    fn read1(&mut self) -> io::Result<i32>;
    fn read2(&mut self) -> io::Result<i32>;
    fn read3(&mut self) -> io::Result<i32>;
    fn read4(&mut self) -> io::Result<i32>;
    fn read_string(&mut self, size: i32) -> io::Result<String>;
    fn read_string1(&mut self) -> io::Result<String>;
    fn read_string2(&mut self) -> io::Result<String>;
    fn read_string3(&mut self) -> io::Result<String>;
    fn read_string4(&mut self) -> io::Result<String>;
    fn read_fix_word(&mut self) -> io::Result<f64>; // divide by 2^20 to get f64 from i32
    fn read_scaled_int(&mut self) -> io::Result<f64>; // divide by 2^16 to get f64 from i32
}

impl<T> KRead for T where T: Read {
    fn read1(&mut self) -> io::Result<i32> {
        let mut buf = [0u8];
        self.read_exact(&mut buf)?;
        Ok(buf[0] as i32)
    }

    fn read2(&mut self) -> io::Result<i32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf[2..4])?;
        Ok(i32::from_be_bytes(buf))
    }

    fn read3(&mut self) -> io::Result<i32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf[1..4])?;
        Ok(i32::from_be_bytes(buf))
    }

    fn read4(&mut self) -> io::Result<i32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf[0..4])?;
        Ok(i32::from_be_bytes(buf))
    }

    fn read_string(&mut self, size: i32) -> io::Result<String> {
        assert!(size > 0);
        let mut buf = vec![0u8; size as usize];
        self.read_exact(&mut buf)?;
        Ok(String::from_utf8_lossy(buf.as_slice()).to_string())

    }

    fn read_string1(&mut self) -> io::Result<String> {
        let size = self.read1()?;
        self.read_string(size)
    }

    fn read_string2(&mut self) -> io::Result<String> {
        let size = self.read2()?;
        self.read_string(size)
    }

    fn read_string3(&mut self) -> io::Result<String> {
        let size = self.read3()?;
        self.read_string(size)
    }

    fn read_string4(&mut self) -> io::Result<String> {
        let size = self.read4()?;
        self.read_string(size)
    }

    fn read_fix_word(&mut self) -> io::Result<f64> {
        let fix_word = self.read4()?;
        Ok(fix_word as f64 / 0x100000 as f64)
    }

    fn read_scaled_int(&mut self) -> io::Result<f64> {
        let scaled_int = self.read4()?;
        Ok(scaled_int as f64 / 0x10000 as f64)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;
    use std::io;

    #[test]
    fn can_read_single_byte() -> io::Result<()> {
        let input_data = [47u8, 56u8];
        let mut input_buffer = Cursor::new(input_data);
        assert_eq!(47, input_buffer.read1()?);
        assert_eq!(56, input_buffer.read1()?);
        Ok(())
    }

    #[test]
    fn can_read_double_byte() -> io::Result<()> {
        let input_data = [47u8, 56u8];
        let mut input_buffer = Cursor::new(input_data);
        assert_eq!(12_088, input_buffer.read2()?);
        Ok(())
    }

    #[test]
    fn can_read_triple_byte() -> io::Result<()> {
        let input_data = [47u8, 56u8, 0u8];
        let mut input_buffer = Cursor::new(input_data);
        assert_eq!(12_088*256, input_buffer.read3()?);
        Ok(())
    }

    #[test]
    fn can_read_quadruple_byte() -> io::Result<()> {
        let input_data = [0xffu8, 0xffu8, 0u8, 0u8];
        let mut input_buffer = Cursor::new(input_data);
        assert_eq!(-65_536, input_buffer.read4()?);
        Ok(())
    }

    #[test]
    fn can_read_string() -> io::Result<()> {
        let input_data = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".as_bytes();
        let mut input_buffer = Cursor::new(input_data);
        assert_eq!("ABCDEFGHIJK", input_buffer.read_string(11)?);
        Ok(())
    }

    #[test]
    fn can_read_string1() -> io::Result<()> {
        let input_data = "\x04ABCDEFGHIJKLMNOPQRSTUVWXYZ".as_bytes();
        let mut input_buffer = Cursor::new(input_data);
        assert_eq!("ABCD", input_buffer.read_string1()?);
        Ok(())
    }

    #[test]
    fn can_read_string2() -> io::Result<()> {
        let input_data = "\x00\x05ABCDEFGHIJKLMNOPQRSTUVWXYZ".as_bytes();
        let mut input_buffer = Cursor::new(input_data);
        assert_eq!("ABCDE", input_buffer.read_string2()?);
        Ok(())
    }


    #[test]
    fn can_read_fix_word() -> io::Result<()> {
        let mut input_buffer = Cursor::new([0x00, 0xa0, 0x00, 0x00]);
        assert_eq!(10.0, input_buffer.read_fix_word()?);
        Ok(())
    }

    #[test]
    fn can_read_scaled_int() ->io::Result<()> {
        let mut input_buffer = Cursor::new([0x00, 0x24, 0x00, 0x00]);
        assert_eq!(36.0, input_buffer.read_scaled_int()?);
        Ok(())
    }
}