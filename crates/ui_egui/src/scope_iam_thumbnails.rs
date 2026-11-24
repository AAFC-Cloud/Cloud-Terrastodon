use crate::app::MyApp;
use crate::azure_account_profile::AzureAccountProfile;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_azure::EntraUserThumbnailGetRequest;
use cloud_terrastodon_command::CancellationErrorExt;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use cloud_terrastodon_execution::ExecutableRequest;
use eframe::egui::ImageSource;
use eframe::egui::TextureHandle;
use eframe::egui::TextureOptions;
use eframe::egui::Ui;
use eframe::egui::UiBuilder;
use eframe::egui::UiStackInfo;
use eframe::egui::load::SizedTexture;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::runtime::Handle;
use tracing::debug;

const MAX_PHOTOS: usize = 256;
const MAX_BATCH: usize = 12;
const THUMBNAIL_CACHE_TAG: &str = "scope-iam-thumbnail-cache";

/// Keeps recent directory profile photos for one view's authenticated content.
#[derive(Default)]
pub(crate) struct ScopeIamThumbnails {
    revision: u64,
    demo_mode: bool,
    photos: HashMap<EntraUserId, Option<TextureHandle>>,
    insertion_order: VecDeque<EntraUserId>,
    visible: Vec<EntraUserId>,
    loading: bool,
}

impl ScopeIamThumbnails {
    fn insert(&mut self, id: EntraUserId, texture: Option<TextureHandle>) {
        if self.photos.len() >= MAX_PHOTOS
            && let Some(oldest) = self.insertion_order.pop_front()
        {
            self.photos.remove(&oldest);
        }
        self.photos.insert(id, texture);
        self.insertion_order.push_back(id);
    }
}

/// Let nested table cells find the cache belonging to this content.
pub(crate) fn with_iam_thumbnails<T>(
    cache: &Arc<Mutex<ScopeIamThumbnails>>,
    demo_mode: bool,
    ui: &mut Ui,
    render: impl FnOnce(&mut Ui) -> T,
) -> T {
    if let Ok(mut state) = cache.lock() {
        if state.demo_mode != demo_mode {
            *state = ScopeIamThumbnails {
                revision: state.revision.wrapping_add(1),
                demo_mode,
                ..Default::default()
            };
        }
        state.visible.clear();
    }
    ui.scope_builder(
        UiBuilder::new().ui_stack_info(
            UiStackInfo::default().with_tag_value(THUMBNAIL_CACHE_TAG, Arc::clone(cache)),
        ),
        render,
    )
    .inner
}

pub(crate) fn scope_iam_thumbnail(ui: &Ui, user_id: EntraUserId) -> Option<ImageSource<'static>> {
    let cache = ui.stack().iter().find_map(|node| {
        node.tags()
            .get_downcast::<Arc<Mutex<ScopeIamThumbnails>>>(THUMBNAIL_CACHE_TAG)
    })?;
    let mut state = cache.lock().ok()?;
    if state.demo_mode {
        return None;
    }
    if let Some(photo) = state.photos.get(&user_id) {
        return photo
            .as_ref()
            .map(|photo| ImageSource::Texture(SizedTexture::new(photo.id(), photo.size_vec2())));
    }
    if !state.loading && state.visible.len() < MAX_BATCH && !state.visible.contains(&user_id) {
        state.visible.push(user_id);
    }
    None
}

/// Schedule work after rendering so each batch only covers currently visible rows.
pub(crate) fn load_queued_iam_thumbnails(app: &MyApp, ui: &Ui) {
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    let cache = &app.azure.scope_iam.thumbnails;
    let (revision, pending) = {
        let Ok(mut state) = cache.lock() else {
            return;
        };
        if state.loading || state.visible.is_empty() {
            return;
        }
        state.loading = true;
        (state.revision, std::mem::take(&mut state.visible))
    };
    let destination = Arc::downgrade(cache);
    let context = ui.ctx().clone();
    let executor = app.executor.clone();
    let queued = app.work_tracker.spawn(
        app.executor.cancellation(),
        &Handle::current(),
        "Load IAM profile photos".to_owned(),
        async move |scope| {
            // This path uses private in-memory reads and needs no disk cache lease.
            let executor = executor.with_cancellation_scope(scope);
            let photos = pending
                .into_iter()
                .map_concurrent(
                    &executor,
                    NonZeroUsize::new(4).unwrap_or(NonZeroUsize::MIN),
                    async |child, user_id| {
                        let photo = EntraUserThumbnailGetRequest {
                            authentication,
                            user_id,
                        }
                        .execute(&child)
                        .await;
                        let photo = match photo {
                            Ok(photo) => photo
                                .and_then(|photo| AzureAccountProfile::decode_photo(&photo.bytes)),
                            Err(error) if error.is_cancellation() => return Err(error),
                            Err(error) => {
                                debug!(error = %error, "Could not load IAM profile photo");
                                None
                            }
                        };
                        eyre::Ok((user_id, photo))
                    },
                )
                .await;
            if let Some(destination) = destination.upgrade()
                && let Ok(mut state) = destination.lock()
                && state.revision == revision
            {
                state.loading = false;
                for (id, photo) in photos? {
                    let texture = photo.map(|photo| {
                        context.load_texture("IAM profile photo", photo, TextureOptions::LINEAR)
                    });
                    state.insert(id, texture);
                }
                context.request_repaint();
            }
            Ok(())
        },
    );
    if let Err(error) = queued {
        if let Ok(mut state) = cache.lock() {
            state.loading = false;
        }
        if !error.is_cancellation() {
            debug!(error = %error, "Could not schedule IAM profile photos");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Color32;
    use eframe::egui::ColorImage;
    use eframe::egui::Context;
    use eframe::egui::RawInput;

    #[test]
    fn thumbnail_cells_use_the_cache_of_their_own_content() -> eyre::Result<()> {
        let user = "00000000-0000-0000-0000-000000000001".parse()?;
        let context = Context::default();
        let mut rendered = Vec::new();
        let mut output = context.run_ui(RawInput::default(), |ui| {
            for color in [Color32::RED, Color32::BLUE] {
                let texture = context.load_texture(
                    "synthetic user photo",
                    ColorImage::filled([1, 1], color),
                    TextureOptions::LINEAR,
                );
                let id = texture.id();
                let mut state = ScopeIamThumbnails::default();
                state.insert(user, Some(texture));
                let cache = Arc::new(Mutex::new(state));
                let result = with_iam_thumbnails(&cache, false, ui, |ui| {
                    ui.horizontal(|ui| scope_iam_thumbnail(ui, user)).inner
                });
                let Some(ImageSource::Texture(actual)) = result else {
                    panic!("selected content should supply its own photo");
                };
                assert_eq!(actual.id, id);
                rendered.push(id);
            }
        });
        output.textures_delta.clear();
        assert_ne!(rendered[0], rendered[1]);
        Ok(())
    }
}
