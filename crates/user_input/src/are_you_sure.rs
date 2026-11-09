use crate::PickResult;
use crate::PickerTui;
use teamy_cancellation::CancellationScope;

pub async fn are_you_sure(
    message: impl Into<String>,
    cancellation: &CancellationScope,
) -> PickResult<bool, &'static str> {
    PickerTui::<&str>::new()
        .set_header(message)
        .pick_one(["No", "Yes"], cancellation)
        .await
        .map(|s| s == "Yes")
}
