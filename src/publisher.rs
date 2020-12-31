use crate::message::Message;

/// Automatically implemented.
///
/// Indicates that the implementer is expected to publish messages of
/// type M. This trait should never be implemented directly, rather it should be
/// automatically implemented by the `publish` attribute of the [`Source`] or
/// [`Actor`] derives.
///
/// [`Actor`]: ::yaaf_macros::Actor
/// [`Source`]: ::yaaf_macros::Source
pub trait Publisher<M: Message> {}
