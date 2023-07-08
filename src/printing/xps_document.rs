use std::sync::Arc;

use log::error;
use windows::{
    core::{Error, HSTRING},
    w,
    Win32::{
        Storage::{
            Packaging::Opc::IOpcPartUri,
            Xps::{
                IXpsOMDocument, IXpsOMDocumentSequence, IXpsOMObjectFactory, IXpsOMPackage,
                XpsOMObjectFactory, XPS_SIZE,
            },
        },
        System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER},
    },
};

use super::xps_page::XPSPage;

pub const PAGE_SIZE_A4: XPS_SIZE = XPS_SIZE {
    width: 2100.0,
    height: 2970.0,
};

pub struct XPSSingleDocument {
    pub(super) factory: Arc<IXpsOMObjectFactory>,
    pub(super) package: IXpsOMPackage,
    pub(super) document_fd: Option<IXpsOMDocument>,
    pub(super) fixed_sequence_part: Option<IOpcPartUri>,
    pub(super) document_sequence: Option<IXpsOMDocumentSequence>,
    pub(super) document_part: Option<IOpcPartUri>,
    pub(super) pages: Vec<XPSPage>,
}

impl XPSSingleDocument {
    pub fn page_at(&mut self, index: usize) -> Option<&mut XPSPage> {
        return self.pages.get_mut(index);
    }

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
                factory: Arc::new(factory),
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

    pub fn newPage(&mut self) -> Result<usize, ()> {
        assert!(self.document_fd.is_some());

        let part_uri_result = unsafe {
            self.factory.CreatePartUri(&HSTRING::from(format!(
                "/Documents/1/Pages/{}.fpage",
                self.pages.len() + 1
            )))
        };

        let Ok(part_uri) = part_uri_result else {
            error!("couldn't create page part uri: {:?}", part_uri_result.unwrap_err());
            return Err(());
        };

        let page_result = unsafe {
            self.factory
                .CreatePage(&PAGE_SIZE_A4, w!("de-DE"), &part_uri)
        };

        let Ok(page) = page_result else {
            error!("couldn't create page: {:?}", page_result.unwrap_err());
            return Err(());
        };

        let page_ref_result = unsafe { self.factory.CreatePageReference(&PAGE_SIZE_A4) };

        let Ok(xpsPageRef) = page_ref_result else {
            error!("couldn't create page reference: {:?}", page_ref_result.unwrap_err());
            return Err(());
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

        let page = XPSPage {
            page: page,
            factory: Arc::clone(&self.factory),
        };

        self.pages.push(page);
        return Ok(self.pages.len() - 1);
    }

    pub fn safe(&self) {
        const FILE_ATTRIBUTE_NORMAL: u32 = 0x00000080; // for some reason this isn't defined in the winapi rust wrapper
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
