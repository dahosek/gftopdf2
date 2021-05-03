use std::io::Read;
use super::file_reader::*;
use thiserror::Error;

pub struct FontData {
    pub title: String,
    pub chars: Vec<CharData>,
    pub design_size: f64,
    pub hppp: f64,
    pub vppp: f64,
    pub min_m: i32,
    pub max_m: i32,
    pub min_n: i32,
    pub max_n: i32,
    
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

struct Context {
    finished: bool,
    font_data: FontData,
    current_character: CharacterContext,
    specials: Vec<Special>,
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
pub enum GfError {
    #[error("invalid GF ID")]
    InvalidGfId,
    #[error("yyy without preceding xxx")]
    YyyWithoutXxx,
    #[error("invalid opcode")]
    InvalidOpCode,
}

impl Context {
    fn paint(&mut self, d: i32) {
        match &self.current_character.color {
            Color::Black => {
                if d > 0 {
                    self.current_character.char_data.bitmap.push(BlackLine { x: self.current_character.m, y: self.current_character.n, w: d });
                }
                self.current_character.color = Color::White;
            },
            Color::White => {
                self.current_character.color = Color::Black;
            }
        }
        self.current_character.m += d;
    }

    fn boc(&mut self, char_code: i32, _p: i32, min_m: i32, max_m: i32, min_n: i32, max_n: i32) {
        self.current_character.started = true;
        self.current_character.m = min_m;
        self.current_character.n = max_n;
        self.current_character.color = Color::White;
        self.current_character.char_data = CharData {
            code: char_code,
            min_m,
            max_m,
            min_n,
            max_n,
            specials: vec![],
            bitmap: vec![]
        };
    }

    fn eoc(& mut self) {
        self.font_data.chars.push(self.current_character.char_data.clone());
        self.current_character.started = false;
    }

    fn skip(&mut self, rows: i32) {
        self.current_character.color = Color::White;
        self.current_character.m = self.current_character.char_data.min_m;
        self.current_character.n -= rows + 1;
    }

    fn new_row(&mut self, indent: i32) {
        self.current_character.color = Color::Black;
        self.current_character.m = self.current_character.char_data.min_m + indent;
        self.current_character.n -= 1;
    }

    fn xxx(&mut self, special: String) {
        self.specials.push(Special { special, numeric_params: vec![] });
    }

    fn yyy(&mut self, val: i32) -> anyhow::Result<()> {
        self.specials.last_mut().ok_or(GfError::YyyWithoutXxx)?.numeric_params.push(val);
        Ok(())
    }

    fn pre(&mut self, id: i32, title: String) -> anyhow::Result<()> {
        if id != 131 {
            anyhow::bail!(GfError::InvalidGfId)
        }
        self.font_data.title = title;
        Ok(())
    }

    // We can safely ignore the bulk of the postamble for our purposes and just verify that
// we've reached it to flag the data as being completely read.
    fn post(&mut self, _p: i32, ds: f64, _cs: i32, hppp: f64, vppp: f64, min_m: i32, max_m: i32, min_n: i32, max_n: i32) {
        self.finished = true;
        self.font_data.design_size = ds;
        self.font_data.hppp = hppp;
        self.font_data.vppp = vppp;
        self.font_data.max_m = max_m;
        self.font_data.min_m = min_m;
        self.font_data.max_n = max_n;
        self.font_data.min_n = min_n;
    }
}

 pub fn gfreader<T : Read>(input: &mut T) -> anyhow::Result<FontData> {
    let mut ctx = Context {
        finished: false,
        font_data: FontData { title: String::new(), chars: vec![], design_size: 0.0, hppp: 0.0, vppp: 0.0, min_m: 0, max_m: 0, min_n: 0, max_n: 0 },
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
     };


     while !ctx.finished {
         let opcode = input.read1()? as u8;
         match opcode {
             0..=63 => ctx.paint(opcode as i32),
             64 => ctx.paint(input.read1()?),
             65 => ctx.paint(input.read2()?),
             66 => ctx.paint(input.read3()?),
             67 => ctx.boc(input.read4()?, input.read4()?, input.read4()?, input.read4()?, input.read4()?, input.read4()?),
             68 => {
                 let char_code = input.read1()?;
                 let del_m = input.read1()?;
                 let max_m = input.read1()?;
                 let del_n = input.read1()?;
                 let max_n = input.read1()?;
                 ctx.boc(char_code, 0, max_m - del_m, max_m, max_n - del_n, max_n);
             }
             69 => ctx.eoc(),
             70 => ctx.skip(0),
             71 => ctx.skip(input.read1()?),
             72 => ctx.skip(input.read2()?),
             73 => ctx.skip(input.read3()?),
             74..=238 => ctx.new_row(opcode as i32 - 74),
             239 => ctx.xxx(input.read_string1()?),
             240 => ctx.xxx(input.read_string2()?),
             241 => ctx.xxx(input.read_string3()?),
             242 => ctx.xxx(input.read_string4()?),
             243 => ctx.yyy(input.read4()?)?,
             244 => {},
             245..=246 => {
                 // We should never reach a char_loc opcode since we stop at the start of
                 // the postamble
                 anyhow::bail!(GfError::InvalidOpCode);
             },
             247 => {
                 ctx.pre(input.read1()?, input.read_string1()?)?;
             },
             248 => {
                 ctx.post(input.read4()?,
                          input.read_fix_word()?,
                          input.read4()?,
                          input.read_fix_word()?,
                          input.read_fix_word()?,
                          input.read4()?,
                          input.read4()?,
                          input.read4()?,
                          input.read4()?);
             },
             249..=255 => {
                 // 249 is valid but we should never ee it; 250–255 are undefined
                 anyhow::bail!(GfError::InvalidOpCode);
             },
         }

     }

     Ok(ctx.font_data)
 }

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context() -> Context {
        Context {
            finished: false,
            font_data: FontData { title: String::new(), chars: vec![], design_size: 0.0, hppp: 0.0, vppp: 0.0, min_m: 0, max_m: 0, min_n: 0, max_n: 0 },
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
        }
    }

