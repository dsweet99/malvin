use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;

use crate::acp::AgentError;
use crate::bridge_sdk::{BridgeSession, ChildProcessSession, StreamLog};
use crate::codex_sdk::CodexSession;
use crate::npm_pi_sdk::NpmPiSession;

pub(crate) type SessionFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), AgentError>> + Send + 'a>>;

pub(crate) trait BackendSession: ChildProcessSession + Send {
    fn send_prompt<'a>(&'a self, prompt: &'a str) -> SessionFuture<'a>;
    fn shutdown(self: Box<Self>) -> SessionFuture<'static>;

    fn as_cursor(&self) -> Option<&BridgeSession> {
        None
    }

    #[cfg(test)]
    fn as_cursor_mut(&mut self) -> Option<&mut BridgeSession> {
        None
    }
}

impl BackendSession for BridgeSession {
    fn send_prompt<'a>(&'a self, prompt: &'a str) -> SessionFuture<'a> {
        Box::pin(Self::send_prompt(self, prompt))
    }

    fn shutdown(self: Box<Self>) -> SessionFuture<'static> {
        Box::pin(Self::shutdown(*self))
    }

    fn as_cursor(&self) -> Option<&BridgeSession> {
        Some(self)
    }

    #[cfg(test)]
    fn as_cursor_mut(&mut self) -> Option<&mut BridgeSession> {
        Some(self)
    }
}

impl BackendSession for NpmPiSession {
    fn send_prompt<'a>(&'a self, prompt: &'a str) -> SessionFuture<'a> {
        Box::pin(Self::send_prompt(self, prompt))
    }

    fn shutdown(self: Box<Self>) -> SessionFuture<'static> {
        Box::pin(Self::shutdown(*self))
    }
}

impl BackendSession for CodexSession {
    fn send_prompt<'a>(&'a self, prompt: &'a str) -> SessionFuture<'a> {
        Box::pin(Self::send_prompt(self, prompt))
    }

    fn shutdown(self: Box<Self>) -> SessionFuture<'static> {
        Box::pin(Self::shutdown(*self))
    }
}

pub(crate) struct SdkSession(Box<dyn BackendSession>);

impl SdkSession {
    pub(crate) fn new(session: impl BackendSession + 'static) -> Self {
        Self(Box::new(session))
    }

    pub(crate) async fn send_prompt(&self, prompt: &str) -> Result<(), AgentError> {
        self.0.send_prompt(prompt).await
    }

    pub(crate) async fn shutdown(self) -> Result<(), AgentError> {
        self.0.shutdown().await
    }

    #[must_use]
    pub(crate) fn as_cursor(&self) -> Option<&BridgeSession> {
        self.0.as_cursor()
    }

    #[cfg(test)]
    pub(crate) fn as_cursor_mut(&mut self) -> Option<&mut BridgeSession> {
        self.0.as_cursor_mut()
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn as_bridge(&self) -> Option<&BridgeSession> {
        self.as_cursor()
    }

    #[cfg(test)]
    pub(crate) fn as_bridge_mut(&mut self) -> Option<&mut BridgeSession> {
        self.as_cursor_mut()
    }
}

impl Deref for SdkSession {
    type Target = StreamLog;

    fn deref(&self) -> &Self::Target {
        &self.0.stdio().log
    }
}

impl DerefMut for SdkSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.stdio_mut().log
    }
}
