use crate::errors::{ErrorKind, Result};
use crate::one::property::color_ref::ColorRef;
use crate::one::property::note_tag::ActionItemType;
use crate::one::property::{simple, PropertyType};
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;

#[derive(Debug)]
pub(crate) struct Data {
    pub(crate) label: String,
    pub(crate) status: NoteTagPropertyStatus,
    pub(crate) shape: NoteTagShape,
    pub(crate) highlight_color: Option<ColorRef>,
    pub(crate) text_color: Option<ColorRef>,
    pub(crate) action_item_type: ActionItemType,
}

pub(crate) fn parse(object: &Object) -> Result<Data> {
    if object.id() != PropertySetId::NoteTagSharedDefinitionContainer.as_jcid() {
        return Err(ErrorKind::MalformedOneNoteFileData(
            format!("unexpected object type: 0x{:X}", object.id().0).into(),
        )
        .into());
    }

    let label = simple::parse_string(PropertyType::NoteTagLabel, object)?.ok_or_else(|| {
        ErrorKind::MalformedOneNoteFileData("note tag container has no label".into())
    })?;
    let status = NoteTagPropertyStatus::parse(object)?.ok_or_else(|| {
        ErrorKind::MalformedOneNoteFileData("note tag container has no status".into())
    })?;
    let shape = simple::parse_u16(PropertyType::NoteTagShape, object)?
        .map(NoteTagShape::parse)
        .ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData("note tag container has no shape".into())
        })?;
    let highlight_color = ColorRef::parse(PropertyType::NoteTagHighlightColor, object)?;
    let text_color = ColorRef::parse(PropertyType::NoteTagTextColor, object)?;
    let action_item_type = ActionItemType::parse(object)?.ok_or_else(|| {
        ErrorKind::MalformedOneNoteFileData("note tag container has no action item type".into())
    })?;

    let data = Data {
        label,
        status,
        shape,
        highlight_color,
        text_color,
        action_item_type,
    };

    Ok(data)
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct NoteTagPropertyStatus {
    has_label: bool,
    has_font_color: bool,
    has_highlight_color: bool,
    has_icon: bool,
    due_today: bool,
    due_tomorrow: bool,
    due_this_week: bool,
    due_next_week: bool,
    due_later: bool,
    due_custom: bool,
}

impl NoteTagPropertyStatus {
    pub fn has_label(&self) -> bool {
        self.has_label
    }

    pub fn has_font_color(&self) -> bool {
        self.has_font_color
    }

    pub fn has_highlight_color(&self) -> bool {
        self.has_highlight_color
    }

    pub fn has_icon(&self) -> bool {
        self.has_icon
    }

    pub fn due_today(&self) -> bool {
        self.due_today
    }

    pub fn due_tomorrow(&self) -> bool {
        self.due_tomorrow
    }

    pub fn due_this_week(&self) -> bool {
        self.due_this_week
    }

    pub fn due_next_week(&self) -> bool {
        self.due_next_week
    }

    pub fn due_later(&self) -> bool {
        self.due_later
    }

    pub fn due_custom(&self) -> bool {
        self.due_custom
    }
}

