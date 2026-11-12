use crate::AzureDevOpsPersonalAccessToken;
use crate::WindowsCredentialManagerTargetName;
use eyre::Context;
use eyre::bail;
use windows::Win32::Foundation::ERROR_NOT_FOUND;
use windows::Win32::Security::Credentials::CRED_MAX_CREDENTIAL_BLOB_SIZE;
use windows::Win32::Security::Credentials::CRED_PERSIST_LOCAL_MACHINE;
use windows::Win32::Security::Credentials::CRED_TYPE_GENERIC;
use windows::Win32::Security::Credentials::CREDENTIALA;
use windows::Win32::Security::Credentials::CredDeleteA;
use windows::Win32::Security::Credentials::CredFree;
use windows::Win32::Security::Credentials::CredReadA;
use windows::Win32::Security::Credentials::CredWriteA;

/// Read a specific credential from Windows Credential Manager
pub fn read_credential_from_manager(
    target_name: &WindowsCredentialManagerTargetName,
) -> eyre::Result<Option<String>> {
    let mut credential_ptr: *mut CREDENTIALA = std::ptr::null_mut();

    let result = unsafe {
        CredReadA(
            windows::core::PCSTR(target_name.as_c_str().as_ptr() as *const u8),
            CRED_TYPE_GENERIC,
            None, // Flags parameter
            &mut credential_ptr,
        )
    };

    if let Err(error) = result {
        if error.code() == ERROR_NOT_FOUND.to_hresult() {
            return Ok(None);
        }
        return Err(error).wrap_err("reading the selected Windows credential");
    }
    eyre::ensure!(
        !credential_ptr.is_null(),
        "Windows returned an empty credential record"
    );
    // Copy the selected blob before releasing Windows' allocation. New CT
    // credentials are UTF-8; explicit legacy import also accepts UTF-16LE blobs.
    let bytes = unsafe {
        let credential = &*credential_ptr;
        let bytes = if credential.CredentialBlobSize == 0 {
            Vec::new()
        } else {
            std::slice::from_raw_parts(
                credential.CredentialBlob,
                credential.CredentialBlobSize as usize,
            )
            .to_vec()
        };
        CredFree(credential_ptr.cast());
        bytes
    };
    let password = if bytes.len().is_multiple_of(2)
        && bytes.iter().skip(1).step_by(2).all(|byte| *byte == 0)
    {
        String::from_utf16(
            &bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .collect::<Vec<_>>(),
        )
        .map_err(|_| eyre::eyre!("selected legacy credential has invalid text encoding"))?
    } else {
        String::from_utf8(bytes)
            .map_err(|_| eyre::eyre!("selected credential has invalid UTF-8 encoding"))?
    };
    Ok(Some(password))
}

/// Write a generic string credential to the current user's Windows Credential Manager.
///
/// `CRED_PERSIST_LOCAL_MACHINE` makes the credential survive logons without
/// replicating it to other machines. Windows protects the credential blob at
/// rest and scopes access to the current user's credential store.
pub fn write_credential_to_manager(
    target_name: &WindowsCredentialManagerTargetName,
    credential: &str,
) -> eyre::Result<()> {
    let credential_bytes = credential.as_bytes();
    let credential_blob_size = u32::try_from(credential_bytes.len())?;
    eyre::ensure!(
        credential_blob_size <= CRED_MAX_CREDENTIAL_BLOB_SIZE,
        "credential for {target_name:?} exceeds the Windows Credential Manager blob limit"
    );

    let credential_record = CREDENTIALA {
        Type: CRED_TYPE_GENERIC,
        TargetName: windows::core::PSTR(target_name.as_c_str().as_ptr() as *mut u8),
        CredentialBlobSize: credential_blob_size,
        CredentialBlob: credential_bytes.as_ptr() as *mut u8,
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        ..Default::default()
    };

    unsafe { CredWriteA(&credential_record, 0) }
        .map_err(|error| eyre::eyre!("writing credential {target_name:?}: {error}"))?;
    Ok(())
}

/// Delete a generic credential from the current user's Windows Credential Manager.
pub fn delete_credential_from_manager(
    target_name: &WindowsCredentialManagerTargetName,
) -> eyre::Result<()> {
    let result = unsafe {
        CredDeleteA(
            windows::core::PCSTR(target_name.as_c_str().as_ptr() as *const u8),
            CRED_TYPE_GENERIC,
            None,
        )
    };
    match result {
        Ok(()) => Ok(()),
        Err(error) if error.code() == ERROR_NOT_FOUND.to_hresult() => Ok(()),
        Err(error) => Err(error).wrap_err_with(|| format!("deleting credential {target_name:?}")),
    }
}

/// Read credentials from Windows Credential Manager
pub(crate) fn read_azure_devops_pat_from_credential_manager()
-> eyre::Result<AzureDevOpsPersonalAccessToken> {
    let target_name = WindowsCredentialManagerTargetName::try_new("azdevops-cli: default")?;
    match read_credential_from_manager(&target_name) {
        Ok(Some(credential)) => Ok(AzureDevOpsPersonalAccessToken::new(credential)),
        Ok(None) => {
            bail!("No default credential found");
        }
        Err(e) => {
            bail!("Failed to read default credential: {}", e);
        }
    }
}
