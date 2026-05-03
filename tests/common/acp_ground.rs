use super::acp_core::{acp_mock_code_with_run_dir_js, acp_mock_js, session_update_chunk_line};

#[cfg(unix)]
pub fn acp_mock_ground_loop_converges_with_missing_grounding_js() -> String {
    let body = r"    const marker = path.join(runDir, 'ground_prompt_visits.txt');
    const mark = (name) => {
      const current = (() => { try { return fs.readFileSync(marker, 'utf8'); } catch { return ''; } })();
      fs.writeFileSync(marker, `${current}${name}\n`, 'utf8');
    };
    if (promptText.includes('{{') || promptText.includes('}}')) {
      fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: unrendered prompt placeholder\\n', 'utf8');
      return;
    }
    if (promptText.includes('write a new grounding file')) {
      mark('write');
      const syncAttempts = (typeof this.syncAttempts === 'undefined') ? 0 : this.syncAttempts;
      this.syncAttempts = syncAttempts + 1;
      fs.writeFileSync(path.join(process.cwd(), 'grounding.md'), 'CREATED\n', 'utf8');
      if (this.syncAttempts > 1) {
        fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM\n', 'utf8');
      }
    } else if (promptText.includes('Find a discrepancy between the codebase and')) {
      mark('check');
      const reviewAttempts = (typeof this.reviewAttempts === 'undefined') ? 0 : this.reviewAttempts;
      this.reviewAttempts = reviewAttempts + 1;
      if (this.reviewAttempts === 1) {
        fs.writeFileSync(path.join(runDir, 'review.md'), 'needs attention\n', 'utf8');
      } else {
        fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM\n', 'utf8');
      }
    } else if (promptText.includes('improve the existing grounding file')) {
      mark('improve');
      const reviewAttempts = (typeof this.reviewAttempts === 'undefined') ? 0 : this.reviewAttempts;
      if (reviewAttempts === 1) {
        fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM\n', 'utf8');
      }
    } else {
      mark('unexpected');
    }";
    acp_mock_code_with_run_dir_js(&format!("    {body}"))
}

#[cfg(unix)]
pub fn acp_mock_ground_check_abort_js() -> String {
    let body = r"    if (promptText.includes('Find a discrepancy between the codebase and')) {
      fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'needs attention\n', 'utf8');
      fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: reviewer requested stop\n', 'utf8');
    }";
    acp_mock_code_with_run_dir_js(&format!("    {body}"))
}

#[cfg(unix)]
pub fn acp_mock_ground_write_tamper_kissconfig_js() -> String {
    let body = r"    if (promptText.includes('write a new grounding file')) {
      fs.writeFileSync(path.join(process.cwd(), 'grounding.md'), 'CREATED\n', 'utf8');
      fs.writeFileSync(path.join(process.cwd(), '.kissconfig'), 'TAMPERED', 'utf8');
      fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM\n', 'utf8');
    } else if (promptText.includes('Find a discrepancy between the codebase and')) {
      fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM\n', 'utf8');
    }";
    acp_mock_code_with_run_dir_js(&format!("    {body}"))
}

#[cfg(unix)]
pub fn acp_mock_ground_check_tamper_kissconfig_js() -> String {
    let body = r"    if (promptText.includes('Find a discrepancy between the codebase and')) {
      const reviewAttempts = (typeof this.reviewAttempts === 'undefined') ? 0 : this.reviewAttempts;
      this.reviewAttempts = reviewAttempts + 1;
      if (this.reviewAttempts === 1) {
        fs.writeFileSync(path.join(process.cwd(), '.kissconfig'), 'TAMPERED', 'utf8');
        fs.writeFileSync(path.join(runDir, 'review.md'), 'needs attention\n', 'utf8');
      } else {
        fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM\n', 'utf8');
      }
    } else if (promptText.includes('improve the existing grounding file')) {
      fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM\n', 'utf8');
    }";
    acp_mock_code_with_run_dir_js(&format!("    {body}"))
}

#[cfg(unix)]
pub fn acp_mock_ground_prompt_render_paths_js() -> String {
    let body = r"    const marker = path.join(runDir, 'ground_prompt_visits.txt');
    const mark = (name) => {
      const current = (() => { try { return fs.readFileSync(marker, 'utf8'); } catch { return ''; } })();
      fs.writeFileSync(marker, `${current}${name}\n`, 'utf8');
    };
    const failIfUnrendered = () => {
      if (promptText.includes('{{') || promptText.includes('}}')) {
        fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: unrendered prompt placeholder\\n', 'utf8');
        return true;
      }
      return false;
    };
    if (failIfUnrendered()) {
      fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: unrendered prompt placeholder\\n', 'utf8');
    } else if (promptText.includes('write a new grounding file')) {
      mark('write');
      fs.writeFileSync(path.join(process.cwd(), 'grounding.md'), 'CREATED\\n', 'utf8');
    } else if (promptText.includes('Find a discrepancy between the codebase and')) {
      mark('check');
      const checkAttempts = (typeof this.groundCheckAttempts === 'undefined') ? 0 : this.groundCheckAttempts;
      this.groundCheckAttempts = checkAttempts + 1;
      if (this.groundCheckAttempts === 1) {
        fs.writeFileSync(path.join(runDir, 'review.md'), 'needs attention\\n', 'utf8');
      } else {
        fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM\\n', 'utf8');
      }
    } else if (promptText.includes('improve the existing grounding file')) {
      mark('improve');
      fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM\\n', 'utf8');
    } else {
      fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: unexpected ground prompt\\n', 'utf8');
    }";
    acp_mock_code_with_run_dir_js(body)
}

#[cfg(unix)]
pub fn acp_mock_ground_never_lgtm_loop_js() -> String {
    let body = r"    if (promptText.includes('write a new grounding file')) {
      fs.writeFileSync(path.join(process.cwd(), 'grounding.md'), 'CREATED\\n', 'utf8');
    } else if (promptText.includes('Find a discrepancy between the codebase and')) {
      fs.writeFileSync(path.join(runDir, 'review.md'), 'needs attention\\n', 'utf8');
    } else if (promptText.includes('improve the existing grounding file')) {
      fs.writeFileSync(path.join(runDir, 'review.md'), 'needs attention\\n', 'utf8');
    }";
    acp_mock_code_with_run_dir_js(&format!("    {body}"))
}

#[cfg(unix)]
pub fn acp_mock_kpop_tamper_then_restore_js() -> String {
    let body = r"    const fs = require('fs');
    const path = require('path');
    const kpopAttempts = (typeof this.kpopAttempts === 'undefined') ? 0 : this.kpopAttempts;
    this.kpopAttempts = kpopAttempts + 1;
    const grounding = (() => { try { return fs.readFileSync(path.join(process.cwd(), 'grounding.md'), 'utf8'); } catch { return ''; } })();
    const kiss = (() => { try { return fs.readFileSync(path.join(process.cwd(), '.kissconfig'), 'utf8'); } catch { return ''; } })();
    if (kpopAttempts === 0) {
      fs.writeFileSync(path.join(process.cwd(), 'grounding.md'), 'TAMPERED', 'utf8');
      fs.writeFileSync(path.join(process.cwd(), '.kissconfig'), 'TAMPERED', 'utf8');
    } else if (grounding !== 'x' || kiss !== 'k') {
      fs.writeFileSync(path.join(process.cwd(), 'result.md'), 'ABORT: kpop tamper restored incorrectly\n', 'utf8');
    }";
    let done = session_update_chunk_line("agent_message_chunk", r"'kpop prompt done\n'");
    acp_mock_js("", &format!("    {body}\n{done}"))
}
