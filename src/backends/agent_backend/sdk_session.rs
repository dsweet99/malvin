use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;

use crate::agent_process::AgentError;
use crate::backends::bridge_sdk::{BridgeSession, ChildProcessSession, StreamLog};
use crate::backends::codex_sdk::CodexSession;
use crate::backends::pi_backend::NpmPiSession;

pub(crate) type SessionFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), AgentError>> + Send + 'a>>;

pub(crate) type InjectFuture<'a> =
    Pin<Box<dyn Future<Output = Result<bool, AgentError>> + Send + 'a>>;

pub(crate) trait BackendSession: ChildProcessSession + Send {
    fn send_prompt<'a>(&'a self, prompt: &'a str) -> SessionFuture<'a>;
    fn shutdown(self: Box<Self>) -> SessionFuture<'static>;

    fn inject_mid_turn<'a>(&'a self, _text: &'a str) -> InjectFuture<'a> {
        Box::pin(std::future::ready(Ok(false)))
    }

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

    fn inject_mid_turn<'a>(&'a self, text: &'a str) -> InjectFuture<'a> {
        Box::pin(Self::inject_mid_turn(self, text))
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

    fn inject_mid_turn<'a>(&'a self, text: &'a str) -> InjectFuture<'a> {
        Box::pin(Self::inject_mid_turn(self, text))
    }
}

impl BackendSession for CodexSession {
    fn send_prompt<'a>(&'a self, prompt: &'a str) -> SessionFuture<'a> {
        Box::pin(Self::send_prompt(self, prompt))
    }

    fn shutdown(self: Box<Self>) -> SessionFuture<'static> {
        Box::pin(Self::shutdown(*self))
    }

    fn inject_mid_turn<'a>(&'a self, text: &'a str) -> InjectFuture<'a> {
        Box::pin(Self::inject_mid_turn(self, text))
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

    pub(crate) async fn inject_mid_turn(&self, text: &str) -> Result<bool, AgentError> {
        self.0.inject_mid_turn(text).await
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
