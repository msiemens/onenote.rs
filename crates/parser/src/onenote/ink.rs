use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::one::property_set::{
    ink_container, ink_data_node, ink_stroke_node, stroke_properties_node,
};
use crate::onenote::ParserContext;
use crate::onenote::ink_recognition::InkRecognizedWord;
use crate::onestore::ObjectSpace;

/// An ink object.
#[derive(Clone, Debug)]
pub struct Ink {
    pub(crate) content: InkContent,
    pub(crate) bounding_box: Option<InkBoundingBox>,

    pub(crate) offset_horizontal: Option<f32>,
    pub(crate) offset_vertical: Option<f32>,
}

/// The contents of an [`Ink`] object.
///
/// An ink container is either a leaf with concrete strokes or an intermediate
/// group of nested ink objects.
#[derive(Clone, Debug)]
pub enum InkContent {
    /// A group of nested ink objects.
    InkGroup(Vec<Ink>),
    /// A leaf set of ink strokes.
    Strokes(Vec<InkStroke>),
}

impl Ink {
    /// The ink strokes contained in this ink object.
    ///
    /// Empty for group containers; see [`Ink::child_groups`] for nested ink.
    pub fn ink_strokes(&self) -> &[InkStroke] {
        match &self.content {
            InkContent::InkGroup(_) => &[],
            InkContent::Strokes(strokes) => strokes,
        }
    }

    /// The nested ink groups contained in this ink object.
    ///
    /// Empty for leaf containers; see [`Ink::ink_strokes`] for stroke data.
    pub fn child_groups(&self) -> &[Ink] {
        match &self.content {
            InkContent::InkGroup(children) => children,
            InkContent::Strokes(_) => &[],
        }
    }

    /// The ink object's bounding box.
    pub fn bounding_box(&self) -> Option<InkBoundingBox> {
        self.bounding_box
    }

    /// The horizontal offset from the page origin in half-inch increments.
    ///
    /// See [\[MS-ONE\] 2.3.18].
    ///
    /// [\[MS-ONE\] 2.3.18]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/5fb9e84a-c9e9-4537-ab14-e5512f24669a
    pub fn offset_horizontal(&self) -> Option<f32> {
        self.offset_horizontal
    }

    /// The vertical offset from the page origin in half-inch increments.
    ///
    /// See [\[MS-ONE\] 2.3.19].
    ///
    /// [\[MS-ONE\] 2.3.19]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/5c4992ba-1db5-43e9-83dd-7299c562104d
    pub fn offset_vertical(&self) -> Option<f32> {
        self.offset_vertical
    }
}

/// An ink stroke.
#[derive(Clone, Debug)]
pub struct InkStroke {
    pub(crate) path: Vec<InkPoint>,
    pub(crate) pen_tip: Option<u8>,
    pub(crate) transparency: Option<u8>,
    pub(crate) height: f32,
    pub(crate) width: f32,
    pub(crate) color: Option<u32>,
    pub(crate) recognized_word: Option<InkRecognizedWord>,
}

impl InkStroke {
    /// The ink stroke's path.
    ///
    /// The path is delta-encoded: the first point holds the absolute start
    /// coordinates and every following point holds its offset from the
    /// previous point. This is the same model as an SVG path made of an `M`
    /// command followed by `l` commands. Use [`InkStroke::absolute_path`] to
    /// get absolute coordinates instead.
    ///
    /// Values are in ink units (see [`InkPoint`]).
    pub fn path(&self) -> &[InkPoint] {
        &self.path
    }

    /// The stroke's path as absolute coordinates.
    ///
    /// This accumulates the offsets returned by [`InkStroke::path`], so the
    /// first point is the stroke's start and every following point is its
    /// absolute position. Values are in ink units (see [`InkPoint`]).
    pub fn absolute_path(&self) -> Vec<InkPoint> {
        let mut absolute = Vec::with_capacity(self.path.len());
        let mut x = 0.0_f32;
        let mut y = 0.0_f32;
        for point in &self.path {
            x += point.x();
            y += point.y();
            absolute.push(InkPoint { x, y });
        }
        absolute
    }

