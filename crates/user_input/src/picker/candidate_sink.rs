use super::candidate_message::CandidateMessage;
use crate::IntoChoices;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct CandidateSink<T> {
    pub(super) sender: mpsc::UnboundedSender<CandidateMessage<T>>,
    pub(super) generation: u64,
}

impl<T> CandidateSink<T> {
    pub fn push(&self, choices: impl IntoChoices<T>) -> eyre::Result<()> {
        self.sender
            .send(CandidateMessage {
                generation: self.generation,
                choices: choices.into_choices(),
            })
            .map_err(|_| eyre::eyre!("picker has already shut down"))
    }
}
