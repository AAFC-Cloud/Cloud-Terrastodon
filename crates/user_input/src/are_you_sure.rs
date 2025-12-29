use crate::PickResult;
use crate::PickerTui;

pub fn are_you_sure(message: impl Into<String>) -> PickResult<bool> {
    PickerTui::new()
        .set_header(message)
        .pick_one(["No", "Yes"])
        .map(|s| s == "Yes")
}
