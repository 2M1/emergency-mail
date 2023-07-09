use log::{error, trace};
use windows::{
    core::HSTRING,
    Win32::{
        Storage::Xps::{
            IXpsOMFontResource, IXpsOMObjectFactory,
            IXpsOMSolidColorBrush, XPS_COLOR, XPS_COLOR_0_1, XPS_COLOR_TYPE_SRGB,
            XPS_FONT_EMBEDDING_OBFUSCATED,
        },
        System::Com::{CoCreateGuid, StringFromGUID2},
    },
};

pub struct XPSHelper;

impl XPSHelper {
    pub fn load_font(
        factory: &IXpsOMObjectFactory,
        font_name: &str,
    ) -> Result<IXpsOMFontResource, ()> {
        let in_stream = unsafe {
            factory.CreateReadOnlyStreamOnFile(&HSTRING::from(format!(
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
        let mut guid_string: Vec<u16> = vec![0; 39]; // should be long enough, common guid length is 36 + \0 + {}
        let _ = unsafe { StringFromGUID2(&guid, &mut guid_string) };
        let guid_str = String::from_utf16(&guid_string).unwrap();
        let uri_str = format!(
            "/Resources/Fonts/{}.odttf",
            &guid_str[1..guid_str.len() - 2] // strip { and }\0 from guid string
        );
        trace!("uri_str: {:?}", uri_str);
        let uri_str = HSTRING::from(uri_str);
        println!("guid: {:?}", guid_string);

        let part_uri = unsafe { factory.CreatePartUri(&uri_str) };
        let Ok(part_uri) = part_uri else {
            error!("couldn't create font part uri: {:?}", part_uri.unwrap_err());
            return Err(());
        };

        let font_resource = unsafe {
            factory.CreateFontResource(&in_stream, XPS_FONT_EMBEDDING_OBFUSCATED, &part_uri, false)
        };
        let Ok(font_resource) = font_resource else {
            error!("couldn't create font resource: {:?}", font_resource.unwrap_err());
            return Err(());
        };

        return Ok(font_resource);
    }

    pub fn create_colour_brush(
        factory: &IXpsOMObjectFactory,
        r: u8,
        g: u8,
        b: u8,
    ) -> Result<IXpsOMSolidColorBrush, ()> {
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

        let colour = unsafe { factory.CreateSolidColorBrush(&xps_colour, None) };
        let Ok(colour_brush) = colour else {
            error!("couldn't create brush: {:?}", colour.unwrap_err());
            return Err(());
        };

        return Ok(colour_brush);
    }
}
