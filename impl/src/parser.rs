use pest::Parser;

mod fmt {
    #[derive(pest_derive::Parser)]
    #[grammar = "fmt.pest"]
    pub struct Parser;
}

#[derive(Debug, PartialEq)]
pub enum ByteOrder {
    Native,
    LittleEndian,
    BigEndian,
}

#[derive(Debug, PartialEq)]
pub enum Modifier {
    Native,
    NativeStandard,
    LittleEndian,
    BigEndian,
}

impl Modifier {
    pub fn byte_order(&self) -> ByteOrder {
        match self {
            Modifier::Native => ByteOrder::Native,
            Modifier::NativeStandard => ByteOrder::Native,
            Modifier::LittleEndian => ByteOrder::LittleEndian,
            Modifier::BigEndian => ByteOrder::BigEndian,
        }
    }

    pub fn native_types(&self) -> bool {
        self == &Modifier::Native
    }
}

impl Default for Modifier {
    fn default() -> Self {
        Modifier::Native
    }
}

#[derive(Debug, PartialEq)]
pub enum FormatChar {
    Array,
    Bool,
    Char,
    Double,
    Float,
    Ident(String),
    Int,
    Long,
    LongLong,
    Pad,
    Short,
    Size,
    UChar,
    UInt,
    ULong,
    ULongLong,
    UShort,
    USize,
}

#[derive(Debug, PartialEq)]
pub struct FormatCode {
    pub repeat: Option<usize>,
    pub chr: FormatChar,
}

#[derive(Debug)]
pub struct Format {
    pub modifier: Option<Modifier>,
    pub codes: Vec<FormatCode>,
}

pub fn parse(inp: &str) -> Result<Format, pest::error::Error<fmt::Rule>> {
    let parse = fmt::Parser::parse(fmt::Rule::fmt, inp)?.next().unwrap();
    let mut modifier = None;
    let mut codes = Vec::new();
    for line in parse.into_inner() {
        match line.as_rule() {
            fmt::Rule::modifier => {
                modifier = Some(match line.as_str() {
                    "@" => Modifier::Native,
                    "=" => Modifier::NativeStandard,
                    "<" => Modifier::LittleEndian,
                    ">" => Modifier::BigEndian,
                    "!" => Modifier::BigEndian,
                    _ => unreachable!(),
                });
            }
            fmt::Rule::code => {
                let mut r = line.into_inner();
                let repeat = r.next().unwrap().as_str().parse().ok();
                let r = r.next().unwrap();
                let chr = match r.as_rule() {
                    fmt::Rule::char => match r.as_str() {
                        "?" => FormatChar::Bool,
                        "B" => FormatChar::UChar,
                        "H" => FormatChar::UShort,
                        "I" => FormatChar::UInt,
                        "L" => FormatChar::ULong,
                        "N" => FormatChar::USize,
                        "Q" => FormatChar::ULongLong,
                        "b" => FormatChar::Char,
                        "d" => FormatChar::Double,
                        "f" => FormatChar::Float,
                        "h" => FormatChar::Short,
                        "i" => FormatChar::Int,
                        "l" => FormatChar::Long,
                        "n" => FormatChar::Size,
                        "q" => FormatChar::LongLong,
                        "s" => FormatChar::Array,
                        "x" => FormatChar::Pad,
                        _ => unreachable!(),
                    },
                    fmt::Rule::ident => FormatChar::Ident(r.as_str().trim_matches('`').to_owned()),
                    _ => unreachable!(),
                };
                codes.push(FormatCode { repeat, chr })
            }
            fmt::Rule::EOI => {}
            _ => unreachable!(),
        }
    }
    Ok(Format { modifier, codes })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let p = parse("2i?").unwrap();
        assert!(p.modifier.is_none());
        assert_eq!(p.codes.len(), 2);
        assert!(
            p.codes[0]
                == FormatCode {
                    repeat: Some(2),
                    chr: FormatChar::Int
                }
        );
        assert!(
            p.codes[1]
                == FormatCode {
                    repeat: None,
                    chr: FormatChar::Bool
                }
        );

        let p = parse("@2`foo`").unwrap();
        assert_eq!(p.modifier, Some(Modifier::Native));
        assert_eq!(p.codes.len(), 1);
        assert!(
            p.codes[0]
                == FormatCode {
                    repeat: Some(2),
                    chr: FormatChar::Ident("foo".to_owned())
                }
        );

        let p = parse("=?").unwrap();
        assert_eq!(p.modifier, Some(Modifier::NativeStandard));
        assert_eq!(p.codes.len(), 1);
        assert!(
            p.codes[0]
                == FormatCode {
                    repeat: None,
                    chr: FormatChar::Bool
                }
        );

        let p = parse("! I").unwrap();
        assert_eq!(p.modifier, Some(Modifier::BigEndian));
        assert_eq!(p.codes.len(), 1);
        assert!(
            p.codes[0]
                == FormatCode {
                    repeat: None,
                    chr: FormatChar::UInt
                }
        );
    }

    #[test]
    fn complex() {
        let p = parse("@3b3b `Bar` 18d12h 6i6l6f 3d32?0`Foo`").unwrap();
        assert_eq!(p.modifier, Some(Modifier::Native));
        assert_eq!(
            p.codes.last(),
            Some(&FormatCode {
                repeat: Some(0),
                chr: FormatChar::Ident("Foo".to_owned())
            })
        );
    }

    #[test]
    fn invalid() {
        assert!(parse("!vd").is_err());
        assert!(parse("@3 b").is_err());
        assert!(parse("`").is_err());
    }
}
