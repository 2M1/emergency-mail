pub struct Fetch {
    pub uid: Option<u32>,
    pub body: Option<Vec<u8>>,
    pub content_type: Option<String>,
}

impl Fetch {
    pub fn text(&self) -> Option<&[u8]> {
        return self.body.as_ref().map(|b| b.as_slice());
    }

    pub fn header(&self) -> Option<&[u8]> {
        return self.body.as_ref().map(|b| b.as_slice());
    }
}
