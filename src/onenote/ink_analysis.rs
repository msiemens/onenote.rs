use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::one::property_set::{
    ink_analysis, ink_analysis_line, ink_analysis_paragraph, ink_analysis_word,
};
use crate::onestore::object_space::ObjectSpace;
use itertools::Itertools;

/// The results of OCR analysis of ink handwriting.
#[derive(Clone, Debug)]
pub struct InkAnalysis {
    pub(crate) paragraphs: Vec<InkAnalysisParagraph>,
}

impl InkAnalysis {
    pub fn paragraphs(&self) -> &[InkAnalysisParagraph] {
        &self.paragraphs
    }
}

#[derive(Clone, Debug)]
pub struct InkAnalysisParagraph {
    pub(crate) lines: Vec<InkAnalysisLine>,
}

impl InkAnalysisParagraph {
    pub fn lines(&self) -> &[InkAnalysisLine] {
        &self.lines
    }
}

#[derive(Clone, Debug)]
pub struct InkAnalysisLine {
    pub(crate) words: Vec<InkAnalysisWord>,
}

impl InkAnalysisLine {
    pub fn words(&self) -> &[InkAnalysisWord] {
        &self.words
    }
}

#[derive(Clone, Debug)]
pub struct InkAnalysisWord {
    pub(crate) language_code: Option<u32>,
    pub(crate) alternatives: Vec<String>,
}

impl InkAnalysisWord {
    pub fn alternatives(&self) -> &[String] {
        &self.alternatives
    }
}

pub(crate) fn parse_ink_analysis(
    ink_analysis_id: ExGuid,
    space: &ObjectSpace,
) -> Result<InkAnalysis> {
    let container_object = space.get_object(ink_analysis_id).ok_or_else(|| {
        ErrorKind::MalformedOneNoteData("ink analysis container is missing".into())
    })?;
    let container_data = ink_analysis::parse(container_object)?;

    let paragraphs = container_data
        .paragraphs
        .iter()
        .map(|id| parse_ink_analysis_paragraph(*id, space))
        .collect::<Result<_>>()?;

    Ok(InkAnalysis { paragraphs })
}

fn parse_ink_analysis_paragraph(
    paragraph_id: ExGuid,
    space: &ObjectSpace,
) -> Result<InkAnalysisParagraph> {
    let object = space.get_object(paragraph_id).ok_or_else(|| {
        ErrorKind::MalformedOneNoteData("ink analysis paragraph is missing".into())
    })?;
    let data = ink_analysis_paragraph::parse(object)?;

    let lines = data
        .lines
        .iter()
        .map(|id| parse_ink_analysis_line(*id, space))
        .collect::<Result<_>>()?;

    Ok(InkAnalysisParagraph { lines })
}

fn parse_ink_analysis_line(line_id: ExGuid, space: &ObjectSpace) -> Result<InkAnalysisLine> {
    let object = space
        .get_object(line_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("ink analysis line is missing".into()))?;
    let data = ink_analysis_line::parse(object)?;

    let words = data
        .words
        .iter()
        .map(|id| parse_ink_analysis_word(*id, space))
        .collect::<Result<_>>()?;

    Ok(InkAnalysisLine { words })
}

fn parse_ink_analysis_word(word_id: ExGuid, space: &ObjectSpace) -> Result<InkAnalysisWord> {
    let object = space
        .get_object(word_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("ink analysis word is missing".into()))?;
    let data = ink_analysis_word::parse(object)?;

    Ok(InkAnalysisWord {
        language_code: data.language_code,
        alternatives: data.alternatives,
    })
}
