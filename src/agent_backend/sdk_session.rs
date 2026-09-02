use std::ops::{Deref, DerefMut};

use crate::acp::AgentError;
use crate::bridge_sdk::{BridgeSession, StreamLog};
use crate::codex_sdk::CodexSession;
use crate::pi_sdk::PiEmbeddedSession;

pub(crate) enum SdkSession {
    Cursor(Box<BridgeSession>),
    Pi(Box<PiEmbeddedSession>),
    Codex(Box<CodexSession>),
}

impl Deref for SdkSession {
    type Target = StreamLog;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Cursor(session) => &session.log,
            Self::Pi(session) => &session.log,
            Self::Codex(session) => &session.log,
        }
    }
}

impl DerefMut for SdkSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Cursor(session) => &mut session.log,
            Self::Pi(session) => &mut session.log,
            Self::Codex(session) => &mut session.log,
        }
    }
}

impl SdkSession {
    pub(crate) async fn send_prompt(&self, prompt: &str) -> Result<(), AgentError> {
        match self {
            Self::Cursor(session) => session.send_prompt(prompt).await,
            Self::Pi(session) => session.send_prompt(prompt).await,
            Self::Codex(session) => session.send_prompt(prompt).await,
        }
    }

    pub(crate) async fn shutdown(self) -> Result<(), AgentError> {
        match self {
            Self::Cursor(session) => session.shutdown().await,
            Self::Pi(session) => session.shutdown().await,
            Self::Codex(session) => session.shutdown().await,
        }
    }

    #[must_use]
    pub(crate) const fn as_cursor(&self) -> Option<&BridgeSession> {
        match self {
            Self::Cursor(session) => Some(session),
            Self::Pi(_) | Self::Codex(_) => None,
        }
    }

    #[cfg(test)]
    pub(crate) const fn as_cursor_mut(&mut self) -> Option<&mut BridgeSession> {
        match self {
            Self::Cursor(session) => Some(session),
            Self::Pi(_) | Self::Codex(_) => None,
        }
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn as_bridge(&self) -> Option<&BridgeSession> {
        self.as_cursor()
    }

    #[cfg(test)]
    pub(crate) const fn as_bridge_mut(&mut self) -> Option<&mut BridgeSession> {
        self.as_cursor_mut()
    }
}
