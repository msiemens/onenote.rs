use crate::one::property::PropertyType;
use crate::onestore::object::Object;

#[derive(Debug, Copy, Clone)]
pub(crate) enum Charset {
    Ansi,
    Default,
    Symbol,
    Mac,
    ShiftJis,
    Hangul,
    Johab,
    Gb2312,
    ChineseBig5,
    Greek,
    Turkish,
    Vietnamese,
    Hebrew,
    Arabic,
    Baltic,
    Russian,
    Thai,
    EastEurope,
    OEM,
}

impl Charset {
    pub(crate) fn parse(prop_type: PropertyType, object: &Object) -> Option<Charset> {
        object
            .props()
            .get(prop_type)
            .map(|value| value.to_u8().expect("charset is not a u8"))
            .map(|value| match value {
                0 => Charset::Ansi,
                1 => Charset::Default,
                2 => Charset::Symbol,
                77 => Charset::Mac,
                128 => Charset::ShiftJis,
                129 => Charset::Hangul,
                130 => Charset::Johab,
                134 => Charset::Gb2312,
                136 => Charset::ChineseBig5,
                161 => Charset::Greek,
                162 => Charset::Turkish,
                163 => Charset::Vietnamese,
                177 => Charset::Hebrew,
                178 => Charset::Arabic,
                186 => Charset::Baltic,
                204 => Charset::Russian,
                222 => Charset::Thai,
                238 => Charset::EastEurope,
                255 => Charset::OEM,
                _ => panic!("invalid charset: {}", value),
            })
    }
}
