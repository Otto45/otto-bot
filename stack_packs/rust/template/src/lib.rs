//! The Domain Container architecture's example app. The domain container is `example_app_facade` and everything
//! under `managers`; `services` sit outside it and call only the facade.

pub mod example_app_facade;
pub mod managers;
pub mod services;
