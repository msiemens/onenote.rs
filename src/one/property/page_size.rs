use crate::one::property::PropertyType;
use crate::onestore::object::Object;

#[derive(Debug)]
pub(crate) enum PageSize {
    Auto,
    US,
    AnsiLetter,
    AnsiTabloid,
    USLegal,
    IsoA3,
    IsoA4,
    IsoA5,
    IsoA6,
    JisB4,
    JisB5,
    JisB6,
    JapanesePostcard,
    IndexCard,
    Billfold,
    Custom,
}

impl PageSize {
    pub(crate) fn parse(prop_type: PropertyType, object: &Object) -> Option<PageSize> {
        object
            .props()
            .get(prop_type)
            .map(|value| value.to_u8().expect("page size is not a u8"))
            .map(|value| match value {
                0 => PageSize::Auto,
                1 => PageSize::US,
                2 => PageSize::AnsiLetter,
                3 => PageSize::AnsiTabloid,
                4 => PageSize::USLegal,
                5 => PageSize::IsoA3,
                6 => PageSize::IsoA4,
                7 => PageSize::IsoA5,
                8 => PageSize::IsoA6,
                9 => PageSize::JisB4,
                10 => PageSize::JisB5,
                11 => PageSize::JisB6,
                12 => PageSize::JapanesePostcard,
                13 => PageSize::IndexCard,
                14 => PageSize::Billfold,
                15 => PageSize::Custom,
                _ => panic!("invalid page size: {}", value),
            })
    }
}

impl Default for PageSize {
    fn default() -> Self {
        PageSize::Auto
    }
}
