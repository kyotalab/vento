use crate::{
    connect_session_and_authenticate, transfer::protocol::TransferProtocolHandler, TransferProfile
};
use anyhow::{Context, Result};
use log::info;
use ssh2::Session;
use std::{
    fs::{read_to_string, File}, io::{Read, Write}, path::{Path, PathBuf}
};

pub struct ScpHandler;

#[async_trait::async_trait]
impl TransferProtocolHandler for ScpHandler {
    async fn send(&self, profile: &TransferProfile) -> Result<()> {
        let protocol = profile.transfer_protocol.protocol.to_string();
        info!(
            "Attempting to send file from '{}' to {:?} destination '{}'@{}:{}{}",
            profile.source.path,
            profile.transfer_protocol.protocol,
            profile
                .destination
                .authentication
                .as_ref()
                .map_or("unknown", |a| a.username.as_str()),
            profile.destination.host.as_deref().unwrap_or("localhost"),
            profile.destination.port.unwrap_or(22),
            profile.destination.path
        );
        let session = connect_session_and_authenticate(
           &protocol, 
            profile.destination.authentication.as_ref(),
            profile.destination.host.as_deref(),
            profile.destination.port,
        )?;

        transfer_file_scp(
            &session,
            &profile.source.path.clone().into(),
            &profile.destination.path.clone().into(),
            true)

    }

    async fn receive(&self, profile: &TransferProfile) -> Result<()> {
        let protocol = profile.transfer_protocol.protocol.to_string();
        info!(
            "Attempting to receive file from {} source '{}'@{}:{}{} to local '{}'",
            &protocol,
            profile
                .source
                .authentication
                .as_ref()
                .map_or("unknown", |a| a.username.as_str()),
            profile.source.host.as_deref().unwrap_or("localhost"),
            profile.source.port.unwrap_or(22),
            profile.source.path,
            profile.destination.path
        );

        let session = connect_session_and_authenticate(
            &protocol,
            profile.source.authentication.as_ref(),
            profile.source.host.as_deref(),
            profile.source.port,
        )?;

        transfer_file_scp(
            &session,
            &profile.source.path.clone().into(),
            &profile.destination.path.clone().into(),
            false
        )
    }
}

fn transfer_file_scp(session: &Session, src: &PathBuf, dst: &PathBuf, upload: bool) -> Result<()>{
    if upload {
        info!(
            "Attempting to upload file from '{}' to remote path '{}'",
            src.display(),
            dst.display()
        );
        let local_file_content = read_to_string(src).with_context(|| {
            format!(
            "failed to read local source file for upload: '{}'",
                src.display()
            )
        })?;
        let content_bytes = local_file_content.as_str().as_bytes();
        let mut remote_file = session.scp_send(Path::new(dst), 0o644, content_bytes.len() as u64, None).with_context(|| {
            format!(
            "Failed to create remote destination file for upload: '{}'",
                dst.display()
            )
        })?;
        // Permissions on sent files are set to 0o644 (owner: read/write, group: read, other: read).
        // The file transfer timeout is set to 10 seconds.
        // No special callback processing is performed during file transfer.
        remote_file.write_all(&content_bytes).with_context(|| {
            format!(
            "Failed to copy data during upload from '{}' to '{}'",
                src.display(),
                dst.display()
            )
        })?;
        // Close the channel and wait for the whole content to be transferred
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;
        info!(
            "Successfully uploaded file from '{}' to '{}'",
            src.display(),
            dst.display()
        );
    } else {
        info!(
            "Attempting to download file from remote path '{}' to local path '{}'",
            src.display(),
            dst.display()
        );
        let (mut remote_file, _) = session.scp_recv(Path::new(src)).with_context(|| {
            format!(
                "Failed to open remote source file for download: '{}'",
                src.display()
            )
        })?;
        let mut contents = Vec::new();
        remote_file.read_to_end(&mut contents).with_context(|| {
            format!(
                "Failed to read remote file for download: '{}'",
                src.display()
            )
        })?;
        // Close the channel and wait for the whole content to be transferred
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;

        // Write contents to local file
        let mut file = File::create(dst)?;
        file.write_all(&contents).with_context(|| {
            format!("Failed to copy data during download from '{}' to '{}'",
                src.display(),
                dst.display()
            )
        })?;
        info!(
            "Successfully downloaded file from '{}' to '{}'",
            src.display(),
            dst.display()
        );
    }

    Ok(())
}
