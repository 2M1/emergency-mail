use std::{path::Path, sync::Arc};

use log::error;

use windows::{
    core::HSTRING,
    w,
    Win32::{
        Storage::{
            Packaging::Opc::IOpcPartUri,
            Xps::{
                IXpsOMDocument, IXpsOMDocumentSequence, IXpsOMObjectFactory, IXpsOMPackage,
                XpsOMObjectFactory,
            },
        },
        System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER},
    },
};

use crate::printing::{
    document::{DocumentBuilder, DocumentBuildingError, PageBuilder, Saveable},
    xps::page::PAGE_SIZE_A4,
};

use super::page::XPSPage;

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
    pub fn new() -> Result<Self, windows::core::Error> {
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

    fn prepare_document(&mut self) {
        let part_uri_result = unsafe {
            self.factory
                .CreatePartUri(w!("/FixedDocumentSequence.fdseq"))
        };

        let Ok(part_uri) = part_uri_result else {
            error!(
                "couldn't create part uri: {:?}",
                part_uri_result.unwrap_err()
            );
            return;
        };

        let doc_seq_result = unsafe { self.factory.CreateDocumentSequence(&part_uri) };
        self.fixed_sequence_part = Some(part_uri);

        let Ok(doc_seq) = doc_seq_result else {
            error!(
                "couldn't create document sequence: {:?}",
                doc_seq_result.unwrap_err()
            );
            return;
        };

        let doc_seq_result = unsafe { self.package.SetDocumentSequence(&doc_seq) };

        let Ok(_) = doc_seq_result else {
            error!(
                "couldn't set document sequence: {:?}",
                doc_seq_result.unwrap_err()
            );
            return;
        };
        self.document_sequence = Some(doc_seq);

        let part_uri_result = unsafe {
            self.factory
                .CreatePartUri(w!("/Documents/1/FixedDocument.fdoc"))
        };

        let Ok(part_uri) = part_uri_result else {
            error!(
                "couldn't create document part uri: {:?}",
                part_uri_result.unwrap_err()
            );
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
}

impl DocumentBuilder for XPSSingleDocument {
    fn begin(&mut self) -> Result<(), DocumentBuildingError> {
        self.prepare_document();
        return Ok(());
    }

    fn page_at(&mut self, index: usize) -> Option<&mut dyn PageBuilder> {
        return self
            .pages
            .get_mut(index)
            .map(|page| page as &mut dyn PageBuilder);
    }

    fn new_page(&mut self) -> Result<usize, DocumentBuildingError> {
        assert!(self.document_fd.is_some());

        let part_uri_result = unsafe {
            self.factory.CreatePartUri(&HSTRING::from(format!(
                "/Documents/1/Pages/{}.fpage",
                self.pages.len() + 1
            )))
        };

        let Ok(part_uri) = part_uri_result else {
            let e = part_uri_result.unwrap_err();
            error!("couldn't create page part uri: {:?}", e);
            return Err(DocumentBuildingError::Error(e.to_string()));
        };

        let page_result = unsafe {
            self.factory
                .CreatePage(&PAGE_SIZE_A4, w!("de-DE"), &part_uri)
        };

        let Ok(page) = page_result else {
            let e = page_result.unwrap_err();
            error!("couldn't create page: {:?}", e);
            return Err(DocumentBuildingError::Error(e.to_string()));
        };

        let page_ref_result = unsafe { self.factory.CreatePageReference(&PAGE_SIZE_A4) };

        let Ok(xpsPageRef) = page_ref_result else {
            let e = page_ref_result.unwrap_err();
            error!("couldn't create page reference: {:?}", e);
            return Err(DocumentBuildingError::Error((e).to_string()));
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

        let page = XPSPage::new(Arc::clone(&self.factory), page);
        let Ok(page) = page else {
            error!("couldn't create page instance!");
            return Err(DocumentBuildingError::Error(
                "couldn't create page instance!".to_string(),
            ));
        };

        self.pages.push(page);
        return Ok(self.pages.len() - 1);
    }
}

impl Saveable for XPSSingleDocument {
    fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        const FILE_ATTRIBUTE_NORMAL: u32 = 0x00000080; // for some reason this isn't defined in the winapi rust wrapper
        let res = unsafe {
            self.package.WriteToFile(
                &HSTRING::from(path.to_str().unwrap()),
                std::ptr::null(),
                FILE_ATTRIBUTE_NORMAL,
                false,
            )
        }
        .map_err(|e| {
            Box::new(DocumentBuildingError::NestedError(Box::new(e))) as Box<dyn std::error::Error>
        });
        return res;
    }
}
