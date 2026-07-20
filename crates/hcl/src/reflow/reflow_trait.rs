use crate::HclProject;

#[async_trait::async_trait]
pub trait HclReflower: Send {
    async fn reflow(&mut self, hcl: HclProject) -> eyre::Result<HclProject>;
}
