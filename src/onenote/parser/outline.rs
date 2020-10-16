use crate::errors::{ErrorKind, Result};
use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property_set::{outline_element_node, outline_group, outline_node, PropertySetId};
use crate::onenote::parser::content::{parse_content, Content};
use crate::onenote::parser::list::{parse_list, List};
use crate::onestore::object_space::ObjectSpace;
use crate::types::exguid::ExGuid;

#[derive(Clone, Debug)]
pub struct Outline {
    pub(crate) items_level: u8,
    pub(crate) list_spacing: Option<f32>,
    pub(crate) indents: Vec<f32>,

    pub(crate) alignment_in_parent: Option<LayoutAlignment>,
    pub(crate) alignment_self: Option<LayoutAlignment>,

    pub(crate) layout_max_height: Option<f32>,
    pub(crate) layout_max_width: Option<f32>,
    pub(crate) layout_reserved_width: Option<f32>,
    pub(crate) layout_minimum_outline_width: Option<f32>,
    pub(crate) is_layout_size_set_by_user: bool,
    pub(crate) offset_horizontal: Option<f32>,
    pub(crate) offset_vertical: Option<f32>,

    pub(crate) items: Vec<OutlineItem>,
}

impl Outline {
    pub fn items(&self) -> &[OutlineItem] {
        &self.items
    }

    pub fn items_level(&self) -> u8 {
        self.items_level
    }

    pub fn list_spacing(&self) -> Option<f32> {
        self.list_spacing
    }

    pub fn indents(&self) -> &[f32] {
        &self.indents
    }

    pub fn alignment_in_parent(&self) -> Option<LayoutAlignment> {
        self.alignment_in_parent
    }

    pub fn alignment_self(&self) -> Option<LayoutAlignment> {
        self.alignment_self
    }

    pub fn layout_max_height(&self) -> Option<f32> {
        self.layout_max_height
    }

    pub fn layout_max_width(&self) -> Option<f32> {
        self.layout_max_width
    }

    pub fn layout_reserved_width(&self) -> Option<f32> {
        self.layout_reserved_width
    }

    pub fn layout_minimum_outline_width(&self) -> Option<f32> {
        self.layout_minimum_outline_width
    }

    pub fn is_layout_size_set_by_user(&self) -> bool {
        self.is_layout_size_set_by_user
    }

    pub fn offset_horizontal(&self) -> Option<f32> {
        self.offset_horizontal
    }

    pub fn offset_vertical(&self) -> Option<f32> {
        self.offset_vertical
    }
}

#[derive(Clone, Debug)]
pub enum OutlineItem {
    Group(OutlineGroup),
    Element(OutlineElement),
}

impl OutlineItem {
    pub fn element(&self) -> Option<&OutlineElement> {
        if let OutlineItem::Element(element) = self {
            Some(element)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct OutlineGroup {
    pub(crate) child_level: u8,
    pub(crate) outlines: Vec<OutlineItem>,
}

impl OutlineGroup {
    pub fn child_level(&self) -> u8 {
        self.child_level
    }

    pub fn outlines(&self) -> &[OutlineItem] {
        &self.outlines
    }
}

#[derive(Clone, Debug)]
pub struct OutlineElement {
    pub(crate) contents: Vec<Content>,

    pub(crate) list_contents: Vec<List>,
    pub(crate) list_spacing: Option<f32>,

    pub(crate) child_level: u8,
    pub(crate) children: Vec<OutlineItem>,
}

impl OutlineElement {
    pub fn contents(&self) -> &[Content] {
        &self.contents
    }

    pub fn list_contents(&self) -> &[List] {
        &self.list_contents
    }

    pub fn list_spacing(&self) -> Option<f32> {
        self.list_spacing
    }

    pub fn child_level(&self) -> u8 {
        self.child_level
    }

    pub fn children(&self) -> &[OutlineItem] {
        &self.children
    }
}

pub(crate) fn parse_outline(outline_id: ExGuid, space: &ObjectSpace) -> Result<Outline> {
    let outline_object = space
        .get_object(outline_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("outline node is missing".into()))?;
    let data = outline_node::parse(outline_object)?;

    let items = data
        .children
        .into_iter()
        .map(|item_id| parse_outline_item(item_id, space))
        .collect::<Result<_>>()?;

    let outline = Outline {
        items,
        items_level: data.child_level,
        list_spacing: data.list_spacing,
        indents: data.outline_indent_distance.into_value(),
        alignment_in_parent: data.layout_alignment_in_parent,
        alignment_self: data.layout_alignment_self,
        layout_max_height: data.layout_max_height,
        layout_max_width: data.layout_max_width,
        layout_reserved_width: data.layout_reserved_width,
        layout_minimum_outline_width: data.layout_minimum_outline_width,
        is_layout_size_set_by_user: data.is_layout_size_set_by_user,
        offset_horizontal: data.offset_from_parent_horiz,
        offset_vertical: data.offset_from_parent_vert,
    };

    Ok(outline)
}

fn parse_outline_item(item_id: ExGuid, space: &ObjectSpace) -> Result<OutlineItem> {
    let content_type = space
        .get_object(item_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("outline item is missing".into()))?
        .id();
    let id = PropertySetId::from_jcid(content_type).ok_or_else(|| {
        ErrorKind::MalformedOneNoteData(
            format!("invalid property set id: 0x{:X}", content_type.0).into(),
        )
    })?;

    let item = match id {
        PropertySetId::OutlineGroup => OutlineItem::Group(parse_outline_group(item_id, space)?),
        PropertySetId::OutlineElementNode => {
            OutlineItem::Element(parse_outline_element(item_id, space)?)
        }
        _ => {
            return Err(ErrorKind::MalformedOneNoteData(
                format!("invalid outline item type: {:?}", id).into(),
            )
            .into())
        }
    };

    Ok(item)
}

fn parse_outline_group(group_id: ExGuid, space: &ObjectSpace) -> Result<OutlineGroup> {
    let group_object = space
        .get_object(group_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("outline group is missing".into()))?;
    let data = outline_group::parse(group_object)?;

    let outlines = data
        .children
        .into_iter()
        .map(|item_id| parse_outline_item(item_id, space))
        .collect::<Result<_>>()?;

    let group = OutlineGroup {
        child_level: data.child_level,
        outlines,
    };

    Ok(group)
}

pub(crate) fn parse_outline_element(
    element_id: ExGuid,
    space: &ObjectSpace,
) -> Result<OutlineElement> {
    let element_object = space
        .get_object(element_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("outline element is missing".into()))?;
    let data = outline_element_node::parse(element_object)?;

    let children = data
        .children
        .into_iter()
        .map(|item_id| parse_outline_item(item_id, space))
        .collect::<Result<_>>()?;

    let contents = data
        .contents
        .into_iter()
        .map(|content_id| parse_content(content_id, space))
        .collect::<Result<_>>()?;

    let list_contents = data
        .list_contents
        .into_iter()
        .map(|list_id| parse_list(list_id, space))
        .collect::<Result<_>>()?;

    let element = OutlineElement {
        child_level: data.child_level,
        list_spacing: data.list_spacing,
        children,
        contents,
        list_contents,
    };

    Ok(element)
}