    #[test]
    fn can_paint() -> anyhow::Result<()> {
        let mut context = make_context();
        context.paint(12);
        assert_eq!(context.current_character.char_data.bitmap[0], BlackLine {x: 0, y: 0, w: 12});
        assert_eq!(context.current_character.m, 12);
        assert_eq!(context.current_character.color, Color::White);
        context.paint(12);
        assert_eq!(context.current_character.char_data.bitmap.len(), 1);
        assert_eq!(context.current_character.m, 24);
        assert_eq!(context.current_character.color, Color::Black);
        context.paint(0);
        assert_eq!(context.current_character.char_data.bitmap.len(), 1);
        assert_eq!(context.current_character.m, 24);
        assert_eq!(context.current_character.color, Color::White);
        Ok(())
    }

    #[test]
    fn can_boc() -> anyhow::Result<()> {
        let mut context = make_context();
        assert_eq!(context.current_character.started, false);
        context.boc(65, -1, 0x100, 0x200, 0x300, 0x400);
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
    fn can_eoc() -> anyhow::Result<()> {
        let mut context = make_context();
        assert_eq!(context.current_character.started, false);
        context.boc(65, -1, 0x100, 0x200, 0x300, 0x400);
        context.paint(0);
        context.paint(32);
        context.eoc();
        assert_eq!(context.current_character.started, false);
        assert_eq!(context.font_data.chars.len(), 1);
        assert_eq!(context.font_data.chars[0].bitmap.len(), 1, "The bitmap didn't get into the copied character");
        context.boc(66, -1, 0x100, 0x200, 0x300, 0x400);
        assert_eq!(context.font_data.chars[0].bitmap.len(), 1);
        assert_eq!(context.current_character.char_data.bitmap.len(), 0);
        Ok(())
    }

    #[test]
    fn can_skip() -> anyhow::Result<()> {
        let mut context = make_context();
        context.current_character.m = 42;
        context.current_character.color = Color::Black;
        context.skip(3);
        assert_eq!(context.current_character.n, -4);
        assert_eq!(context.current_character.m, 0);
        assert_eq!(context.current_character.color, Color::White);
        Ok(())
    }

    #[test]
    fn can_new_row() -> anyhow::Result<()> {
        let mut context = make_context();
        context.current_character.m = 42;
        context.current_character.color = Color::White;
        context.new_row(3);
        assert_eq!(context.current_character.n, -1);
        assert_eq!(context.current_character.m, 3);
        assert_eq!(context.current_character.color, Color::Black);
        Ok(())
    }

    #[test]
    fn can_xxx() -> anyhow::Result<()> {
        let mut context = make_context();
        context.xxx(String::from("rule "));
        assert_eq!(context.specials[0].special, "rule ");
        Ok(())
    }

    #[test]
    fn can_yyy() -> anyhow::Result<()> {
        let mut context = make_context();
        context.xxx(String::from("rule "));
        context.yyy(47)?;
        context.yyy(21)?;
        assert_eq!(context.specials[0].numeric_params[0], 47);
        assert_eq!(context.specials[0].numeric_params[1], 21);
        Ok(())
    }

    #[test]
    fn can_pre() -> anyhow::Result<()> {
        let mut context = make_context();
        context.pre(131, String::from("Title"))?;
        assert_eq!(context.font_data.title, String::from("Title"));
        Ok(())
    }

    #[test]
    fn can_post() -> anyhow::Result<()> {
        let mut context = make_context();
        context.post(47, 10.0, 0, 24.0, 24.0, 0, 1, 2, 3);
        assert_eq!(context.finished, true);
        Ok(())
    }
}