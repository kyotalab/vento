use crate::TransferProfile;
use anyhow::Result;

#[async_trait::async_trait]
pub trait TransferProtocolHandler {
    async fn send(&self, profile: &TransferProfile) -> Result<()>;
    async fn receive(&self, profile: &TransferProfile) -> Result<()>;
}
