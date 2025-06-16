use crate::TransferProfile;
use anyhow::Result;

pub trait TransferProtocolHandler {
    fn send(&self, profile: &TransferProfile) -> Result<()>;
    fn receive(&self, profile: &TransferProfile) -> Result<()>;
}
