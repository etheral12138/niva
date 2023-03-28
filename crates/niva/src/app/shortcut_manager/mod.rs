use crate::unsafe_impl_sync_send;

use super::{
    options::ShortcutsOptions,
    utils::{arc, arc_mut, ArcMut, Counter},
    NivaEventLoop, NivaId,
};
use anyhow::{anyhow, Result};
use std::{collections::HashMap, hash::Hash, result, str::FromStr, sync::Arc};
use tao::{
    accelerator::{Accelerator, AcceleratorId},
    global_shortcut::{GlobalShortcut, ShortcutManager},
};

unsafe_impl_sync_send!(NivaShortcutManager);
pub struct NivaShortcutManager {
    manager: ShortcutManager,
    shortcuts: HashMap<u16, (String, GlobalShortcut)>,
}

impl NivaShortcutManager {
    pub fn new(
        options: &Option<ShortcutsOptions>,
        event_loop: &NivaEventLoop,
    ) -> ArcMut<NivaShortcutManager> {
        let mut manager = NivaShortcutManager {
            manager: ShortcutManager::new(event_loop),
            shortcuts: HashMap::new(),
        };

        if let Some(ShortcutsOptions(options)) = options.clone() {
            for (accelerator_str, id) in options {
                manager.register( id,accelerator_str );
            }
        }

        arc_mut(manager)
    }

    pub fn register(&mut self, id: u16, accelerator_str: String) -> Result<()> {
        if self.shortcuts.contains_key(&id) {
            return Err(anyhow!("Shortcut with id {} already registered", id));
        }

        let accelerator = Accelerator::from_str(&accelerator_str)
            .map_err(|err| anyhow!("{}", err.to_string()))?
            .with_id(AcceleratorId(id));
        let shortcut = self.manager.register(accelerator)?;

        self.shortcuts.insert(id, (accelerator_str, shortcut));
        Ok(())
    }

    pub fn unregister(&mut self, id: u16) -> Result<()> {
        let (_, shortcut) = self
            .shortcuts
            .remove(&id)
            .ok_or(anyhow!("Shortcut with id {} not found", id))?;
        self.manager.unregister(shortcut)?;
        Ok(())
    }

    pub fn unregister_all(&mut self) -> Result<()> {
        self.manager.unregister_all()?;
        self.shortcuts.clear();
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<(u16, String)>> {
        Ok(self
            .shortcuts
            .iter()
            .map(|(id, (accelerator_str, _))| (*id, accelerator_str.clone()))
            .collect())
    }
}
