use crate::one::property::author::Author;
use crate::one::property::object_reference::ObjectReference;
use crate::one::property::page_size::PageSize;
use crate::one::property::time::Time;
use crate::one::property::{simple, PropertyType};

use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;

use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    last_modified: Option<Time>,
    cached_title: Option<String>,
    author: Option<Author>, // FIXME: Force this?
    content: Vec<ExGuid>,
    title: ExGuid,
    orientation_portrait: bool,
    page_width: Option<f32>,  // FIXME: Force this?
    page_height: Option<f32>, // FIXME: Force this?
    page_margin_origin_x: Option<f32>,
    page_margin_origin_y: Option<f32>,
    page_margin_left: Option<f32>,   // FIXME: Force this?
    page_margin_right: Option<f32>,  // FIXME: Force this?
    page_margin_top: Option<f32>,    // FIXME: Force this?
    page_margin_bottom: Option<f32>, // FIXME: Force this?
    page_size: PageSize,
    rtl: bool,
}

impl Data {
    pub(crate) fn author(&self) -> Option<&Author> {
        self.author.as_ref()
    }

    pub(crate) fn content(&self) -> &[ExGuid] {
        &self.content
    }

    pub(crate) fn title(&self) -> ExGuid {
        self.title
    }

    pub(crate) fn page_height(&self) -> Option<f32> {
        self.page_height
    }
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::PageNode.as_jcid());

    let last_modified = Time::parse(PropertyType::LastModifiedTime, object);
    let cached_title = simple::parse_string(PropertyType::CachedTitleStringFromPage, object);
    let author = Author::parse(object);
    let content =
        ObjectReference::parse_vec(PropertyType::ElementChildNodes, object).unwrap_or_default();
    let title = ObjectReference::parse_vec(PropertyType::StructureElementChildNodes, object)
        .expect("page has no title reference prop")
        .first()
        .copied()
        .expect("page has no title");
    let orientation_portrait =
        simple::parse_bool(PropertyType::PortraitPage, object).unwrap_or_default();
    let page_width = simple::parse_f32(PropertyType::PageWidth, object);
    let page_height = simple::parse_f32(PropertyType::PageHeight, object);
    let page_margin_origin_x = simple::parse_f32(PropertyType::PageMarginOriginX, object);
    let page_margin_origin_y = simple::parse_f32(PropertyType::PageMarginOriginY, object);
    let page_margin_left = simple::parse_f32(PropertyType::PageMarginLeft, object);
    let page_margin_right = simple::parse_f32(PropertyType::PageMarginRight, object);
    let page_margin_top = simple::parse_f32(PropertyType::PageMarginTop, object);
    let page_margin_bottom = simple::parse_f32(PropertyType::PageMarginBottom, object);
    let page_size = PageSize::parse(PropertyType::PageSize, object).unwrap_or_default();
    let rtl = simple::parse_bool(PropertyType::EditRootRTL, object).unwrap_or_default();

    Data {
        last_modified,
        cached_title,
        author,
        content,
        title,
        orientation_portrait,
        page_width,
        page_height,
        page_margin_origin_x,
        page_margin_origin_y,
        page_margin_left,
        page_margin_right,
        page_margin_top,
        page_margin_bottom,
        page_size,
        rtl,
    }
}
