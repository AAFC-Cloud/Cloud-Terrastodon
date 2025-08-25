// use chrono::{DateTime, Utc};
// use eyre::eyre;
// use facet::Facet;
// use facet_json::from_str;
// use std::collections::HashMap;
// use windows::Win32::Security::Cryptography::CRYPT_INTEGER_BLOB;
// use windows::Win32::Security::Cryptography::CRYPTPROTECT_UI_FORBIDDEN;
// use windows::Win32::Security::Cryptography::CryptUnprotectData;

// /// Represents the complete MSAL token cache structure
// #[derive(Facet, Debug)]
// pub struct MsalTokenCache {
//     #[facet(rename = "AccessToken")]
//     pub access_tokens: HashMap<String, AccessToken>,
//     #[facet(rename = "Account")]
//     pub accounts: HashMap<String, Account>,
//     #[facet(rename = "IdToken")]
//     pub id_tokens: HashMap<String, IdToken>,
//     #[facet(rename = "AppMetadata")]
//     pub app_metadata: HashMap<String, AppMetadata>,
// }

// /// Represents an access token entry
// #[derive(Facet, Debug)]
// pub struct AccessToken {
//     pub credential_type: String,
//     #[facet(sensitive)]
//     pub secret: String,
//     pub home_account_id: String,
//     pub environment: String,
//     pub client_id: String,
//     pub target: String,
//     pub realm: String,
//     pub token_type: String,
//     /// Unix timestamp when the token was cached
//     pub cached_at: String,
//     /// Unix timestamp when the token expires
//     pub expires_on: String,
//     /// Unix timestamp for extended expiration
//     pub extended_expires_on: String,
// }

// /// Represents an account entry
// #[derive(Facet, Debug)]
// pub struct Account {
//     pub home_account_id: String,
//     pub environment: String,
//     pub realm: String,
//     pub local_account_id: String,
//     pub username: String,
//     pub authority_type: String,
//     pub account_source: String,
// }

// /// Represents an ID token entry
// #[derive(Facet, Debug)]
// pub struct IdToken {
//     pub credential_type: String,
//     #[facet(sensitive)]
//     pub secret: String,
//     pub home_account_id: String,
//     pub environment: String,
//     pub realm: String,
//     pub client_id: String,
// }

// /// Represents app metadata
// #[derive(Facet, Debug)]
// pub struct AppMetadata {
//     pub client_id: String,
//     pub environment: String,
// }

// impl AccessToken {
//     /// Parse the cached_at timestamp as a DateTime
//     pub fn cached_at_datetime(&self) -> eyre::Result<DateTime<Utc>> {
//         let timestamp = self.cached_at.parse::<i64>()?;
//         Ok(DateTime::from_timestamp(timestamp, 0).unwrap_or_default())
//     }

//     /// Parse the expires_on timestamp as a DateTime
//     pub fn expires_on_datetime(&self) -> eyre::Result<DateTime<Utc>> {
//         let timestamp = self.expires_on.parse::<i64>()?;
//         Ok(DateTime::from_timestamp(timestamp, 0).unwrap_or_default())
//     }

//     /// Parse the extended_expires_on timestamp as a DateTime
//     pub fn extended_expires_on_datetime(&self) -> eyre::Result<DateTime<Utc>> {
//         let timestamp = self.extended_expires_on.parse::<i64>()?;
//         Ok(DateTime::from_timestamp(timestamp, 0).unwrap_or_default())
//     }

//     /// Check if the token is expired
//     pub fn is_expired(&self) -> bool {
//         if let Ok(expires_on) = self.expires_on_datetime() {
//             expires_on < Utc::now()
//         } else {
//             true // Consider invalid timestamps as expired
//         }
//     }
// }

// /// Decrypt the MSAL token cache bytes
// ///
// /// This function implements the decryption logic to handle the Windows Data Protection API (DPAPI) encrypted cache file
// pub fn decrypt_msal_token_cache(cache_bytes: &[u8], global_args: &GlobalArgs) -> eyre::Result<()> {
//     tracing::info!(
//         "Attempting to decrypt MSAL token cache ({} bytes)",
//         cache_bytes.len()
//     );

//     // Create input data blob for DPAPI
//     let input_blob = CRYPT_INTEGER_BLOB {
//         cbData: cache_bytes.len() as u32,
//         pbData: cache_bytes.as_ptr() as *mut u8,
//     };

//     let mut output_blob = CRYPT_INTEGER_BLOB::default();

//     // Call Windows DPAPI to decrypt the data
//     let result = unsafe {
//         CryptUnprotectData(
//             &input_blob,               // Input encrypted data
//             None,                      // Description (not needed)
//             None,                      // Optional entropy
//             None,                      // Reserved
//             None,                      // Prompt struct
//             CRYPTPROTECT_UI_FORBIDDEN, // Flags
//             &mut output_blob,          // Output decrypted data
//         )
//     };

//     if result.is_err() {
//         return Err(eyre::eyre!(
//             "Failed to decrypt MSAL token cache: DPAPI decryption failed"
//         ));
//     }

