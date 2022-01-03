use std::sync::{Arc, RwLock};

/// The Tigger class represents a one-way shared boolean. It has the following
/// features:
/// - The state is shared. It can be cloned like an Arc
/// - The default value is false
/// - Once `fire()` has been called, the value is set to true and can never be
///   set back to false again
#[derive(Clone, Default, Debug)]
pub struct Trigger {
    flag: Arc<RwLock<bool>>,
}

impl Trigger {
    /// Sets the value to `true`
    pub fn fire(&self) {
        *(self.flag.write().unwrap()) = true;
    }

    /// Returns the current value of the trigger
    pub fn is_fired(&self) -> bool {
        *(self.flag.read().unwrap())
    }
}

#[cfg(test)]
mod unit {

    use super::*;

    #[test]
    fn default_is_false() {
        let trigger = Trigger::default();
        assert!(!*trigger.flag.read().unwrap());
    }

    #[test]
    fn fire_value_is_true() {
        let trigger = Trigger::default();
        trigger.fire();
        assert!(*trigger.flag.read().unwrap());
    }

    #[test]
    fn value_can_be_queried() {
        let trigger = Trigger::default();
        assert_eq!(*trigger.flag.read().unwrap(), trigger.is_fired());
    }

    #[test]
    fn content_is_shared_between_clones() {
        let trigger_a = Trigger::default();
        let trigger_b = trigger_a.clone();
        assert_eq!(trigger_a.is_fired(), trigger_b.is_fired());
        trigger_a.fire();
        assert_eq!(trigger_a.is_fired(), trigger_b.is_fired());
    }
}
