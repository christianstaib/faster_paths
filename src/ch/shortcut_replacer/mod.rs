use crate::graphs::path::Path;

pub mod fast_shortcut_replacer;
pub mod slow_shortcut_replacer;

pub trait ShortcutReplacer {
    fn get_path(&self, path: &Path) -> Path;
}
