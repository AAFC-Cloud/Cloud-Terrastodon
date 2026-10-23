use crate::HclProject;

#[expect(
    clippy::double_must_use,
    reason = "async-trait adds #[must_use] to an already must-use future"
)]
#[async_trait::async_trait]
pub trait HclReflower: Send {
    async fn reflow(&mut self, hcl: HclProject) -> eyre::Result<HclProject>;
}
