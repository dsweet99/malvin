pub(crate) struct PromptTraceDispatchMeta<'a> {
    pub incoming_tag: String,
    pub stdout_replacement_who: &'a str,
    pub trace_raw_output: bool,
    pub plain_lines: bool,
}

pub(crate) async fn uniform_outgoing_trace_preamble<'a>(
    session: &'a AcpSession,
    file: &mut tokio::fs::File,
    text: &'a str,
    u: &'a UniformOutgoingTrace<'a>,
) -> Result<PromptTraceDispatchMeta<'a>, String> {
    trace_write_invocation_header(file).await?;
    trace_write_outgoing_prompt(file, u.trace_who, text).await?;
    if !session.0.raw_output {
        let bracket_label = u.stdout_bracket_label.unwrap_or(u.trace_who);
        crate::output::print_outgoing_prompt_log(u.trace_who, bracket_label);
        if session.0.log_full_outgoing_prompts {
            let who_line = crate::output::format_acp_directional_tag_prefix('>', u.trace_who);
            crate::output::print_stdout_text(&who_line, text);
        }
    }
    append_prompts_log_uniform(
        session.0.prompts_log_run_dir.as_deref(),
        u.trace_who,
        u.stdout_bracket_label.unwrap_or(u.trace_who),
        session.0.log_full_outgoing_prompts.then_some(text),
    )
    .await?;
    Ok(PromptTraceDispatchMeta {
        incoming_tag: crate::output::format_acp_directional_tag_prefix('<', u.trace_who),
        stdout_replacement_who: u.trace_who,
        trace_raw_output: session.0.raw_output,
        plain_lines: false,
    })
}

pub(crate) async fn do_split_outgoing_trace_preamble<'a>(
    session: &'a AcpSession,
    file: &mut tokio::fs::File,
    split: &'a DoPromptTraceSplit<'a>,
) -> Result<PromptTraceDispatchMeta<'a>, String> {
    do_split_trace_preamble(file, session.0.raw_output, split).await?;
    let parts = DoOutgoingTraceParts {
        style_text: split.style_text,
        header_text: split.header,
        user_text: split.user,
    };
    let combined = compose_do_split_prompt_text(&parts);
    if !session.0.raw_output && session.0.log_full_outgoing_prompts {
        let who_line = crate::output::format_acp_directional_tag_prefix('>', "do");
        crate::output::print_stdout_text(&who_line, &combined);
    }
    append_prompts_log_do_plain(
        session.0.prompts_log_run_dir.as_deref(),
        &parts,
        session.0.log_full_outgoing_prompts,
    )
    .await?;
    Ok(PromptTraceDispatchMeta {
        incoming_tag: crate::output::format_acp_directional_tag_prefix('<', "do"),
        stdout_replacement_who: "do",
        trace_raw_output: session.0.raw_output,
        plain_lines: true,
    })
}
