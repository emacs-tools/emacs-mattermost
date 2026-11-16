use mm_core::BridgeStatus;

#[derive(Debug, Clone)]
pub struct WebState {
    pub status: BridgeStatus,
}

impl WebState {
    pub fn new(status: BridgeStatus) -> Self {
        Self { status }
    }
}
