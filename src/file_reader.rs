use std::io::Read;
use std::io;


pub fn read1<R: Read>(input: &mut R) -> io::Result<i32> {
    let mut buf = [0u8];
    input.read(&mut buf)?;
    Ok(buf[0] as i32)
}

pub fn read2<R: Read>(input: &mut R) -> io::Result<i32> {
    let mut buf = [0u8; 4];
    input.read(&mut buf[2..4])?;
    Ok(i32::from_be_bytes(buf))
}

pub fn read3<R: Read>(input: &mut R) -> io::Result<i32> {
    let mut buf = [0u8; 4];
    input.read(&mut buf[1..4])?;
    Ok(i32::from_be_bytes(buf))
}

pub fn read4<R: Read>(input: &mut R) -> io::Result<i32> {
    let mut buf = [0u8; 4];
    input.read(&mut buf[0..4])?;
    Ok(i32::from_be_bytes(buf))
}

pub fn read_string<R: Read>(input: &mut R, size: i32) -> io::Result<String> {
    assert!(size > 0);
    let mut buf = vec![0u8; size as usize];
    input.read_exact(&mut buf)?;
    Ok(String::from_utf8_lossy(buf.as_slice()).to_string())
    // match String::from_utf8(buf) {
    //     Ok (v) => Ok(v),
    //     Err (e) => Err(io::Error::new(ErrorKind::Other, e))
    // }
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
        assert_eq!(47, read1(&mut input_buffer)?);
        assert_eq!(56, read1(&mut input_buffer)?);
        Ok(())
    }

    #[test]
    fn can_read_double_byte() -> io::Result<()> {
        let input_data = [47u8, 56u8];
        let mut input_buffer = Cursor::new(input_data);
        assert_eq!(12_088, read2(&mut input_buffer)?);
        Ok(())
    }

    #[test]
    fn can_read_triple_byte() -> io::Result<()> {
        let input_data = [47u8, 56u8, 0u8];
        let mut input_buffer = Cursor::new(input_data);
        assert_eq!(12_088*256, read3(&mut input_buffer)?);
        Ok(())
    }

    #[test]
    fn can_read_quadruple_byte() -> io::Result<()> {
        let input_data = [0xffu8, 0xffu8, 0u8, 0u8];
        let mut input_buffer = Cursor::new(input_data);
        assert_eq!(-65_536, read4(&mut input_buffer)?);
        Ok(())
    }

    #[test]
    fn can_read_string() -> io::Result<()> {
        let input_data = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".as_bytes();
        let mut input_buffer = Cursor::new(input_data);
        assert_eq!("ABCDEFGHIJK", read_string(&mut input_buffer, 11)?);
        Ok(())
    }

}