//     // Extract the decrypted data
//     let decrypted_data =
//         unsafe { std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize) };

//     tracing::info!("Successfully decrypted {} bytes", decrypted_data.len());

//     // Convert to string and handle sensitive data display
//     match std::str::from_utf8(decrypted_data) {
//         Ok(json_str) => {
//             if global_args.show_sensitive {
//                 println!("Decrypted token cache content:\n{}", json_str);
//             } else {
//                 println!(
//                     "Decrypted token cache content: <SENSITIVE DATA - use --show-sensitive to view>"
//                 );
//                 tracing::debug!("Content length: {} characters", json_str.len());

//                 // Try to parse as MSAL token cache structure
//                 match from_str::<MsalTokenCache>(json_str) {
//                     Ok(token_cache) => {
//                         display_token_cache_summary(&token_cache);
//                     }
//                     Err(e) => {
//                         tracing::warn!("Failed to parse token cache as MSAL structure: {}", e);
//                         println!("Raw JSON structure (first 500 chars): {}",
//                             &json_str[..json_str.len().min(500)]);
//                     }
//                 }
//             }
//         }
//         Err(_) => {
//             tracing::warn!("Decrypted data is not valid UTF-8, showing as hex dump");
//             if global_args.show_sensitive {
//                 println!(
//                     "Decrypted binary data (hex): {}",
//                     hex::encode(decrypted_data)
//                 );
//             } else {
//                 println!(
//                     "Decrypted binary data: <SENSITIVE DATA - use --show-sensitive to view>"
//                 );
//                 tracing::debug!("Binary data length: {} bytes", decrypted_data.len());
//             }
//         }
//     }

//     // Free the allocated memory from DPAPI
//     // Note: In practice, Windows manages this memory automatically
//     // unsafe {
//     //     windows::Win32::System::Memory::LocalFree(HLOCAL(output_blob.pbData as _));
//     // }

//     Ok(())
// }

// /// Decrypt MSAL token cache and return the JSON string
// fn decrypt_msal_cache_to_string(cache_bytes: &[u8]) -> eyre::Result<String> {
//     tracing::debug!(
//         "Attempting to decrypt MSAL token cache ({} bytes)",
//         cache_bytes.len()
//     );

//     // Create input data blob for DPAPI
//     let input_blob = CRYPT_INTEGER_BLOB {
//         cbData: cache_bytes.len() as u32,
//         pbData: cache_bytes.as_ptr() as *mut u8,
//     };

//     let mut output_blob = CRYPT_INTEGER_BLOB::default();

//     // Call Windows DPAPI to decrypt the data
//     let result = unsafe {
//         CryptUnprotectData(
//             &input_blob,               // Input encrypted data
//             None,                      // Description (not needed)
//             None,                      // Optional entropy
//             None,                      // Reserved
//             None,                      // Prompt struct
//             CRYPTPROTECT_UI_FORBIDDEN, // Flags
//             &mut output_blob,          // Output decrypted data
//         )
//     };

//     if result.is_err() {
//         return Err(eyre::eyre!(
//             "Failed to decrypt MSAL token cache: DPAPI decryption failed"
//         ));
//     }

//     // Extract the decrypted data
//     let decrypted_data =
//         unsafe { std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize) };

//     // Convert to string
//     match std::str::from_utf8(decrypted_data) {
//         Ok(json_str) => Ok(json_str.to_string()),
//         Err(_) => Err(eyre::eyre!("Decrypted data is not valid UTF-8")),
//     }
// }

// /// Display a summary of the token cache without sensitive information
// fn display_token_cache_summary(cache: &MsalTokenCache) {
//     println!("\n=== MSAL Token Cache Summary ===");

//     // Access Tokens Summary
//     println!("\n📋 Access Tokens: {}", cache.access_tokens.len());
//     for (key, token) in &cache.access_tokens {
//         println!("  🔑 Object Key (not the token): {}", truncate_key(key));
//         println!("     Client ID: {}", token.client_id);
//         println!("     Environment: {}", token.environment);
//         println!("     Target: {}", truncate_target(&token.target));
//         println!("     Token Type: {}", token.token_type);

//         // Parse and display timestamps
//         if let Ok(cached_at) = token.cached_at_datetime() {
//             println!("     Cached At: {}", cached_at.format("%Y-%m-%d %H:%M:%S UTC"));
//         }
//         if let Ok(expires_on) = token.expires_on_datetime() {
//             let status = if token.is_expired() { "❌ EXPIRED" } else { "✅ VALID" };
//             println!("     Expires On: {} UTC ({})", expires_on.format("%Y-%m-%d %H:%M:%S"), status);
//         }
//         println!();
//     }

//     // Accounts Summary
//     println!("\n👤 Accounts: {}", cache.accounts.len());
//     for (key, account) in &cache.accounts {
//         println!("  🔑 Object Key (not the token): {}", truncate_key(key));
//         println!("     Username: {}", account.username);
//         println!("     Environment: {}", account.environment);
//         println!("     Authority Type: {}", account.authority_type);
//         println!("     Account Source: {}", account.account_source);
//         println!();
//     }

