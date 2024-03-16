use crate::graphs::path::Path;

pub mod fast_shortcut_replacer;
pub mod slow_shortcut_replacer;

pub trait ShortcutReplacer: Sync + Send {
    fn replace_shortcuts(&self, path: &Path) -> Path;
}
