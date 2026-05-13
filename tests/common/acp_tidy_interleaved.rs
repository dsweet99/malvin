use super::acp_core::acp_mock_js;

#[allow(clippy::needless_raw_string_hashes)]
const TIDY_REVIEWER_LGTM_HANDLER: &str = r#"    const fs = require('fs');
    const path = require('path');
    const promptText = (((msg.params || {}).prompt || [])[0] || {}).text || '';
    const runRoot = path.join(process.cwd(), '_malvin');
    const runDirNames = fs.readdirSync(runRoot, { withFileTypes: true }).filter((e) => e.isDirectory()).map((e) => e.name).sort();
    const runDir = path.join(runRoot, runDirNames[runDirNames.length - 1]);
    if (promptText.includes('<!-- malvin:review_tidy_turn_v1 -->')) {
      fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM\n', 'utf8');
      console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text: 'review\n' } } } }));
    } else {
      console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text: 'coder\n' } } } }));
    }"#;

#[must_use]
pub fn acp_mock_tidy_reviewer_lgtm_js() -> String {
    acp_mock_js("", TIDY_REVIEWER_LGTM_HANDLER)
}

#[cfg(test)]
mod acp_tidy_interleaved_kiss {
    #[test]
    fn kiss_stringify_acp_tidy_interleaved() {
        let _ = stringify!(super::acp_mock_tidy_reviewer_lgtm_js);
    }
}
