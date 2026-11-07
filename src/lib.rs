//! otto-bot, a coding agent harness built on the Domain Container architecture. The domain container is
//! `otto_bot_facade` and everything under `managers`; `services` sit outside it and call only the facade.

pub mod managers;
pub mod otto_bot_facade;
pub mod services;
