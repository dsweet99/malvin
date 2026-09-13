use crate::acp::AgentError;

use super::sdk_client::SdkClient;

pub struct ActiveCoderSession<'a> {
    pub(super) client: &'a mut SdkClient,
}

impl SdkClient {
    pub fn active_coder_session(&mut self) -> Result<ActiveCoderSession<'_>, AgentError> {
        if self.coder.is_idle() {
            return Err(AgentError("begin_coder_session was not called".into()));
        }
        Ok(ActiveCoderSession { client: self })
    }
}

#[cfg(test)]
mod tests {
    use super::super::sdk_client::BegunCoderSession;
    use super::super::test_support::test_io;
    use super::SdkClient;
    use crate::model_id::parse_model_id;
    use std::path::PathBuf;

    fn test_client() -> SdkClient {
        let model = parse_model_id("cursor:auto").expect("model");
        SdkClient::new(model, test_io())
    }

    #[test]
    fn idle_client_cannot_form_active_coder_session() {
        let mut client = test_client();
        let Err(err) = client.active_coder_session() else {
            panic!("idle client must not form ActiveCoderSession")
        };
        assert!(err.message.contains("begin_coder_session was not called"));
    }

    #[test]
    fn needs_respawn_forms_active_coder_session() {
        let mut client = test_client();
        client.coder = BegunCoderSession::NeedsRespawn {
            cwd: PathBuf::from("/tmp/work"),
        };
        assert!(client.active_coder_session().is_ok());
    }
}
