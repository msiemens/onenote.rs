#[derive(Debug)]
pub struct Notebook {
    pub(crate) sections: Vec<Section>,
}

#[derive(Debug)]
pub struct Section {
    pub(crate) pages: Vec<Page>,
}

#[derive(Debug)]
pub struct Page {
    pub(crate) contents: Vec<PageContent>,
}

#[derive(Debug)]
pub enum PageContent {
    Outline(Outline),
    Content(Content),
}

#[derive(Debug)]
pub struct Outline {
    pub(crate) paragraphs: Vec<Paragraph>,
}

#[derive(Debug)]
pub struct Paragraph {
    pub(crate) content: Vec<Content>,
}

#[derive(Debug)]
pub enum Content {
    Paragraph(Paragraph),
    RichText(RichText),
    Image(Image),
    Table(Table),
}

#[derive(Debug)]
pub struct RichText {}

#[derive(Debug)]
pub struct Image {
    format: ImageFormat,
    data: Vec<u8>,
}

#[derive(Debug)]
pub enum ImageFormat {
    DIB,
    EMF,
    JPEG,
    PNG,
    TIFF,
    WMF,
}

#[derive(Debug)]
pub struct Table {
    rows: u32,
    cols: u32,
    contents: Vec<TableRow>,
}

#[derive(Debug)]
pub struct TableRow {
    contents: Vec<TableCell>,
}

#[derive(Debug)]
pub struct TableCell {
    contents: Vec<Outline>,
}
