use imap::error::No;
use log::{error, trace};
use windows::{
    core::{ComInterface, Error, HSTRING},
    w,
    Win32::{
        Storage::{
            Packaging::Opc::IOpcPartUri,
            Xps::{
                IXpsOMDocument, IXpsOMDocumentSequence, IXpsOMFontResource, IXpsOMObjectFactory,
                IXpsOMPackage, IXpsOMPage, IXpsOMSolidColorBrush, XpsOMObjectFactory, XPS_COLOR,
                XPS_COLOR_0_1, XPS_COLOR_TYPE, XPS_COLOR_TYPE_SRGB, XPS_FONT_EMBEDDING_OBFUSCATED,
                XPS_POINT, XPS_SIZE,
            },
        },
        System::Com::{CoCreateGuid, CoCreateInstance, StringFromGUID2, CLSCTX_INPROC_SERVER},
    },
};

pub const page_size_a4: XPS_SIZE = XPS_SIZE {
    width: 210.0,
    height: 297.0,
};

pub struct XPSSingleDocument {
    pub(super) factory: IXpsOMObjectFactory,
    pub(super) package: IXpsOMPackage,
    pub(super) document_fd: Option<IXpsOMDocument>,
    pub(super) fixed_sequence_part: Option<IOpcPartUri>,
    pub(super) document_sequence: Option<IXpsOMDocumentSequence>,
    pub(super) document_part: Option<IOpcPartUri>,
    pub(super) pages: Vec<IXpsOMPage>,
}

impl XPSSingleDocument {
    fn prepare_document(&mut self) {
        let part_uri_result = unsafe {
            self.factory
                .CreatePartUri(w!("/FixedDocumentSequence.fdseq"))
        };

        let Ok(part_uri) = part_uri_result else {
            error!("couldn't create part uri: {:?}", part_uri_result.unwrap_err());
            return;
        };

        let doc_seq_result = unsafe { self.factory.CreateDocumentSequence(&part_uri) };
        self.fixed_sequence_part = Some(part_uri);

        let Ok(doc_seq) = doc_seq_result else {
            error!("couldn't create document sequence: {:?}", doc_seq_result.unwrap_err());
            return;
        };

        let doc_seq_result = unsafe { self.package.SetDocumentSequence(&doc_seq) };

        let Ok(_) = doc_seq_result else {
            error!("couldn't set document sequence: {:?}", doc_seq_result.unwrap_err());
            return;
        };
        self.document_sequence = Some(doc_seq);

        let part_uri_result = unsafe {
            self.factory
                .CreatePartUri(w!("/Documents/1/FixedDocument.fdoc"))
        };

        let Ok(part_uri) = part_uri_result else {
            error!("couldn't create document part uri: {:?}", part_uri_result.unwrap_err());
            return;
        };

        let doc_result = unsafe { self.factory.CreateDocument(&part_uri) };
        self.document_part = Some(part_uri);

        let Ok(doc) = doc_result else {
            error!("couldn't create document: {:?}", doc_result.unwrap_err());
            return;
        };
        unsafe {
            self.document_sequence
                .as_mut()
                .unwrap()
                .GetDocuments()
                .unwrap()
                .Append(&doc)
                .unwrap()
        };
        self.document_fd = Some(doc);
    }

    pub fn new() -> Result<Self, Error> {
        let factory = unsafe { CoCreateInstance(&XpsOMObjectFactory, None, CLSCTX_INPROC_SERVER) };

        if let Err(e) = factory {
            error!("failed to create XPS Document: {:?}", e);
            return Err(e);
        }
        let factory: IXpsOMObjectFactory = factory.unwrap();

        let package = unsafe { factory.CreatePackage() };

        let mut res = match package {
            Ok(package) => Self {
                factory: factory,
                package: package,
                pages: vec![],
                document_fd: None,
                document_sequence: None,
                fixed_sequence_part: None,
                document_part: None,
            },
            Err(e) => {
                error!("failed to create XPS Document Package: {:?}", e);
                return Err(e);
            }
        };

        res.prepare_document();
        return Ok(res);
    }

    pub fn newPage(&mut self) {
        assert!(self.document_fd.is_some());

        let part_uri_result = unsafe {
            self.factory.CreatePartUri(&HSTRING::from(format!(
                "/Documents/1/Pages/{}.fpage",
                self.pages.len() + 1
            )))
        };

        let Ok(part_uri) = part_uri_result else {
            error!("couldn't create page part uri: {:?}", part_uri_result.unwrap_err());
            return;
        };

        let page_result = unsafe {
            self.factory
                .CreatePage(&page_size_a4, w!("de-DE"), &part_uri)
        };

        let Ok(page) = page_result else {
            error!("couldn't create page: {:?}", page_result.unwrap_err());
            return;
        };

        let page_ref_result = unsafe { self.factory.CreatePageReference(&page_size_a4) };

        let Ok(xpsPageRef) = page_ref_result else {
            error!("couldn't create page reference: {:?}", page_ref_result.unwrap_err());
            return;
        };

        unsafe {
            self.document_fd
                .as_mut()
                .unwrap()
                .GetPageReferences()
                .unwrap()
                .Append(&xpsPageRef)
                .unwrap();
            xpsPageRef.SetPage(&page).unwrap();
        };
        self.pages.push(page);
    }

