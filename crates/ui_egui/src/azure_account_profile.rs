use cloud_terrastodon_auth::CredentialAuthenticationProvider;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::EntraSignedInUserGetRequest;
use cloud_terrastodon_azure::EntraSignedInUserThumbnailGetRequest;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_azure::UserPrincipalName;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eframe::egui::ColorImage;
use eframe::egui::Context;
use eframe::egui::TextureHandle;
use eframe::egui::TextureOptions;
use eyre::Result;
use eyre::ensure;
use image::ImageFormat;
use image::ImageReader;
use image::Limits;
use std::io::Cursor;

/// Optional private identity details for a card in the desktop account picker.
#[derive(Default)]
pub(crate) struct AzureAccountProfile {
    // The presentation accessor protects the email in demo mode.
    email: Option<String>,
    pub photo: Option<TextureHandle>,
}

impl AzureAccountProfile {
    pub const REDACTED_EMAIL: &str = "redacted@redacted.ca";

    pub fn display_email(&self, demo_mode: bool) -> Option<&str> {
        self.email.as_deref().map(|email| {
            if demo_mode {
                Self::REDACTED_EMAIL
            } else {
                email
            }
        })
    }

    pub async fn load(
        executor: &CloudTerrastodonExecutor,
        authentication_provider: &CredentialAuthenticationProvider,
        authentication: AzureBearerAuthenticationSelection,
        expected_user: EntraUserId,
        ctx: &Context,
    ) -> Result<Self> {
        let executor = executor.with_authentication_lease(&authentication.into())?;
        let user = EntraSignedInUserGetRequest { authentication }
            .execute(&executor)
            .await
            .ok();
        let verified = user.is_some();
        let mut email = None;
        if let Some(user) = user {
            ensure!(
                user.id == expected_user,
                "Graph returned a different signed-in identity"
            );
            email = user
                .mail
                .filter(|mail| !mail.trim().is_empty())
                .or_else(|| Self::displayable_sign_in_name(user.user_principal_name));
        }
        if email.is_none() && authentication.auth_source == AzureBearerAuthSource::AzureCli {
            email = authentication_provider
                .azure_cli_sign_in_name(authentication, executor.cancellation())
                .await
                .ok()
                .and_then(|name| UserPrincipalName::try_from(name).ok())
                .and_then(Self::displayable_sign_in_name);
        }
        let photo = if verified {
            EntraSignedInUserThumbnailGetRequest { authentication }
                .execute(&executor)
                .await
                .ok()
                .flatten()
                .and_then(|photo| Self::decode_photo(&photo.bytes))
                .map(|image| {
                    ctx.load_texture("Azure account thumbnail", image, TextureOptions::LINEAR)
                })
        } else {
            None
        };
        Ok(Self { email, photo })
    }

    fn displayable_sign_in_name(name: UserPrincipalName) -> Option<String> {
        // External guest names identify the directory object, not the mailbox.
        (!name
            .as_bytes()
            .windows(5)
            .any(|part| part.eq_ignore_ascii_case(b"#EXT#")))
        .then(|| name.into())
    }

    fn decode_photo(bytes: &[u8]) -> Option<ColorImage> {
        let mut reader = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .ok()?;
        if !matches!(reader.format(), Some(ImageFormat::Jpeg | ImageFormat::Png)) {
            return None;
        }
        let mut limits = Limits::default();
        limits.max_image_width = Some(128);
        limits.max_image_height = Some(128);
        limits.max_alloc = Some(1024 * 1024);
        reader.limits(limits);
        let image = reader.decode().ok()?.into_rgba8();
        Some(ColorImage::from_rgba_unmultiplied(
            [image.width() as usize, image.height() as usize],
            image.as_raw(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;

    #[test]
    fn masks_the_displayed_email_and_skips_synthetic_guest_names() -> Result<()> {
        let name = UserPrincipalName::try_new("personal@example.invalid")?;
        let profile = AzureAccountProfile {
            email: AzureAccountProfile::displayable_sign_in_name(name),
            photo: None,
        };
        assert_eq!(
            profile.display_email(false),
            Some("personal@example.invalid")
        );
        assert_eq!(
            profile.display_email(true),
            Some(AzureAccountProfile::REDACTED_EMAIL)
        );
        assert_eq!(AzureAccountProfile::default().display_email(true), None);
        let guest = UserPrincipalName::try_new("personal_example.invalid#ext#@tenant.invalid")?;
        assert_eq!(AzureAccountProfile::displayable_sign_in_name(guest), None);
        Ok(())
    }

    #[test]
    fn thumbnail_decode_bounds_dimensions() -> Result<()> {
        for format in [ImageFormat::Jpeg, ImageFormat::Png] {
            let mut bytes = Cursor::new(Vec::new());
            DynamicImage::new_rgb8(48, 48).write_to(&mut bytes, format)?;
            assert_eq!(
                AzureAccountProfile::decode_photo(bytes.get_ref())
                    .unwrap()
                    .size,
                [48, 48]
            );
        }

        let mut oversized = Cursor::new(Vec::new());
        DynamicImage::new_rgba8(129, 48).write_to(&mut oversized, ImageFormat::Png)?;
        assert!(AzureAccountProfile::decode_photo(oversized.get_ref()).is_none());
        Ok(())
    }
}