impl NoteTagPropertyStatus {
    fn parse(object: &Object) -> Result<Option<NoteTagPropertyStatus>> {
        let status = object
            .props()
            .get(PropertyType::NoteTagPropertyStatus)
            .map(|value| {
                value.to_u32().ok_or_else(|| {
                    ErrorKind::MalformedOneNoteFileData(
                        "note tag property status is not a u32".into(),
                    )
                })
            })
            .transpose()?
            .map(|value| NoteTagPropertyStatus {
                has_label: value & 0x1 != 0,
                has_font_color: (value >> 1) & 0x1 != 0,
                has_highlight_color: (value >> 2) & 0x1 != 0,
                has_icon: (value >> 3) & 0x1 != 0,
                due_today: (value >> 6) & 0x1 != 0,
                due_tomorrow: (value >> 7) & 0x1 != 0,
                due_this_week: (value >> 8) & 0x1 != 0,
                due_next_week: (value >> 9) & 0x1 != 0,
                due_later: (value >> 10) & 0x1 != 0,
                due_custom: (value >> 11) & 0x1 != 0,
            });

        Ok(status)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[allow(dead_code)]
pub enum NoteTagShape {
    NoIcon,
    GreenCheckBox,
    YellowCheckBox,
    BlueCheckBox,
    GreenStarCheckBox,
    YellowStarCheckBox,
    BlueStarCheckBox,
    GreenExclamationCheckBox,
    YellowExclamationCheckBox,
    BlueExclamationCheckBox,
    GreenRightArrowCheckBox,
    YellowRightArrowCheckBox,
    BlueRightArrowCheckBox,
    YellowStar,
    BlueFollowUpFlag,
    QuestionMark,
    BlueRightArrow,
    HighPriority,
    ContactInformation,
    Meeting,
    TimeSensitive,
    LightBulb,
    Pushpin,
    Home,
    CommentBubble,
    SmilingFace,
    AwardRibbon,
    YellowKey,
    BlueCheckBox1,
    BlueCircle1,
    BlueCheckBox2,
    BlueCircle2,
    BlueCheckBox3,
    BlueCircle3,
    BlueEightPointStar,
    BlueCheckMark,
    BlueCircle,
    BlueDownArrow,
    BlueLeftArrow,
    BlueSolidTarget,
    BlueStar,
    BlueSun,
    BlueTarget,
    BlueTriangle,
    BlueUmbrella,
    BlueUpArrow,
    BlueXWithDots,
    BlueX,
    GreenCheckBox1,
    GreenCircle1,
    GreenCheckBox2,
    GreenCircle2,
    GreenCheckBox3,
    GreenCircle3,
    GreenEightPointStar,
    GreenCheckMark,
    GreenCircle,
    GreenDownArrow,
    GreenLeftArrow,
    GreenRightArrow,
    GreenSolidArrow,
    GreenStar,
    GreenSun,
    GreenTarget,
    GreenTriangle,
    GreenUmbrella,
    GreenUpArrow,
    GreenXWithDots,
    GreenX,
    YellowCheckBox1,
    YellowCircle1,
    YellowCheckBox2,
    YellowCircle2,
    YellowCheckBox3,
    YellowCircle3,
    YellowEightPointStar,
    YellowCheckMark,
    YellowCircle,
    YellowDownArrow,
    YellowLeftArrow,
    YellowRightArrow,
    YellowSolidTarget,
    YellowSun,
    YellowTarget,
    YellowTriangle,
    YellowUmbrella,
    YellowUpArrow,
    YellowXWithDots,
    YellowX,
    FollowUpTodayFlag,
    FollowUpTomorrowFlag,
    FollowUpThisWeekFlag,
    FollowUpNextWeekFlag,
    NoFollowUpDateFlag,
    BluePersonCheckBox,
    YellowPersonCheckBox,
    GreenPersonCheckBox,
    BlueFlagCheckBox,
    RedFlagCheckBox,
    GreenFlagCheckBox,
    RedSquare,
    YellowSquare,
    BlueSquare,
    GreenSquare,
    OrangeSquare,
    PinkSquare,
    EMailMessage,
    ClosedEnvelope,
    OpenEnvelope,
    MobilePhone,
    TelephoneWithClock,
    QuestionBalloon,
    PaperClip,
    FrowningFace,
    InstantMessagingContactPerson,
    PersonWithExclamationMark,
    TwoPeople,
    ReminderBell,
    Contact,
    RoseOnAStem,
    CalendarDateWithClock,
    MusicalNote,
    MovieClip,
    QuotationMark,
    Globe,
    HyperlinkGlobe,
    Laptop,
    Plane,
    Car,
    Binoculars,
    PresentationSlide,
    Padlock,
    OpenBook,
    NotebookWithClock,
    BlankPaperWithLines,
    Research,
    Pen,
    DollarSign,
    CoinsWithAWindowBackdrop,
    ScheduledTask,
    LightningBolt,
    Cloud,
    Heart,
    Sunflower,
}

impl NoteTagShape {
    pub(crate) fn parse(value: u16) -> NoteTagShape {
        match value {
            0 => NoteTagShape::NoIcon,
            1 => NoteTagShape::GreenCheckBox,
            2 => NoteTagShape::YellowCheckBox,
            3 => NoteTagShape::BlueCheckBox,
            4 => NoteTagShape::GreenStarCheckBox,
            5 => NoteTagShape::YellowStarCheckBox,
            6 => NoteTagShape::BlueStarCheckBox,
            7 => NoteTagShape::GreenExclamationCheckBox,
            8 => NoteTagShape::YellowExclamationCheckBox,
            9 => NoteTagShape::BlueExclamationCheckBox,
            10 => NoteTagShape::GreenRightArrowCheckBox,
            11 => NoteTagShape::YellowRightArrowCheckBox,
            12 => NoteTagShape::BlueRightArrowCheckBox,
            13 => NoteTagShape::YellowStar,
            14 => NoteTagShape::BlueFollowUpFlag,
            15 => NoteTagShape::QuestionMark,
            16 => NoteTagShape::BlueRightArrow,
            17 => NoteTagShape::HighPriority,
            18 => NoteTagShape::ContactInformation,
            19 => NoteTagShape::Meeting,
            20 => NoteTagShape::TimeSensitive,
            21 => NoteTagShape::LightBulb,
            22 => NoteTagShape::Pushpin,
            23 => NoteTagShape::Home,
            24 => NoteTagShape::CommentBubble,
            25 => NoteTagShape::SmilingFace,
            26 => NoteTagShape::AwardRibbon,
            27 => NoteTagShape::YellowKey,
            28 => NoteTagShape::BlueCheckBox1,
            29 => NoteTagShape::BlueCircle1,
            30 => NoteTagShape::BlueCheckBox2,
            31 => NoteTagShape::BlueCircle2,
            32 => NoteTagShape::BlueCheckBox3,
            33 => NoteTagShape::BlueCircle3,
            34 => NoteTagShape::BlueEightPointStar,
            35 => NoteTagShape::BlueCheckMark,
            36 => NoteTagShape::BlueCircle,
            37 => NoteTagShape::BlueDownArrow,
            38 => NoteTagShape::BlueLeftArrow,
            39 => NoteTagShape::BlueSolidTarget,
            40 => NoteTagShape::BlueStar,
            41 => NoteTagShape::BlueSun,
            42 => NoteTagShape::BlueTarget,
            43 => NoteTagShape::BlueTriangle,
            44 => NoteTagShape::BlueUmbrella,
            45 => NoteTagShape::BlueUpArrow,
            46 => NoteTagShape::BlueXWithDots,
            47 => NoteTagShape::BlueX,
            48 => NoteTagShape::GreenCheckBox1,
            49 => NoteTagShape::GreenCircle1,
            50 => NoteTagShape::GreenCheckBox2,
            51 => NoteTagShape::GreenCircle2,
            52 => NoteTagShape::GreenCheckBox3,
            53 => NoteTagShape::GreenCircle3,
            54 => NoteTagShape::GreenEightPointStar,
            55 => NoteTagShape::GreenCheckMark,
            56 => NoteTagShape::GreenCircle,
            57 => NoteTagShape::GreenDownArrow,
            58 => NoteTagShape::GreenLeftArrow,
            59 => NoteTagShape::GreenRightArrow,
            60 => NoteTagShape::GreenSolidArrow,
            61 => NoteTagShape::GreenStar,
            62 => NoteTagShape::GreenSun,
            63 => NoteTagShape::GreenTarget,
            64 => NoteTagShape::GreenTriangle,
            65 => NoteTagShape::GreenUmbrella,
            66 => NoteTagShape::GreenUpArrow,
            67 => NoteTagShape::GreenXWithDots,
            68 => NoteTagShape::GreenX,
            69 => NoteTagShape::YellowCheckBox1,
            70 => NoteTagShape::YellowCircle1,
            71 => NoteTagShape::YellowCheckBox2,
            72 => NoteTagShape::YellowCircle2,
            73 => NoteTagShape::YellowCheckBox3,
            74 => NoteTagShape::YellowCircle3,
            75 => NoteTagShape::YellowEightPointStar,
            76 => NoteTagShape::YellowCheckMark,
            77 => NoteTagShape::YellowCircle,
            78 => NoteTagShape::YellowDownArrow,
            79 => NoteTagShape::YellowLeftArrow,
            80 => NoteTagShape::YellowRightArrow,
            81 => NoteTagShape::YellowSolidTarget,
            82 => NoteTagShape::YellowSun,
            83 => NoteTagShape::YellowTarget,
            84 => NoteTagShape::YellowTriangle,
            85 => NoteTagShape::YellowUmbrella,
            86 => NoteTagShape::YellowUpArrow,
            87 => NoteTagShape::YellowXWithDots,
            88 => NoteTagShape::YellowX,
            89 => NoteTagShape::FollowUpTodayFlag,
            90 => NoteTagShape::FollowUpTomorrowFlag,
            91 => NoteTagShape::FollowUpThisWeekFlag,
            92 => NoteTagShape::FollowUpNextWeekFlag,
            93 => NoteTagShape::NoFollowUpDateFlag,
            94 => NoteTagShape::BluePersonCheckBox,
            95 => NoteTagShape::YellowPersonCheckBox,
            96 => NoteTagShape::GreenPersonCheckBox,
            97 => NoteTagShape::BlueFlagCheckBox,
            98 => NoteTagShape::RedFlagCheckBox,
            99 => NoteTagShape::GreenFlagCheckBox,
            100 => NoteTagShape::RedSquare,
            101 => NoteTagShape::YellowSquare,
            102 => NoteTagShape::BlueSquare,
            103 => NoteTagShape::GreenSquare,
            104 => NoteTagShape::OrangeSquare,
            105 => NoteTagShape::PinkSquare,
            106 => NoteTagShape::EMailMessage,
            107 => NoteTagShape::ClosedEnvelope,
            108 => NoteTagShape::OpenEnvelope,
            109 => NoteTagShape::MobilePhone,
            110 => NoteTagShape::TelephoneWithClock,
            111 => NoteTagShape::QuestionBalloon,
            112 => NoteTagShape::PaperClip,
            113 => NoteTagShape::FrowningFace,
            114 => NoteTagShape::InstantMessagingContactPerson,
            115 => NoteTagShape::PersonWithExclamationMark,
            116 => NoteTagShape::TwoPeople,
            117 => NoteTagShape::ReminderBell,
            118 => NoteTagShape::Contact,
            119 => NoteTagShape::RoseOnAStem,
            120 => NoteTagShape::CalendarDateWithClock,
            121 => NoteTagShape::MusicalNote,
            122 => NoteTagShape::MovieClip,
            123 => NoteTagShape::QuotationMark,
            124 => NoteTagShape::Globe,
            125 => NoteTagShape::HyperlinkGlobe,
            126 => NoteTagShape::Laptop,
            127 => NoteTagShape::Plane,
            128 => NoteTagShape::Car,
            129 => NoteTagShape::Binoculars,
            130 => NoteTagShape::PresentationSlide,
            131 => NoteTagShape::Padlock,
            132 => NoteTagShape::OpenBook,
            133 => NoteTagShape::NotebookWithClock,
            134 => NoteTagShape::BlankPaperWithLines,
            135 => NoteTagShape::Research,
            136 => NoteTagShape::Pen,
            137 => NoteTagShape::DollarSign,
            138 => NoteTagShape::CoinsWithAWindowBackdrop,
            139 => NoteTagShape::ScheduledTask,
            140 => NoteTagShape::LightningBolt,
            141 => NoteTagShape::Cloud,
            142 => NoteTagShape::Heart,
            143 => NoteTagShape::Sunflower,
            _ => panic!("invalid note tag shape: {}", value),
        }
    }

    pub fn is_checkable(&self) -> bool {
        matches!(
            self,
            NoteTagShape::GreenCheckBox
                | NoteTagShape::YellowCheckBox
                | NoteTagShape::BlueCheckBox
                | NoteTagShape::GreenStarCheckBox
                | NoteTagShape::YellowStarCheckBox
                | NoteTagShape::BlueStarCheckBox
                | NoteTagShape::GreenExclamationCheckBox
                | NoteTagShape::YellowExclamationCheckBox
                | NoteTagShape::BlueExclamationCheckBox
                | NoteTagShape::GreenRightArrowCheckBox
                | NoteTagShape::YellowRightArrowCheckBox
                | NoteTagShape::BlueRightArrowCheckBox
                | NoteTagShape::BlueCheckBox1
                | NoteTagShape::BlueCheckBox2
                | NoteTagShape::BlueCheckBox3
                | NoteTagShape::GreenCheckBox1
                | NoteTagShape::GreenCheckBox2
                | NoteTagShape::GreenCheckBox3
                | NoteTagShape::YellowCheckBox1
                | NoteTagShape::YellowCheckBox2
                | NoteTagShape::YellowCheckBox3
                | NoteTagShape::FollowUpTodayFlag
                | NoteTagShape::FollowUpTomorrowFlag
                | NoteTagShape::FollowUpThisWeekFlag
                | NoteTagShape::FollowUpNextWeekFlag
                | NoteTagShape::NoFollowUpDateFlag
                | NoteTagShape::BluePersonCheckBox
                | NoteTagShape::YellowPersonCheckBox
                | NoteTagShape::GreenPersonCheckBox
                | NoteTagShape::BlueFlagCheckBox
                | NoteTagShape::RedFlagCheckBox
                | NoteTagShape::GreenFlagCheckBox
        )
    }
}
