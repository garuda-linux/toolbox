//! Unit signals that relist a given number of times, then end with the
//! given result.

use futures_lite::future::Boxed;
use toolbox_plan::UnitScope;

use crate::bus::CallError;
use crate::systemd::{OnSignal, UnitSignals};

#[derive(Clone, Debug)]
pub struct ScriptedSignals {
    pub signals: usize,
    pub ended: Result<(), CallError>,
}

impl UnitSignals for ScriptedSignals {
    fn follow(&self, _scope: UnitScope, mut on_signal: OnSignal) -> Boxed<Result<(), CallError>> {
        let script = self.clone();
        Box::pin(async move {
            for _ in 0..script.signals {
                on_signal();
            }

            script.ended
        })
    }
}