    /// The pen tip used for the ink path.
    ///
    /// The exact meaning is not specified.
    pub fn pen_tip(&self) -> Option<u8> {
        self.pen_tip
    }

    /// The path's transparency
    ///
    /// The exact meaning is not specified. It seems like 0 means translucent and 255 means opaque.
    pub fn transparency(&self) -> Option<u8> {
        self.transparency
    }

    /// The ink stroke's total height.
    pub fn height(&self) -> f32 {
        self.height
    }

    /// The ink stroke's total width.
    ///
    /// The exact meaning is not specified.
    pub fn width(&self) -> f32 {
        self.width
    }

    /// The ink stroke's color.
    ///
    /// The exact meaning is not specified.
    pub fn color(&self) -> Option<u32> {
        self.color
    }

    /// The handwriting-recognition word this stroke is part of, if OneNote for
    /// Windows recognized it.
    ///
    /// Multiple strokes that form the same word share an equal value here. The
    /// same words are also reachable page-wide, in the recognizer's order, via
    /// [`Page::ink_recognition`]. Returns `None` for strokes that are drawing
    /// rather than recognized handwriting.
    ///
    /// [`Page::ink_recognition`]: crate::page::Page::ink_recognition
    pub fn recognized_word(&self) -> Option<&InkRecognizedWord> {
        self.recognized_word.as_ref()
    }
}

/// A point in an ink path.
///
/// Coordinates are in ink units. Before scaling, `1 unit = 0.01 mm`, which is
/// `0.0378 px` at 96 DPI. If the ink specifies `InkScalingX`/`InkScalingY`,
/// these factors are already applied, so the physical size of a unit only
/// holds for a scale of `1`. Coordinates are **not** expressed in any
/// page/outline layout unit (e.g. half-inch); that conversion is the caller's
/// responsibility.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct InkPoint {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl InkPoint {
    /// The point's X coordinate in ink units (`0.01 mm` each).
    pub fn x(&self) -> f32 {
        self.x
    }

    /// The point's Y coordinate in ink units (`0.01 mm` each).
    pub fn y(&self) -> f32 {
        self.y
    }
}

/// The bounding box of an ink object.
#[derive(Clone, Copy, Debug)]
pub struct InkBoundingBox {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) height: f32,
    pub(crate) width: f32,
}

impl InkBoundingBox {
    /// The initial X coordinate of the bounding box.
    ///
    /// The exact meaning and unit are not specified.
    pub fn x(&self) -> f32 {
        self.x
    }

    /// The initial Y coordinate of the bounding box.
    ///
    /// The exact meaning and unit are not specified.
    pub fn y(&self) -> f32 {
        self.y
    }

    /// The height of the bounding box.
    ///
    /// The exact meaning and unit are not specified.
    pub fn height(&self) -> f32 {
        self.height
    }

    /// The width of the bounding box.
    ///
    /// The exact meaning and unit are not specified.
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Scale the bounding box by a constant factor.
    pub fn scale(&self, factor: f32) -> InkBoundingBox {
        InkBoundingBox {
            x: self.x * factor,
            y: self.y * factor,
            height: self.height * factor,
            width: self.width * factor,
        }
    }

    fn union(&self, other: Option<InkBoundingBox>) -> InkBoundingBox {
        let Some(other) = other else {
            return *self;
        };

        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let x2 = (self.x + self.width).max(other.x + other.width);
        let y2 = (self.y + self.height).max(other.y + other.height);

        InkBoundingBox {
            x,
            y,
            height: y2 - y,
            width: x2 - x,
        }
    }
}

pub(crate) fn parse_ink(
    ink_container_id: ExGuid,
    space: &(impl ObjectSpace + ?Sized),
    ctx: &mut ParserContext,
) -> Result<Ink> {
    parse_ink_rec(ink_container_id, space, ctx, 0)
}

// Cut maximum recursion depth at 16 to guard against cycles (e.g. an ink node
// declared to recursively contain itself). An explicit error is easier to
// debug than the stack overflow that would otherwise occur.
const MAX_INK_NESTING_DEPTH: u32 = 16;

