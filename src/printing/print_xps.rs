use log::{error, info, trace};
use windows::{
    w,
    Win32::{
        Storage::Xps::{
            Printing::{IXpsPrintJob, IXpsPrintJobStream, StartXpsPrintJob},
            XPS_INTERLEAVING_ON,
        },
        System::Threading::{CreateEventA, WaitForSingleObject, INFINITE},
    },
};

use crate::printing::xps_page::PAGE_SIZE_A4;

use super::xps_document::XPSSingleDocument;

pub fn print_test(doc: XPSSingleDocument) {
    // @pre: COM is initialized.

    let completionEvent_res = unsafe { CreateEventA(None, true, false, None) };

    let Ok(completionEvent) = completionEvent_res else {
        error!(
            "couldn't create completion event: {}",
            completionEvent_res.unwrap_err().message()
        );
        return;
    };

    let mut job: Option<IXpsPrintJob> = None;
    let mut stream: Option<IXpsPrintJobStream> = None;
    let mut ticket_stream: Option<IXpsPrintJobStream> = None;
    // start print job.
    match unsafe {
        StartXpsPrintJob(
            w!("HPE76479 (HP OfficeJet Pro 8020 series)"),
            w!("test"),
            None,
            None,
            completionEvent,
            &[0u8],
            &mut job,
            &mut stream,
            &mut ticket_stream,
        )
    } {
        Ok(_) => {}
        Err(e) => {
            error!("couldn't start print job: {}", e.message());
            return;
        }
    }

    let package_writer_res = unsafe {
        doc.factory.CreatePackageWriterOnStream(
            stream.as_ref().unwrap(),
            true,
            XPS_INTERLEAVING_ON,
            doc.fixed_sequence_part.as_ref().unwrap(),
            None,
            None,
            None,
            None,
        )
    };

    let Ok(package_writer) = package_writer_res else {
        error!(
            "couldn't create package writer on stream: {}",
            package_writer_res.unwrap_err().message()
        );
        return;
    };

    let wirter_res = unsafe {
        package_writer.StartNewDocument(doc.document_part.as_ref().unwrap(), None, None, None, None)
    };
    if let Err(e) = wirter_res {
        error!("couldn't start new document: {}", e.message());
        return;
    }

    for page in doc.pages.iter() {
        let res =
            unsafe { package_writer.AddPage(&page.page, &PAGE_SIZE_A4, None, None, None, None) };
        if let Err(e) = res {
            error!("couldn't add page: {}", e.message());
            return;
        }
        info!("added page");
    }

    let writer_res = unsafe { package_writer.Close() };
    if let Err(e) = writer_res {
        error!(
            "couldn't close package writer: {}, {:?}",
            e.message(),
            e.code()
        );
        return;
    }

    let job_res = unsafe { stream.unwrap().Close() };
    if let Err(e) = job_res {
        error!("couldn't close job stream: {}", e.message());
        return;
    }

    trace!("waiting for completion event");
    let waiting_res = unsafe { WaitForSingleObject(completionEvent, INFINITE) };
    if waiting_res.is_err() {
        error!(
            "couldn't wait for completion event: {}",
            waiting_res.to_hresult().message()
        );
        return;
    }
    info!("print job completed");
}
