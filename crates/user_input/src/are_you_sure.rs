use crate::PickResult;
use crate::PickerTui;

pub async fn are_you_sure(message: impl Into<String>) -> PickResult<bool> {
    PickerTui::<&str>::new()
        .set_header(message)
        .pick_one(["No", "Yes"])
        .await
        .map(|s| s == "Yes")
}
