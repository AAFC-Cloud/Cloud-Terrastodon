use jiff::Timestamp;

/// The state of an assignment within its service-reported time window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PimAssignmentStatus {
    #[default]
    Eligible,
    Active,
    Expired,
    Scheduled,
}

impl PimAssignmentStatus {
    pub fn at(self, now: Timestamp, start: Option<Timestamp>, end: Option<Timestamp>) -> Self {
        if end.is_some_and(|end| end <= now) {
            Self::Expired
        } else if start.is_some_and(|start| start > now) {
            Self::Scheduled
        } else {
            self
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Eligible => "Eligible assignments",
            Self::Active => "Active assignments",
            Self::Expired => "Expired assignments",
            Self::Scheduled => "Scheduled assignments",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::SignedDuration;

    #[test]
    fn an_eligible_assignment_is_available_only_during_its_window() {
        let start = Timestamp::UNIX_EPOCH;
        let end = start + SignedDuration::from_hours(1);
        let status = PimAssignmentStatus::Eligible;
        assert_eq!(
            status.at(start - SignedDuration::from_secs(1), Some(start), Some(end)),
            PimAssignmentStatus::Scheduled
        );
        assert_eq!(
            status.at(start, Some(start), Some(end)),
            PimAssignmentStatus::Eligible
        );
        assert_eq!(
            status.at(end, Some(start), Some(end)),
            PimAssignmentStatus::Expired
        );
    }
}
