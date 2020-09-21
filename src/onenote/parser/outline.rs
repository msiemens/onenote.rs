use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property_set::{outline_element_node, outline_group, outline_node, PropertySetId};
use crate::onenote::parser::content::{parse_content, Content};
use crate::onenote::parser::list::{parse_list, List};
use crate::onestore::object_space::ObjectSpace;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub struct Outline {
    pub(crate) items: Vec<OutlineItem>,
    pub(crate) items_level: u8,
    pub(crate) list_spacing: Option<f32>,
    pub(crate) indents: Vec<f32>,
    pub(crate) alignment_in_parent: Option<LayoutAlignment>,
    pub(crate) alignment_self: Option<LayoutAlignment>,
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
}

#[derive(Debug)]
pub enum OutlineItem {
    Group(OutlineGroup),
    Element(OutlineElement),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct OutlineElement {
    pub(crate) contents: Vec<Content>,

    pub(crate) list_contents: Vec<List>,
    pub(crate) list_spacing: Option<f32>,

    pub(crate) child_level: u8,
    pub(crate) children: Vec<OutlineItem>,
}

pub(crate) fn parse_outline(outline_id: ExGuid, space: &ObjectSpace) -> Outline {
    let outline_object = space
        .get_object(outline_id)
        .expect("outline node is missing");
    let data = outline_node::parse(outline_object);

    let items = data
        .children()
        .iter()
        .map(|id| parse_outline_item(*id, space))
        .collect();

    Outline {
        items,
        items_level: data.child_level(),
        list_spacing: data.list_spacing(),
        indents: data.outline_indent_distance().value().to_vec(),
        alignment_in_parent: data.layout_alignment_in_parent().copied(),
        alignment_self: data.layout_alignment_self().copied(),
    }
}

fn parse_outline_item(item_id: ExGuid, space: &ObjectSpace) -> OutlineItem {
    let content_type = space
        .get_object(item_id)
        .expect("outline item is missing")
        .id();
    let id = PropertySetId::from_jcid(content_type).unwrap();

    match id {
        PropertySetId::OutlineGroup => OutlineItem::Group(parse_outline_group(item_id, space)),
        PropertySetId::OutlineElementNode => {
            OutlineItem::Element(parse_outline_element(item_id, space))
        }
        _ => panic!("invalid outline item type: {:?}", id),
    }
}

fn parse_outline_group(group_id: ExGuid, space: &ObjectSpace) -> OutlineGroup {
    let group_object = space
        .get_object(group_id)
        .expect("outline group is missing");
    let data = outline_group::parse(group_object);

    let outlines = data
        .children()
        .iter()
        .map(|id| parse_outline_item(*id, space))
        .collect();

    OutlineGroup {
        child_level: data.child_level(),
        outlines,
    }
}

pub(crate) fn parse_outline_element(element_id: ExGuid, space: &ObjectSpace) -> OutlineElement {
    let element_object = space
        .get_object(element_id)
        .expect("outline element is missing");
    let data = outline_element_node::parse(element_object);

    let children = data
        .children()
        .iter()
        .map(|id| parse_outline_item(*id, space))
        .collect();

    let contents = data
        .contents()
        .iter()
        .map(|id| parse_content(*id, space))
        .collect();

    let list_contents = data
        .list_contents()
        .iter()
        .map(|id| parse_list(*id, space))
        .collect();

    OutlineElement {
        child_level: data.child_level(),
        list_spacing: data.list_spacing(),
        children,
        contents,
        list_contents,
    }
}
