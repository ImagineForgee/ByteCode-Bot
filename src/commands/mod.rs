mod register;

use poise::{Command};

use crate::{Data, FullError};

pub struct CommandRegistration {
    pub constructor: fn() -> Command<Data, FullError>,
}

inventory::collect!(CommandRegistration);

pub fn collect_commands() -> Vec<Command<Data, FullError>> {
    inventory::iter::<CommandRegistration>
        .into_iter()
        .map(|reg| (reg.constructor)())
        .collect()
}
