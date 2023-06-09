#[derive(Debug)]
pub struct RadioIdentifier {
    pub org: String,
    pub county: String,
    pub agency: u64,
    pub car_type: u64,
    pub number: u64,
}
