use crate::one::property::PropertyType;
use crate::onestore::object::Object;

#[derive(Debug, Copy, Clone)]
pub(crate) struct LayoutAlignment {
    alignment_horizontal: HorizontalAlignment,
    alignment_margin_horizontal: HorizontalAlignmentMargin,
    alignment_vertical: VerticalAlignment,
    alignment_margin_vertical: VerticalAlignmentMargin,
}

impl LayoutAlignment {
    pub(crate) fn parse(prop_type: PropertyType, object: &Object) -> Option<LayoutAlignment> {
        object
            .props()
            .get(prop_type)
            .map(|value| value.to_u32().expect("layout alignment is not a u32"))
            .and_then(|value| {
                if (value >> 31) & 0x1 != 0 {
                    return None;
                }

                let alignment_horizontal = HorizontalAlignment::parse(value & 0x7);
                let alignment_margin_horizontal =
                    HorizontalAlignmentMargin::parse((value >> 3) & 0x1);
                let alignment_vertical = VerticalAlignment::parse((value >> 16) & 0x1);
                let alignment_margin_vertical = VerticalAlignmentMargin::parse((value >> 19) & 0x1);

                Some(LayoutAlignment {
                    alignment_horizontal,
                    alignment_margin_horizontal,
                    alignment_vertical,
                    alignment_margin_vertical,
                })
            })
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum HorizontalAlignment {
    Unknown,
    Left,
    Center,
    Right,
    BiDiNormal,
    BiDiReverse,
}

impl HorizontalAlignment {
    pub(crate) fn parse(value: u32) -> HorizontalAlignment {
        match value {
            0 => HorizontalAlignment::Unknown,
            1 => HorizontalAlignment::Left,
            2 => HorizontalAlignment::Center,
            3 => HorizontalAlignment::Right,
            4 => HorizontalAlignment::BiDiNormal,
            5 => HorizontalAlignment::BiDiReverse,
            _ => panic!("invalid horizontal alignment: {}", value),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum HorizontalAlignmentMargin {
    Right,
    Left,
}

impl HorizontalAlignmentMargin {
    pub(crate) fn parse(value: u32) -> HorizontalAlignmentMargin {
        match value {
            0 => HorizontalAlignmentMargin::Right,
            1 => HorizontalAlignmentMargin::Left,
            _ => panic!("invalid horizontal alignment margin: {}", value),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum VerticalAlignment {
    Bottom,
    Top,
}

impl VerticalAlignment {
    pub(crate) fn parse(value: u32) -> VerticalAlignment {
        match value {
            0 => VerticalAlignment::Bottom,
            1 => VerticalAlignment::Top,
            _ => panic!("invalid vertical alignment: {}", value),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum VerticalAlignmentMargin {
    Bottom,
    Top,
}

impl VerticalAlignmentMargin {
    pub(crate) fn parse(value: u32) -> VerticalAlignmentMargin {
        match value {
            0 => VerticalAlignmentMargin::Bottom,
            1 => VerticalAlignmentMargin::Top,
            _ => panic!("invalid vertical alignment margin: {}", value),
        }
    }
}
