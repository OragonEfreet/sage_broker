#[derive(Debug, PartialEq)]
pub enum PeerState {
    New,
    Active,
    Closed,
}
