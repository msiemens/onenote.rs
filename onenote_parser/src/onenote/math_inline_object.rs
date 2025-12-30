use crate::errors::ErrorKind;
use crate::one::property_set::math_inline_object::Data;
use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;

/// The type of a math inline object.
///
/// Note: These values are derived from `OBJECTTYPE` in Win32 `tom.h`.
#[derive(Copy, Clone, Debug, PartialEq, Primitive)]
#[repr(u32)]
#[derive(Default)]
pub enum MathObjectType {
    /// Not an inline function.
    #[default]
    SimpleText = 0,
    /// A math accent object.
    Accent = 10,
    /// Abstract box with properties.
    Box = 11,
    /// Encloses the argument in a rectangle.
    BoxedFormula = 12,
    /// Encloses the argument in brackets, parentheses, and so on.
    Brackets = 13,
    /// Encloses the argument in brackets, parentheses, and so on, and with separators.
    BracketsWithSeps = 14,
    /// Column of aligned equations.
    EquationArray = 15,
    /// Fraction.
    Fraction = 16,
    /// Function apply.
    FunctionApply = 17,
    /// Left subscript or superscript.
    LeftSubSup = 18,
    /// Second argument below the first.
    LowerLimit = 19,
    /// Matrix.
    Matrix = 20,
    /// General n-ary expression.
    Nary = 21,
    /// Internal use for no-build operators.
    OpChar = 22,
    /// Overscores argument.
    Overbar = 23,
    /// Special spacing.
    Phantom = 24,
    /// Square root, and so on.
    Radical = 25,
    /// Skewed and built-up linear fractions.
    SlashedFraction = 26,
    /// "Fraction" with no divide bar.
    Stack = 27,
    /// Stretch character horizontally over or under argument.
    StretchStack = 28,
    /// Subscript.
    Subscript = 29,
    /// Subscript and superscript combination.
    SubSup = 30,
    /// Superscript.
    Superscript = 31,
    /// Underscores the argument.
    Underbar = 32,
    /// Second argument above the first.
    UpperLimit = 33,

    /// Plain text (undocumented).
    PlainText = 0x90000000u32,
}

/// A math inline object.
///
/// The parameters and their interpretation seem to derive from [`ITextRange2::GetInlineObject`]
/// in Win32 `tom.h`.
///
/// [`ITextRange2::GetInlineObject`]: https://learn.microsoft.com/en-us/windows/win32/api/tom/nf-tom-itextrange2-getinlineobject
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct MathInlineObject {
    pub(crate) object_type: MathObjectType,
    pub(crate) arg_count: u32,
    pub(crate) column: Option<u8>,
    pub(crate) align: Option<u8>,
    pub(crate) char: Option<char>,
    pub(crate) char1: Option<char>,
    pub(crate) char2: Option<char>,
}

impl MathInlineObject {
    /// The type of the math inline object.
    pub fn object_type(&self) -> MathObjectType {
        self.object_type
    }

    /// The number of arguments in the math inline object.
    pub fn arg_count(&self) -> u32 {
        self.arg_count
    }

    /// The column in an equation array.
    pub fn column(&self) -> Option<u8> {
        self.column
    }

    /// The alignment in an equation array.
    pub fn align(&self) -> Option<u8> {
        self.align
    }

    /// The character in the math inline object.
    pub fn char(&self) -> Option<char> {
        self.char
    }

    /// The second character in the math inline object (typically a closing bracket).
    pub fn char1(&self) -> Option<char> {
        self.char1
    }

    /// The third character in the math inline object (typically a separator).
    pub fn char2(&self) -> Option<char> {
        self.char2
    }
}

pub(crate) fn parse_math_inline_object(data: Data) -> crate::errors::Result<MathInlineObject> {
    let object_type = MathObjectType::from_u32(data.object_type.unwrap_or(0)).ok_or_else(|| {
        ErrorKind::MalformedOneNoteFileData(
            format!("MathInlineObject has invalid type: {:?}", data.object_type).into(),
        )
    })?;

    Ok(MathInlineObject {
        object_type,
        arg_count: data.arg_count.unwrap_or(0),
        column: data.column,
        align: data.align,
        char: data.char,
        char1: data.char1,
        char2: data.char2,
    })
}
