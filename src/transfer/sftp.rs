use crate::{
    connect_session_and_authenticate, transfer::protocol::TransferProtocolHandler, TransferProfile
};
use anyhow::{anyhow, Context, Result};
use log::info;
use ssh2::Sftp;
use std::{
    fs::File,
    io::copy,
    path::{Path, PathBuf},
};

pub struct SftpHandler;

#[async_trait::async_trait]
impl TransferProtocolHandler for SftpHandler {
    async fn send(&self, profile: &TransferProfile) -> Result<()> {
        let protocol = profile.transfer_protocol.protocol.to_string();
        info!(
            "Attempting to send file from '{}' to SFTP destination '{}'@{}:{}{}",
            profile.source.path,
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

        let sftp = session.sftp()?;

        transfer_file_sftp(
            &sftp,
            &profile.source.path.clone().into(),
            &profile.destination.path.clone().into(),
            true, // upload
        )
    }

    async fn receive(&self, profile: &TransferProfile) -> Result<()> {
        let protocol = profile.transfer_protocol.protocol.to_string();
        info!(
            "Attempting to receive file from SFTP source '{}'@{}:{}{} to local '{}'",
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

        let sftp = session.sftp()?;

        transfer_file_sftp(
            &sftp,
            &profile.source.path.clone().into(),
            &profile.destination.path.clone().into(),
            false, // download
        )
    }
}

fn transfer_file_sftp(sftp: &Sftp, src: &PathBuf, dst: &PathBuf, upload: bool) -> Result<()> {
    use crate::MAX_FILE_SIZE_MB;

    let max_mb = *MAX_FILE_SIZE_MB.read().unwrap();
    if upload {
        info!(
            "Attempting to upload file from '{}' to remote path '{}'",
            src.display(),
            dst.display()
        );
        let mut local_file = File::open(src).with_context(|| {
            format!(
                "Failed to open local source file for upload: '{}'",
                src.display()
            )
        })?;

        let metadata = local_file.metadata()?;
        let file_size = metadata.len();
        let max_size_bytes = max_mb * 1024 * 1024;
        if file_size > max_size_bytes {
            return Err(anyhow!(
                "File '{}' exceeds max allowed size ({} MB)",
                src.display(),
                max_size_bytes / 1024 / 1024
            ));
        }

        let mut remote_file = sftp.create(Path::new(dst)).with_context(|| {
            format!(
                "Failed to create remote destination file for upload: '{}'",
                dst.display()
            )
        })?;
        copy(&mut local_file, &mut remote_file).with_context(|| {
            format!(
                "Failed to copy data during upload from '{}' to '{}'",
                src.display(),
                dst.display()
            )
        })?;
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
        let mut remote_file = sftp.open(Path::new(src)).with_context(|| {
            format!(
                "Failed to open remote source file for download: '{}'",
                src.display()
            )
        })?;

        let stat = remote_file.stat()?;
        let file_size = stat.size.ok_or_else(|| anyhow!("Unable to get size of remote file"))?;
        let max_size_bytes = max_mb * 1024 * 1024;
        if file_size > max_size_bytes {
            return Err(anyhow!(
                "File '{}' exceeds max allowed size ({} MB)",
                src.display(),
                max_size_bytes / 1024 / 1024
            ));
        }

        let mut local_file = File::create(dst).with_context(|| {
            format!(
                "Failed to create local destination file for download: '{}'",
                dst.display()
            )
        })?;
        copy(&mut remote_file, &mut local_file).with_context(|| {
            format!(
                "Failed to copy data during download from '{}' to '{}'",
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
