use crate::errors::Result;
use crate::one::property::{PropertyType, simple};
use crate::onestore::object::Object;

/// An embedded ink handwriting container.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct Data {
    pub(crate) space_width: Option<f32>,
    pub(crate) space_height: Option<f32>,

    pub(crate) start_x: Option<f32>,
    pub(crate) start_y: Option<f32>,
    pub(crate) height: Option<f32>,
    pub(crate) width: Option<f32>,
    pub(crate) offset_horiz: Option<f32>,
    pub(crate) offset_vert: Option<f32>,
}

impl Data {
    pub(crate) fn parse(object: Object) -> Result<Data> {
        let space_width = simple::parse_f32(PropertyType::EmbeddedInkSpaceWidth, &object)?;
        let space_height = simple::parse_f32(PropertyType::EmbeddedInkSpaceHeight, &object)?;

        let start_x = simple::parse_f32(PropertyType::EmbeddedInkStartX, &object)?;
        let start_y = simple::parse_f32(PropertyType::EmbeddedInkStartY, &object)?;
        let height = simple::parse_f32(PropertyType::EmbeddedInkHeight, &object)?;
        let width = simple::parse_f32(PropertyType::EmbeddedInkWidth, &object)?;
        let offset_horiz = simple::parse_f32(PropertyType::EmbeddedInkOffsetHoriz, &object)?;
        let offset_vert = simple::parse_f32(PropertyType::EmbeddedInkOffsetVert, &object)?;

        let data = Data {
            space_width,
            space_height,
            start_x,
            start_y,
            height,
            width,
            offset_horiz,
            offset_vert,
        };

        Ok(data)
    }
}
