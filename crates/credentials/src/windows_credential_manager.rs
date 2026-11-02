use crate::AzureDevOpsPersonalAccessToken;
use crate::WindowsCredentialManagerTargetName;
use crate::auth_logout_error::AuthLogoutError;
use eyre::Context;
use eyre::bail;
use windows::Win32::Foundation::ERROR_NOT_FOUND;
use windows::Win32::Security::Credentials::CRED_MAX_CREDENTIAL_BLOB_SIZE;
use windows::Win32::Security::Credentials::CRED_PERSIST_LOCAL_MACHINE;
use windows::Win32::Security::Credentials::CRED_TYPE_GENERIC;
use windows::Win32::Security::Credentials::CREDENTIALA;
use windows::Win32::Security::Credentials::CredDeleteA;
use windows::Win32::Security::Credentials::CredEnumerateA;
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

    if result.is_ok() && !credential_ptr.is_null() {
        let credential = unsafe { &*credential_ptr };

        // Extract the credential blob (password)
        let password_bytes = unsafe {
            std::slice::from_raw_parts(
                credential.CredentialBlob,
                credential.CredentialBlobSize as usize,
            )
        };

        let password = String::from_utf8_lossy(password_bytes)
            .chars()
            .filter(|c| *c != '\0') // this is ugly hack and idk why the null bytes are there in the first place D:
            .collect::<String>();

        // Free the credential
        unsafe {
            CredFree(credential_ptr as *mut std::ffi::c_void);
        }

        Ok(Some(password))
    } else {
        Ok(None)
    }
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

/// Delete all generic credentials with an application-owned prefix, including
/// tokens left behind when the persisted session switched tenants or clients.
/// Uses the prefix filter from
/// https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credenumeratea.
pub(crate) fn delete_credentials_with_prefix(prefix: &str) -> eyre::Result<()> {
    let filter = WindowsCredentialManagerTargetName::try_new(format!("{prefix}*"))?;
    let mut count = 0;
    let mut credentials: *mut *mut CREDENTIALA = std::ptr::null_mut();
    // SAFETY: The filter is NUL-terminated and both output pointers are valid.
    let result = unsafe {
        CredEnumerateA(
            windows::core::PCSTR(filter.as_c_str().as_ptr().cast()),
            None,
            &mut count,
            &mut credentials,
        )
    };
    if let Err(error) = result {
        if error.code() == ERROR_NOT_FOUND.to_hresult() {
            return Ok(());
        }
        return Err(error).wrap_err("enumerating stored Cloud Terrastodon refresh tokens");
    }
    // SAFETY: Successful enumeration provides count records in a single Windows
    // allocation. Copy target names before releasing that allocation with CredFree.
    // No credential blobs are read. Collection retains errors until after freeing.
    let targets = unsafe {
        std::slice::from_raw_parts(credentials, count as usize)
            .iter()
            .filter(|&&credential| (*credential).Type == CRED_TYPE_GENERIC)
            .map(|credential| {
                (**credential)
                    .TargetName
                    .to_string()
                    .wrap_err("reading a stored refresh-token target name")
                    .and_then(WindowsCredentialManagerTargetName::try_new)
            })
            .collect::<Vec<_>>()
    };
    // SAFETY: credentials is the allocation returned by successful enumeration;
    // all references to its records have ended and it is freed exactly once.
    unsafe { CredFree(credentials.cast()) };
    let mut errors = Vec::new();
    for target in targets {
        if let Err(error) = target.and_then(|target| delete_credential_from_manager(&target)) {
            errors.push(error);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(AuthLogoutError { errors }.into())
    }
}

/// Read credentials from Windows Credential Manager
pub fn read_azure_devops_pat_from_credential_manager()
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
