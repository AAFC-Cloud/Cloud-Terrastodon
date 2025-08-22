use eyre::bail;
use windows::Win32::Security::Credentials::CRED_TYPE_GENERIC;
use windows::Win32::Security::Credentials::CREDENTIALA;
use windows::Win32::Security::Credentials::CredFree;
use windows::Win32::Security::Credentials::CredReadA;

use crate::AzureDevOpsPersonalAccessToken;

/// Read a specific credential from Windows Credential Manager
pub fn read_credential_from_manager(target_name: &str) -> eyre::Result<Option<String>> {
    let target_name_cstr = std::ffi::CString::new(target_name)?;
    let mut credential_ptr: *mut CREDENTIALA = std::ptr::null_mut();

    let result = unsafe {
        CredReadA(
            windows::core::PCSTR(target_name_cstr.as_ptr() as *const u8),
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

/// Read credentials from Windows Credential Manager
pub fn read_azure_devops_pat_from_credential_manager()
-> eyre::Result<AzureDevOpsPersonalAccessToken> {
    match read_credential_from_manager("azdevops-cli: default") {
        Ok(Some(credential)) => Ok(AzureDevOpsPersonalAccessToken::new(credential)),
        Ok(None) => {
            bail!("No default credential found");
        }
        Err(e) => {
            bail!("Failed to read default credential: {}", e);
        }
    }
}