fn parse_ink_rec(
    ink_container_id: ExGuid,
    space: &(impl ObjectSpace + ?Sized),
    ctx: &mut ParserContext,
    depth: u32,
) -> Result<Ink> {
    if depth > MAX_INK_NESTING_DEPTH {
        return Err(
            ErrorKind::MalformedOneNoteData("maximum ink nesting depth exceeded".into()).into(),
        );
    }

    let container_object = space
        .get_object(ink_container_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("ink container is missing".into()))?;
    let container = ink_container::parse(container_object)?;

    let Some(ink_data_id) = container.ink_data else {
        let Some(children) = container.children else {
            return Ok(Ink {
                content: InkContent::Strokes(vec![]),
                bounding_box: None,
                offset_horizontal: None,
                offset_vertical: None,
            });
        };

        let (bounding_box, content) = parse_ink_group(children, space, ctx, depth)?;
        return Ok(Ink {
            bounding_box,
            offset_horizontal: container.offset_from_parent_horiz,
            offset_vertical: container.offset_from_parent_vert,
            content,
        });
    };

    let (ink_strokes, bounding_box) = parse_ink_data(
        ink_data_id,
        space,
        container.ink_scaling_x,
        container.ink_scaling_y,
        ctx,
    )?;

    Ok(Ink {
        content: InkContent::Strokes(ink_strokes),
        bounding_box,
        offset_horizontal: container.offset_from_parent_horiz,
        offset_vertical: container.offset_from_parent_vert,
    })
}

fn parse_ink_group(
    children: Vec<ExGuid>,
    space: &(impl ObjectSpace + ?Sized),
    ctx: &mut ParserContext,
    depth: u32,
) -> Result<(Option<InkBoundingBox>, InkContent)> {
    let mut ink_contents = Vec::with_capacity(children.len());
    let mut bbox = None;

    for child_id in children {
        let child = parse_ink_rec(child_id, space, ctx, depth + 1)?;
        if let Some(child_bbox) = &child.bounding_box {
            bbox = Some(child_bbox.union(bbox));
        }
        ink_contents.push(child);
    }

    Ok((bbox, InkContent::InkGroup(ink_contents)))
}

pub(crate) fn parse_ink_data(
    ink_data_id: ExGuid,
    space: &(impl ObjectSpace + ?Sized),
    scale_x: Option<f32>,
    scale_y: Option<f32>,
    ctx: &mut ParserContext,
) -> Result<(Vec<InkStroke>, Option<InkBoundingBox>)> {
    let Some(ink_data_object) = space.get_object(ink_data_id) else {
        warn!(
            ctx,
            "ink data object {:?} not found, skipping ink", ink_data_id
        );
        return Ok((vec![], None));
    };
    let ink_data = ink_data_node::parse(ink_data_object, ctx)?;

    let strokes = ink_data
        .strokes
        .iter()
        .copied()
        .map(|ink_stroke_id| parse_ink_stroke(ink_stroke_id, space, scale_x, scale_y, ctx))
        .collect::<Result<_>>()?;

    let scale_x = scale_x.unwrap_or(1.0);
    let scale_y = scale_y.unwrap_or(1.0);

    let bounding_box = ink_data
        .bounding_box
        .map(|[x_min, y_min, x_max, y_max]| InkBoundingBox {
            x: x_min as f32 * scale_x,
            y: y_min as f32 * scale_y,
            width: (x_max as f32 - x_min as f32) * scale_x,
            height: (y_max as f32 - y_min as f32) * scale_y,
        });

    Ok((strokes, bounding_box))
}

fn parse_ink_stroke(
    ink_stroke_id: ExGuid,
    space: &(impl ObjectSpace + ?Sized),
    scale_x: Option<f32>,
    scale_y: Option<f32>,
    ctx: &mut ParserContext,
) -> Result<InkStroke> {
    let object = space
        .get_object(ink_stroke_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("ink stroke node is missing".into()))?;
    let data = ink_stroke_node::parse(object, ctx)?;

    let props_object = space.get_object(data.properties).ok_or_else(|| {
        ErrorKind::MalformedOneNoteData("ink stroke properties node is missing".into())
    })?;
    let props = stroke_properties_node::parse(props_object)?;

    let path = parse_ink_path(data.path, &props, scale_x, scale_y)?;

    let recognized_word = ctx.recognized_words.get(&ink_stroke_id).cloned();

    Ok(InkStroke {
        path,
        pen_tip: props.pen_tip,
        transparency: props.transparency,
        height: props.ink_height,
        width: props.ink_width,
        color: props.color,
        recognized_word,
    })
}

