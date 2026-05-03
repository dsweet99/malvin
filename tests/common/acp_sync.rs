use super::acp_core::{acp_mock_code_with_run_dir_js, write_artifact_lgtm};

pub fn acp_mock_sync_review_lgtm_with_abort_js() -> String {
    let lgtm = write_artifact_lgtm();
    let body = format!(
        r"    if (promptText.includes('Find a discrepancy between the codebase and')) {{
      let attempts = (typeof this.syncAttempts === 'undefined') ? 0 : this.syncAttempts;
      this.syncAttempts = attempts + 1;
      if (this.syncAttempts === 1) {{
        fs.writeFileSync(path.join(runDir, 'review.md'), 'needs attention\\n', 'utf8');
      }} else {{
{lgtm}
      }}
    }} else if (promptText.includes('Please review the codebase.')) {{
{lgtm}
      fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: sync review LGTM abort test\\n', 'utf8');
    }} else if (promptText.includes('Concerns')) {{
    }}",
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_sync_tamper_and_review_restore_js() -> String {
    let body = r"    if (promptText.includes('Find a discrepancy between the codebase and')) {
      const syncAttempts = (typeof this.syncAttempts === 'undefined') ? 0 : this.syncAttempts;
      this.syncAttempts = syncAttempts + 1;
      if (syncAttempts === 0) {
        fs.writeFileSync(path.join(process.cwd(), 'grounding.md'), 'TAMPERED', 'utf8');
        fs.writeFileSync(path.join(process.cwd(), '.kissconfig'), 'TAMPERED', 'utf8');
      }
      fs.writeFileSync(path.join(runDir, 'review.md'), 'needs attention\n', 'utf8');
    } else if (promptText.includes('Please review the codebase.')) {
      const grounding = (() => { try { return fs.readFileSync(path.join(process.cwd(), 'grounding.md'), 'utf8'); } catch { return ''; } })();
      const kiss = (() => { try { return fs.readFileSync(path.join(process.cwd(), '.kissconfig'), 'utf8'); } catch { return ''; } })();
      if (grounding === 'x' && kiss === 'k') {
        fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM\n', 'utf8');
      } else {
        fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: review saw tampered files\n', 'utf8');
      }
    }";
    acp_mock_code_with_run_dir_js(&format!("    {body}"))
}

pub fn acp_mock_sync_reviewer_restore_between_attempts_js() -> String {
    let body = r"    if (promptText.includes('Find a discrepancy between the codebase and')) {
      fs.writeFileSync(path.join(process.cwd(), 'grounding.md'), 'x', 'utf8');
      fs.writeFileSync(path.join(process.cwd(), '.kissconfig'), 'k', 'utf8');
    } else if (promptText.includes('Please review the codebase.')) {
      const reviewAttempts = (typeof this.reviewAttempts === 'undefined') ? 0 : this.reviewAttempts;
      this.reviewAttempts = reviewAttempts + 1;
      if (reviewAttempts === 0) {
        fs.writeFileSync(path.join(process.cwd(), 'grounding.md'), 'TAMPERED', 'utf8');
        fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'needs attention\n', 'utf8');
      } else {
        const grounding = (() => { try { return fs.readFileSync(path.join(process.cwd(), 'grounding.md'), 'utf8'); } catch { return ''; } })();
        const kiss = (() => { try { return fs.readFileSync(path.join(process.cwd(), '.kissconfig'), 'utf8'); } catch { return ''; } })();
        if (grounding === 'x' && kiss === 'k') {
          fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM\n', 'utf8');
        } else {
          fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: review still tampered\n', 'utf8');
        }
      }
    }";
    acp_mock_code_with_run_dir_js(&format!("    {body}"))
}

pub fn acp_mock_sync_check_sync_non_exact_lgtm_js() -> String {
    let body = r"    if (promptText.includes('Find a discrepancy between the codebase and')) {
      fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM with notes\\n', 'utf8');
    } else if (promptText.includes('Please review the codebase.')) {
      fs.writeFileSync(path.join(runDir, 'review.md'), 'needs attention\\n', 'utf8');
    }";
    acp_mock_code_with_run_dir_js(&format!("    {body}"))
}
