use crate::{Pending, PushFailure, Subscription};
pub type DeliveryOutcome = (
    String,
    Pending,
    Option<Subscription>,
    Result<(), PushFailure>,
);
