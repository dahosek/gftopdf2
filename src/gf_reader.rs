use std::io::Read;
use anyhow;
use super::file_reader::*;
use thiserror::Error;

pub struct FontData {
    pub title: String,
    pub chars: Vec<CharData>
}

#[derive(Clone, Debug)]
pub struct CharData {
    pub code: i32,
    pub min_m: i32,
    pub max_m: i32,
    pub min_n: i32,
    pub max_n: i32,
    pub specials: Vec<Special>,
    pub bitmap: Vec<BlackLine>
}

#[derive(Eq, PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct BlackLine {
    pub x: i32,
    pub y: i32,
    pub w: i32
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Special {
    pub special: String,
    pub numeric_params: Vec<i32>
}

struct Context<I: Read> {
    finished: bool,
    font_data: FontData,
    current_character: CharacterContext,
    specials: Vec<Special>,
    input: I,
}

#[derive(Eq, PartialEq)]
#[derive(Debug)]
enum Color {
    Black,
    White
}

struct CharacterContext {
    char_data: CharData,
    color: Color,
    m: i32,
    n: i32,
    started: bool,

}

#[derive(Error, Debug)]
pub enum GFError {
    #[error("invalid GF ID")]
    InvalidGFID,
    #[error("yyy without preceding xxx")]
    YYYWithoutXXX,
    #[error("negative length for special")]
    NegativeLengthForSpecial,
    #[error("invalid opcode")]
    InvalidOpCode,
}

fn paint<T: Read>(d: i32, ctx: &mut Context<T>)  -> anyhow::Result<()> {
    match &ctx.current_character.color {
        Color::Black => {
            if d > 0 {
                ctx.current_character.char_data.bitmap.push(BlackLine { x: ctx.current_character.m, y: ctx.current_character.n, w: d });
            }
            ctx.current_character.color = Color::White;
        },
        Color::White => {
            ctx.current_character.color = Color::Black;
        }
    }
    ctx.current_character.m += d;
    Ok(())
}

fn boc<T: Read>(char_code: i32, ctx: &mut Context<T>) -> anyhow::Result<()> {
    let _ = read4(&mut ctx.input);
    let min_m = read4(&mut ctx.input)?;
    let max_m = read4(&mut ctx.input)?;
    let min_n = read4(&mut ctx.input)?;
    let max_n = read4(&mut ctx.input)?;
    boc_common(char_code, min_m, max_m, min_n, max_n, ctx)
}

fn boc_common<T: Read>(char_code: i32, min_m: i32, max_m: i32, min_n: i32, max_n: i32, ctx: &mut Context<T>)
    -> anyhow::Result<()> {
    ctx.current_character.started = true;
    ctx.current_character.m = min_m;
    ctx.current_character.n = max_n;
    ctx.current_character.color = Color::White;
    ctx.current_character.char_data = CharData {
        code: char_code,
        min_m: min_m,
        max_m: max_m,
        min_n: min_n,
        max_n: max_n,
        specials: vec![],
        bitmap: vec![]
    };
    Ok(())
}

fn boc1<T: Read>(char_code: i32, ctx: &mut Context<T>) -> anyhow::Result<()> {
    let del_m = read1(&mut ctx.input)?;
    let max_m = read1(&mut ctx.input)?;
    let del_n = read1(&mut ctx.input)?;
    let max_n = read1(&mut ctx.input)?;
    boc_common(char_code, max_m-del_m, max_m,max_n-del_n,max_n, ctx)
}

fn eoc<T: Read>(ctx: &mut Context<T>) -> anyhow::Result<()> {
    ctx.font_data.chars.push(ctx.current_character.char_data.clone());
    ctx.current_character.started=false;
    Ok(())
}

fn skip<T: Read>(rows: i32, ctx: &mut Context<T>) -> anyhow::Result<()> {
    ctx.current_character.color = Color::White;
    ctx.current_character.m = ctx.current_character.char_data.min_m;
    ctx.current_character.n -= rows + 1;
    Ok(())
}

fn new_row<T: Read>(indent: i32, ctx: &mut Context<T>) -> anyhow::Result<()> {
    ctx.current_character.color = Color::Black;
    ctx.current_character.m = ctx.current_character.char_data.min_m + indent;
    ctx.current_character.n -= 1;
    Ok(())
}

fn xxx<T: Read>(size: i32, ctx: &mut Context<T>) -> anyhow::Result<()> {
    if size < 0 {
        anyhow::bail!(GFError::NegativeLengthForSpecial)
    }
    let special = read_string(&mut ctx.input, size)?;
    ctx.specials.push(Special { special, numeric_params: vec![] });
    Ok(())
}

fn yyy<T: Read>(val: i32, ctx: &mut Context<T>) -> anyhow::Result<()> {
    ctx.specials.last_mut().ok_or(GFError::YYYWithoutXXX )?.numeric_params.push(val);
    Ok(())
}

