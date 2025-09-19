use crate::PickResult;
use crate::PickerTui;

pub fn are_you_sure(message: impl Into<String>) -> PickResult<bool> {
    PickerTui::new(["No", "Yes"])
        .set_header(message)
        .pick_one()
        .map(|s| s == "Yes")
}
