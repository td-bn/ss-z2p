use crate::domain::subscriber_name::SubscriberName;
use crate::domain::SubscriberEmail;

#[derive(Debug)]
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