fn pre<T: Read>(id: i32, ctx: &mut Context<T>) -> anyhow::Result<()> {
    if id != 131 {
        anyhow::bail!(GFError::InvalidGFID)
    }
    let size = read1(&mut ctx.input)?;
    ctx.font_data.title = read_string(&mut ctx.input, size)?;
    Ok(())
}

// We can safely ignore the bulk of the postamble for our purposes and just verify that
// we've reached it to flag the data as being completely read.
fn post<T: Read>(_: i32, ctx: &mut Context<T>) -> anyhow::Result<()> {
    ctx.finished = true;
    // TODO Need to read whole-font data from postamble still
    Ok(())
}


 pub fn gfreader<T: Read>(input: &mut T) -> anyhow::Result<FontData> {
    let mut ctx = Context {
        finished: false,
        font_data: FontData { title: String::new(), chars: vec![] },
        current_character: CharacterContext {
            char_data: CharData {
                code: 0,
                min_m: 0,
                max_m: 0,
                min_n: 0,
                max_n: 0,
                specials: vec![],
                bitmap: vec![]
            },
            color: Color::Black,
            m: 0,
            n: 0,
            started: false
        },
        specials: vec![],
        input: input
    };


     while !ctx.finished {
         let opcode = read1(&mut ctx.input)? as u8;
         match opcode {
             0..=63 => paint(opcode as i32, &mut ctx)?,
             64 => {
                 let d = read1(&mut ctx.input)?;
                 paint(d, &mut ctx)?
             },
             65 => {
                 let d = read2(&mut ctx.input)?;
                 paint(d, &mut ctx)?
             },
             66 => {
                 let d = read3(&mut ctx.input)?;
                 paint(d, &mut ctx)?
             },
             67 => {
                 let c = read4(&mut ctx.input)?;
                 boc(c, &mut ctx)?
             },
             68 => {
                 let c = read1(&mut ctx.input)?;
                 boc1(c, &mut ctx)?
             }
             69 => eoc(&mut ctx)?,
             70 => skip(0, &mut ctx)?,
             71 => {
                 let rows = read1(&mut ctx.input)?;
                 skip(rows, &mut ctx)?;
             },
             72 => {
                 let rows = read2(&mut ctx.input)?;
                 skip(rows, &mut ctx)?;
             },
             73 => {
                 let rows = read3(&mut ctx.input)?;
                 skip(rows, &mut ctx)?;
             },
             74..=238 => new_row(opcode as i32 - 74, &mut ctx)?,
             239 => {
                 let k = read1(&mut ctx.input)?;
                 xxx(k, &mut ctx)?;
             },
             240 => {
                 let k = read2(&mut ctx.input)?;
                 xxx(k, &mut ctx)?;
             },
             241 => {
                 let k = read3(&mut ctx.input)?;
                 xxx(k, &mut ctx)?;
             },
             242 => {
                 let k = read4(&mut ctx.input)?;
                 xxx(k, &mut ctx)?;
             },
             243 => {
                 let y = read4(&mut ctx.input)?;
                 yyy(y, &mut ctx)?;
             },
             244 => {},
             245..=246 => {
                 // We should never reach a char_loc opcode since we stop at the start of
                 // the postamble
                 anyhow::bail!(GFError::InvalidOpCode);
             },
             247 => {
                 let id = read1(&mut ctx.input)?;
                 pre(id, &mut ctx)?;
             },
             248 => {
                 let p = read4(&mut ctx.input)?;
                 post(p, &mut ctx)?;
             },
             249..=255 => {
                 // 249 is valid but we should never ee it; 250–255 are undefined
                 anyhow::bail!(GFError::InvalidOpCode);
             },
         }

     }

     Ok(ctx.font_data)
 }

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn make_context<T: Read>(input: T) -> Context<T> {
        Context {
            finished: false,
            font_data: FontData { title: String::new(), chars: vec![] },
            current_character: CharacterContext {
                char_data: CharData {
                    code: 0,
                    min_m: 0,
                    max_m: 0,
                    min_n: 0,
                    max_n: 0,
                    specials: vec![],
                    bitmap: vec![]
                },
                color: Color::Black,
                m: 0,
                n: 0,
                started: false
            },
            specials: vec![],
            input: input
        }
    }

    #[test]
    fn can_paint() -> anyhow::Result<()> {
        let mut context = make_context(Cursor::new(""));
        paint(12, &mut context)?;
        assert_eq!(context.current_character.char_data.bitmap[0], BlackLine {x: 0, y: 0, w: 12});
        assert_eq!(context.current_character.m, 12);
        assert_eq!(context.current_character.color, Color::White);
        paint(12, &mut context)?;
        assert_eq!(context.current_character.char_data.bitmap.len(), 1);
        assert_eq!(context.current_character.m, 24);
        assert_eq!(context.current_character.color, Color::Black);
        paint(0, &mut context)?;
        assert_eq!(context.current_character.char_data.bitmap.len(), 1);
        assert_eq!(context.current_character.m, 24);
        assert_eq!(context.current_character.color, Color::White);
        Ok(())
    }

    #[test]
    fn can_boc() -> anyhow::Result<()> {
        let mut context = make_context(Cursor::new([0xff, 0xff, 0xff, 0xff,
        0x00,0x00,0x01,0x00,
        0x00,0x00,0x02,0x00,
        0x00,0x00,0x03,0x00,
        0x00,0x00,0x04,0x00]
        ));
        assert_eq!(context.current_character.started, false);
        boc(65, &mut context)?;
        assert_eq!(context.current_character.char_data.code, 65);
        assert_eq!(context.current_character.char_data.min_m, 0x100);
        assert_eq!(context.current_character.char_data.max_m, 0x200);
        assert_eq!(context.current_character.char_data.min_n, 0x300);
        assert_eq!(context.current_character.char_data.max_n, 0x400);
        assert_eq!(context.current_character.started, true);
        assert_eq!(context.current_character.m, 0x100);
        assert_eq!(context.current_character.n, 0x400);
        assert_eq!(context.current_character.color, Color::White);
        Ok(())
    }

    #[test]
    fn can_boc1() -> anyhow::Result<()> {
        let mut context = make_context(Cursor::new([0x05, 0x10, 0x1f, 0x3f]
        ));
        assert_eq!(context.current_character.started, false);
        boc1(65, &mut context)?;
        assert_eq!(context.current_character.char_data.code, 65);
        assert_eq!(context.current_character.char_data.min_m, 0x0b);
        assert_eq!(context.current_character.char_data.max_m, 0x10);
        assert_eq!(context.current_character.char_data.min_n, 0x20);
        assert_eq!(context.current_character.char_data.max_n, 0x3f);
        assert_eq!(context.current_character.started, true);
        assert_eq!(context.current_character.m, 0x0b);
        assert_eq!(context.current_character.n, 0x3f);
        assert_eq!(context.current_character.color, Color::White);
        Ok(())
    }

    #[test]
    fn can_eoc() -> anyhow::Result<()> {
        let mut context = make_context(Cursor::new([0x05, 0x10, 0x1f, 0x3f,
            0x05, 0x10, 0x1f, 0x3f]
        ));
        assert_eq!(context.current_character.started, false);
        boc1(65, &mut context)?;
        paint(0, &mut context)?;
        paint(32, &mut context)?;
        eoc(&mut context)?;
        assert_eq!(context.current_character.started, false);
        assert_eq!(context.font_data.chars.len(), 1);
        assert_eq!(context.font_data.chars[0].bitmap.len(), 1, "The bitmap didn't get into the copied character");
        boc1(66, &mut context)?;
        assert_eq!(context.font_data.chars[0].bitmap.len(), 1);
        assert_eq!(context.current_character.char_data.bitmap.len(), 0);
        Ok(())
    }

    #[test]
    fn can_skip() -> anyhow::Result<()> {
        let mut context = make_context(Cursor::new([]));
        context.current_character.m = 42;
        context.current_character.color = Color::Black;
        skip(3, &mut context)?;
        assert_eq!(context.current_character.n, -4);
        assert_eq!(context.current_character.m, 0);
        assert_eq!(context.current_character.color, Color::White);
        Ok(())
    }

    #[test]
    fn can_new_row() -> anyhow::Result<()> {
        let mut context = make_context(Cursor::new([]));
        context.current_character.m = 42;
        context.current_character.color = Color::White;
        new_row(3, &mut context)?;
        assert_eq!(context.current_character.n, -1);
        assert_eq!(context.current_character.m, 3);
        assert_eq!(context.current_character.color, Color::Black);
        Ok(())
    }

    #[test]
    fn can_xxx() -> anyhow::Result<()> {
        let mut context = make_context(Cursor::new("rule "));
        xxx(5, &mut context)?;
        assert_eq!(context.specials[0].special, "rule ");
        Ok(())
    }

    #[test]
    fn can_yyy() -> anyhow::Result<()> {
        let mut context = make_context(Cursor::new("rule "));
        xxx(5, &mut context)?;
        yyy(47, &mut context)?;
        yyy(21, &mut context)?;
        assert_eq!(context.specials[0].numeric_params[0], 47);
        assert_eq!(context.specials[0].numeric_params[1], 21);
        Ok(())
    }

    #[test]
    fn can_pre() -> anyhow::Result<()> {
        let mut context = make_context(Cursor::new("\x05Title"));
        pre(131, &mut context)?;
        assert_eq!(context.font_data.title, String::from("Title"));
        Ok(())
    }

    #[test]
    fn can_post() -> anyhow::Result<()> {
        let mut context = make_context(Cursor::new([]));
        post(47, &mut context)?;
        assert_eq!(context.finished, true);
        Ok(())
    }
}