//     // ID Tokens Summary
//     println!("\n🆔 ID Tokens: {}", cache.id_tokens.len());
//     for (key, token) in &cache.id_tokens {
//         println!("  🔑 Object Key (not the token): {}", truncate_key(key));
//         println!("     Client ID: {}", token.client_id);
//         println!("     Environment: {}", token.environment);
//         println!();
//     }

//     // App Metadata Summary
//     println!("\n📱 App Metadata: {}", cache.app_metadata.len());
//     for (key, metadata) in &cache.app_metadata {
//         println!("  🔑 Object Key (not the token): {}", truncate_key(key));
//         println!("     Client ID: {}", metadata.client_id);
//         println!("     Environment: {}", metadata.environment);
//         println!();
//     }
// }

// /// Truncate long keys for display
// fn truncate_key(key: &str) -> String {
//     if key.len() > 60 {
//         format!("{}...{}", &key[..30], &key[key.len()-15..])
//     } else {
//         key.to_string()
//     }
// }

// /// Truncate long target strings for display
// fn truncate_target(target: &str) -> String {
//     if target.len() > 80 {
//         format!("{}...", &target[..77])
//     } else {
//         target.to_string()
//     }
// }

// /// Test Azure credentials by making API calls to Microsoft Graph
// pub fn test_azure_credentials(cache_bytes: &[u8], global_args: &GlobalArgs) -> eyre::Result<()> {
//     // Decrypt the cache
//     let decrypted_data = decrypt_msal_cache_to_string(cache_bytes)?;

//     // Parse the decrypted JSON
//     let cache: MsalTokenCache = from_str(&decrypted_data).map_err(|e| eyre!("Failed to deserialize: {e:?}"))?;

//     // Find valid access tokens
//     let mut tested_tokens = 0;
//     let mut successful_tests = 0;

//     for (key, access_token) in &cache.access_tokens {
//         // Skip expired tokens
//         if let Ok(expires_on) = access_token.expires_on.parse::<i64>() {
//             let expiry_time = DateTime::from_timestamp(expires_on, 0)
//                 .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());
//             if expiry_time <= Utc::now() {
//                 tracing::debug!("Skipping expired token for key: {}", truncate_key(key));
//                 continue;
//             }
//         }

//         tested_tokens += 1;

//         println!("\n🧪 Testing token for:");
//         println!("   Target: {}", truncate_target(&access_token.target));
//         println!("   Environment: {}", access_token.environment);
//         println!("   Username: {}", get_username_for_token(&cache, &access_token.home_account_id));

//         // Test the token against Microsoft Graph /me endpoint
//         match test_microsoft_graph_token(&access_token.secret, global_args) {
//             Ok(user_info) => {
//                 successful_tests += 1;
//                 println!("   ✅ Success! User: {}", user_info.display_name.unwrap_or_else(|| user_info.user_principal_name.clone()));
//             }
//             Err(e) => {
//                 println!("   ❌ Failed: {}", e);
//             }
//         }
//     }

//     println!("\n📊 Test Summary:");
//     println!("   Tokens tested: {}", tested_tokens);
//     println!("   Successful: {}", successful_tests);
//     println!("   Failed: {}", tested_tokens - successful_tests);

//     if tested_tokens == 0 {
//         println!("   ℹ️  No valid tokens found to test");
//     }

//     Ok(())
// }

// /// Test a token against Microsoft Graph API
// fn test_microsoft_graph_token(token: &str, global_args: &GlobalArgs) -> eyre::Result<GraphUserInfo> {
//     let client = reqwest::blocking::Client::new();

//     let response = client
//         .get("https://graph.microsoft.com/v1.0/me")
//         .header("Authorization", format!("Bearer {}", token))
//         .header("Accept", "application/json")
//         .send()?;

//     if response.status().is_success() {
//         let json_text = response.text()?;

//         let user_info: GraphUserInfo = from_str(&json_text).map_err(|e| eyre!("Failed to deserialize: {e:?}"))?;
//         Ok(user_info)
//     } else {
//         let status = response.status();
//         let error_text = response.text()?;

//         if global_args.debug {
//             tracing::debug!("HTTP {}: {}", status, error_text);
//         }

//         Err(eyre::eyre!("HTTP {} - {}", status,
//             if error_text.len() > 100 {
//                 format!("{}...", &error_text[..97])
//             } else {
//                 error_text
//             }))
//     }
// }

// /// Microsoft Graph user information response
// #[derive(Facet, Debug)]
// struct GraphUserInfo {
//     #[facet(rename = "displayName")]
//     display_name: Option<String>,
//     #[facet(rename = "userPrincipalName")]
//     user_principal_name: String,
//     #[facet(rename = "id")]
//     id: String,
// }

// /// Get username for a token based on home account ID
// fn get_username_for_token(cache: &MsalTokenCache, home_account_id: &str) -> String {
//     cache.accounts
//         .values()
//         .find(|account| account.home_account_id == home_account_id)
//         .map(|account| account.username.clone())
//         .unwrap_or_else(|| format!("Unknown ({})", &home_account_id[..8]))
// }