fn parse_ink_path(
    data: Vec<i64>,
    props: &stroke_properties_node::Data,
    scale_x: Option<f32>,
    scale_y: Option<f32>,
) -> Result<Vec<InkPoint>> {
    // Find dimension indexes
    let idx_x = props
        .dimensions
        .iter()
        .position(|d| d.id == guid!("598a6a8f-52c0-4ba0-93af-af357411a561"))
        .ok_or_else(|| {
            ErrorKind::MalformedOneNoteData("ink stroke properties has no x dimension".into())
        })?;
    let idx_y = props
        .dimensions
        .iter()
        .position(|d| d.id == guid!("b53f9f75-04e0-4498-a7ee-c30dbb5a9011"))
        .ok_or_else(|| {
            ErrorKind::MalformedOneNoteData("ink stroke properties has no y dimension".into())
        })?;

    // Find dimensions data
    let dimension_offset = data.len() / props.dimensions.len();

    let start_x = dimension_offset * idx_x;
    let start_y = dimension_offset * idx_y;

    let x = &data[start_x..start_x + dimension_offset];
    let y = &data[start_y..start_y + dimension_offset];

    let scale_x = scale_x.unwrap_or(1.0);
    let scale_y = scale_y.unwrap_or(1.0);

    let path = x
        .iter()
        .copied()
        .zip(y.iter().copied())
        .map(|(x, y)| InkPoint {
            x: scale_x * x as f32,
            y: scale_y * y as f32,
        })
        .collect();

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::one::property::ink_dimensions::InkDimension;
    use crate::one::property_set::stroke_properties_node::Data;

    /// A synthetic stroke-properties node carrying exactly the two well-known
    /// ISF dimension records the parser looks up (X first, then Y).
    fn dimensions() -> Vec<InkDimension> {
        vec![
            InkDimension {
                id: guid!("598a6a8f-52c0-4ba0-93af-af357411a561"),
                limit_lower: 0,
                limit_upper: 0,
            },
            InkDimension {
                id: guid!("b53f9f75-04e0-4498-a7ee-c30dbb5a9011"),
                limit_lower: 0,
                limit_upper: 0,
            },
        ]
    }

    fn stroke_properties() -> Data {
        Data {
            aliased: false,
            fit_to_curve: false,
            ignore_pressure: false,
            pen_tip: None,
            raster_operation: None,
            transparency: None,
            ink_height: 0.0,
            ink_width: 0.0,
            color: None,
            dimensions: dimensions(),
        }
    }

    /// Parses a stroke from dimension-first storage: all X values, then all Y
    /// values (x = [500, 12, 15], y = [300, -3, 2]).
    fn stroke() -> InkStroke {
        let data = vec![500, 12, 15, 300, -3, 2];
        InkStroke {
            path: parse_ink_path(data, &stroke_properties(), None, None).expect("parses"),
            pen_tip: None,
            transparency: None,
            height: 0.0,
            width: 0.0,
            color: None,
            recognized_word: None,
        }
    }

    #[test]
    fn path_keeps_the_stored_delta_encoding() {
        assert_eq!(
            stroke().path(),
            [
                InkPoint { x: 500.0, y: 300.0 },
                InkPoint { x: 12.0, y: -3.0 },
                InkPoint { x: 15.0, y: 2.0 },
            ]
        );
    }

    #[test]
    fn absolute_path_accumulates_the_stored_deltas() {
        assert_eq!(
            stroke().absolute_path(),
            [
                InkPoint { x: 500.0, y: 300.0 },
                InkPoint { x: 512.0, y: 297.0 },
                InkPoint { x: 527.0, y: 299.0 },
            ]
        );
    }
}
