use crate::one::property::object_reference::ObjectReference;
use crate::one::property::time::Time;
use crate::one::property::{simple, PropertyType};

use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;

use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    created_at: Time,
    last_modified: Time,
    children: Vec<ExGuid>,
    child_level: u8,
    contents: Vec<ExGuid>,
    list_contents: Vec<ExGuid>,
    list_spacing: Option<f32>,
    author_original: ExGuid,
    author_most_recent: ExGuid,
    rtl: bool,
    is_deletable: bool,
    is_selectable: bool,
    is_title_text: bool,
}

impl Data {
    pub(crate) fn children(&self) -> &[ExGuid] {
        &self.children
    }

    pub(crate) fn child_level(&self) -> u8 {
        self.child_level
    }

    pub(crate) fn contents(&self) -> &[ExGuid] {
        &self.contents
    }

    pub(crate) fn list_contents(&self) -> &[ExGuid] {
        &self.list_contents
    }

    pub(crate) fn list_spacing(&self) -> Option<f32> {
        self.list_spacing
    }
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::OutlineElementNode.as_jcid());

    let created_at = Time::parse(PropertyType::CreationTimeStamp, object)
        .expect("outline element has no creation timestamp");
    let last_modified = Time::parse(PropertyType::LastModifiedTime, object)
        .expect("outline element has no last modified time");
    let children =
        ObjectReference::parse_vec(PropertyType::ElementChildNodes, object).unwrap_or_default();
    let child_level = simple::parse_u8(PropertyType::OutlineElementChildLevel, object)
        .expect("outline has no child element level");
    let contents =
        ObjectReference::parse_vec(PropertyType::ContentChildNodes, object).unwrap_or_default();
    let list_contents =
        ObjectReference::parse_vec(PropertyType::ListNodes, object).unwrap_or_default();
    let list_spacing = simple::parse_f32(PropertyType::ListSpacingMu, object);
    let author_original = ObjectReference::parse(PropertyType::AuthorOriginal, object)
        .expect("outline element has no original author");
    let author_most_recent = ObjectReference::parse(PropertyType::AuthorMostRecent, object)
        .expect("outline element has no most recent author");
    let rtl = simple::parse_bool(PropertyType::OutlineElementRTL, object).unwrap_or_default();
    let is_deletable = simple::parse_bool(PropertyType::Deletable, object).unwrap_or_default();
    let is_selectable = simple::parse_bool(PropertyType::CannotBeSelected, object)
        .map(|value| !value)
        .unwrap_or_default();
    let is_title_text = simple::parse_bool(PropertyType::IsTitleText, object).unwrap_or_default();

    Data {
        created_at,
        last_modified,
        children,
        child_level,
        contents,
        list_contents,
        list_spacing,
        author_original,
        author_most_recent,
        rtl,
        is_deletable,
        is_selectable,
        is_title_text,
    }
}
