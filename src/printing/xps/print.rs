use log::{error, info, trace};
use windows::{
    w,
    Win32::{
        Storage::Xps::{
            Printing::{
                IXpsPrintJob, IXpsPrintJobStream, StartXpsPrintJob, XPS_JOB_CANCELLED,
                XPS_JOB_STATUS,
            },
            XPS_INTERLEAVING_ON,
        },
        System::Threading::{CreateEventA, WaitForSingleObject, INFINITE},
    },
};

use crate::printing::xps::page::PAGE_SIZE_A4;

use super::document::XPSSingleDocument;

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
    let _ticket_stream: Option<IXpsPrintJobStream> = None;
    // start print job.
    match unsafe {
        StartXpsPrintJob(
            w!("HPE76479 (HP OfficeJet Pro 8020 series)"),
            w!("emegency mail print job"),
            None,
            None,
            completionEvent,
            &[0u8],
            &mut job,
            &mut stream,
            std::ptr::null_mut(),
        )
    } {
        Ok(_) => {}
        Err(e) => {
            error!("couldn't start print job: {}", e.message());
            return;
        }
    }

    // let res = unsafe { package.WriteToStream(stream.as_ref().unwrap(), false) };
    // if let Err(e) = res {
    //     error!("couldn't write package to stream: {}", e.message());
    //     return;
    // }

    let part_uri_result = unsafe {
        doc.factory
            .CreatePartUri(w!("/FixedDocumentSequence.fdseq"))
    };

    let Ok(part_uri) = part_uri_result else {
        error!("couldn't create document part uri: {:?}", part_uri_result.unwrap_err());
        return;
    };

    let package_writer_res = unsafe {
        doc.factory.CreatePackageWriterOnStream(
            stream.as_ref().unwrap(),
            true,
            XPS_INTERLEAVING_ON,
            &part_uri,
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

    let part_uri_result = unsafe {
        doc.factory
            .CreatePartUri(w!("/Documents/1/FixedDocument.fdoc"))
    };

    let Ok(part_uri) = part_uri_result else {
        error!("couldn't create document part uri: {:?}", part_uri_result.unwrap_err());
        return;
    };

    let wirter_res = unsafe { package_writer.StartNewDocument(&part_uri, None, None, None, None) };
    if let Err(e) = wirter_res {
        error!("couldn't start new document: {}", e.message());
        return;
    }
    info!("started new document");

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
    let mut job_status: XPS_JOB_STATUS = unsafe { std::mem::zeroed() };
    let job_res = unsafe { job.unwrap().GetJobStatus(&mut job_status) };
    if let Err(e) = job_res {
        error!("couldn't get job status: {}", e.message());
        return;
    } else {
        info!("job status: {:?}", job_status);
        info!("job status msg: {:?}", job_status.jobStatus);
        if job_status.completion != XPS_JOB_CANCELLED {
            error!("print job failed");
        }
    }
}