    pub fn addText(&mut self, page: usize, text: String, x: f32, y: f32) {
        assert!(self.document_fd.is_some());
        assert!(self.pages.len() >= page);

        let page = &self.pages[page - 1];

        let font = self.loadFont("Arial").unwrap();

        let glyphs = unsafe { self.factory.CreateGlyphs(&font) };
        let Ok(glyphs) = glyphs else {
            error!("couldn't create glyphs: {:?}", glyphs.unwrap_err());
            return;
        };

        let origin_res = unsafe { glyphs.SetOrigin(&XPS_POINT { x: x, y: y }) };
        let Ok(_) = origin_res else {
            error!("couldn't set origin: {:?}", origin_res.unwrap_err());
            return;
        };

        let font_size_res = unsafe { glyphs.SetFontRenderingEmSize(12.0) };
        let Ok(_) = font_size_res else {
            error!("couldn't set font size: {:?}", font_size_res.unwrap_err());
            return;
        };

        let brush = self.createColourBrush(0, 0, 0).unwrap();
        let brush_res = unsafe { glyphs.SetFillBrushLocal(&brush) };
        let Ok(_) = brush_res else {
            error!("couldn't set brush: {:?}", brush_res.unwrap_err());
            return;
        };

        let glyphs_editor = unsafe { glyphs.GetGlyphsEditor() };
        let Ok(glyphs_editor) = glyphs_editor else {
            error!("couldn't get glyphs editor: {:?}", glyphs_editor.unwrap_err());
            return;
        };

        let text_res = unsafe { glyphs_editor.SetUnicodeString(&HSTRING::from(text)) };
        let Ok(_) = text_res else {
            error!("couldn't set text: {:?}", text_res.unwrap_err());
            return;
        };

        let apply_res = unsafe { glyphs_editor.ApplyEdits() };
        let Ok(_) = apply_res else {
            error!("couldn't apply edits: {:?}", apply_res.unwrap_err());
            return;
        };

        let page_add_res = unsafe { page.GetVisuals().unwrap().Append(&glyphs) };
        let Ok(_) = page_add_res else {
            error!("couldn't add glyphs to page: {:?}", page_add_res.unwrap_err());
            return;
        };
    }

    pub fn loadFont(&self, font_name: &str) -> Result<IXpsOMFontResource, ()> {
        let in_stream = unsafe {
            self.factory
                .CreateReadOnlyStreamOnFile(&HSTRING::from(format!(
                    "C:\\Windows\\Fonts\\{}.ttf",
                    font_name
                )))
        };
        let Ok(in_stream) = in_stream else {
            error!("couldn't create font stream: {:?}", in_stream.unwrap_err());
            return Err(());
        };

        let guid = unsafe { CoCreateGuid() };
        let Ok(guid) = guid else {
            error!("couldn't create guid: {:?}", guid.unwrap_err());
            return Err(());
        };
        let mut guid_string: Vec<u16> = vec![0; 39];
        let res = unsafe { StringFromGUID2(&guid, &mut guid_string) };
        let guid_str = String::from_utf16(&guid_string).unwrap();
        let uri_str = format!(
            "/Resources/Fonts/{}.odttf",
            &guid_str[1..guid_str.len() - 2]
        );
        trace!("uri_str: {:?}", uri_str);
        let uri_str = HSTRING::from(uri_str);
        println!("guid: {:?}", guid_string);

        let part_uri = unsafe { self.factory.CreatePartUri(&uri_str) };
        let Ok(part_uri) = part_uri else {
            error!("couldn't create font part uri: {:?}", part_uri.unwrap_err());
            return Err(());
        };

        let font_resource = unsafe {
            self.factory.CreateFontResource(
                &in_stream,
                XPS_FONT_EMBEDDING_OBFUSCATED,
                &part_uri,
                false,
            )
        };
        let Ok(font_resource) = font_resource else {
            error!("couldn't create font resource: {:?}", font_resource.unwrap_err());
            return Err(());
        };

        return Ok(font_resource);
    }

    pub fn createColourBrush(&self, r: u8, g: u8, b: u8) -> Result<IXpsOMSolidColorBrush, ()> {
        let xps_colour = XPS_COLOR {
            colorType: XPS_COLOR_TYPE_SRGB,
            value: windows::Win32::Storage::Xps::XPS_COLOR_0 {
                sRGB: XPS_COLOR_0_1 {
                    alpha: 0xff,
                    red: r,
                    green: g,
                    blue: b,
                },
            },
        };

        let colour = unsafe { self.factory.CreateSolidColorBrush(&xps_colour, None) };
        let Ok(colour_brush) = colour else {
            error!("couldn't create brush: {:?}", colour.unwrap_err());
            return Err(());
        };

        return Ok(colour_brush);
    }

    pub fn safe(&self) {
        const FILE_ATTRIBUTE_NORMAL: u32 = 0x00000080;
        unsafe {
            self.package
                .WriteToFile(
                    w!("test.xps"),
                    std::ptr::null(),
                    FILE_ATTRIBUTE_NORMAL,
                    false,
                )
                .unwrap();
        }
    }
}
