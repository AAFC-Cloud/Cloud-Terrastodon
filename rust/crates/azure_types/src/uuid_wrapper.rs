use uuid::Uuid;

pub trait UuidWrapper {
    fn new(uuid: Uuid) -> Self;
    fn as_ref(&self) -> &Uuid;
}
