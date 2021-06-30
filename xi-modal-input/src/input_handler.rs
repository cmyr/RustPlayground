use libc::c_char;
use std::ffi::CString;

use xi_core_lib::edit_types::EventDomain;

use super::OneView;
use crate::update::{Update, UpdateBuilder};

// a token for an event that has been scheduled with a delay.
pub type PendingToken = u32;

type Milliseconds = u32;

pub enum EventPayload {}

pub struct KeyEvent {
    pub modifiers: u32,
    pub characters: &'static str,
    pub payload: *const EventPayload,
}

pub struct Plumber {
    event_callback: extern "C" fn(*const EventPayload, bool),
    action_callback: extern "C" fn(*const c_char),
    timer_callback: extern "C" fn(*const EventPayload, u32) -> u32,
    cancel_timer_callback: extern "C" fn(u32),
}

impl Plumber {
    pub fn new(
        event_callback: extern "C" fn(*const EventPayload, bool),
        action_callback: extern "C" fn(*const c_char),
        timer_callback: extern "C" fn(*const EventPayload, u32) -> u32,
        cancel_timer_callback: extern "C" fn(u32),
    ) -> Plumber {
        Plumber { event_callback, action_callback, timer_callback, cancel_timer_callback }
    }
}

pub struct EventCtx<'a> {
    pub plumber: &'a Plumber,
    pub state: &'a mut OneView,
}

impl<'a> EventCtx<'a> {
    pub(crate) fn send_event(&self, event: KeyEvent) {
        let KeyEvent { payload, .. } = event;
        (self.plumber.event_callback)(payload, false);
    }

    pub(crate) fn free_event(&self, event: KeyEvent) {
        let KeyEvent { payload, .. } = event;
        (self.plumber.event_callback)(payload, true);
    }

    pub(crate) fn schedule_event(&self, event: KeyEvent, delay: Milliseconds) -> PendingToken {
        let KeyEvent { payload, .. } = event;
        (self.plumber.timer_callback)(payload, delay)
    }

    pub(crate) fn cancel_timer(&self, token: PendingToken) {
        (self.plumber.cancel_timer_callback)(token);
    }

    pub(crate) fn send_client_rpc<V>(&self, method: &str, params: V)
    where
        V: Into<Option<serde_json::Value>>,
    {
        let params = params.into().unwrap_or_else(|| serde_json::Map::new().into());
        let json = json!({
            "method": method,
            "params": params,
        });
        let action_str = serde_json::to_string(&json).expect("Value is always valid json");
        let action_cstr = CString::new(action_str).expect("json should be well formed");
        (self.plumber.action_callback)(action_cstr.as_ptr());
    }

    pub(crate) fn do_core_event(
        &mut self,
        action: EventDomain,
        repeat: usize,
        update: &mut UpdateBuilder,
    ) {
        eprintln!("doing {:?} x {}", &action, repeat);
        for _ in 0..repeat {
            let this_update = self.state.handle_event(action.clone());
            update.inner = this_update;
        }
    }
}

pub trait Handler {
    /// Returns `true` if we should update after this event
    fn handle_event(&mut self, event: KeyEvent, ctx: EventCtx) -> Option<Update>;
    /// Informs the handler that the given delayed event has fired.
    fn clear_pending(&mut self, token: PendingToken);
}
