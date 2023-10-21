use crate::{ClassId, Compiler};

impl<'a> Compiler<'a> {
    pub fn is_subclass(&mut self, base_class_id: ClassId, child_class_id: ClassId) -> bool {
        let mut current_class_id = child_class_id;
        loop {
            if current_class_id == base_class_id {
                return true;
            }
            if let Some(base) = self.super_class_id(current_class_id) {
                current_class_id = base;
            } else {
                return false;
            }
        }
    }
}
