Could the main agent benefit from any of the advice in `malvin --advice`?

If it could benefit from any advice tag other than `facts`, output
__MALVIN_BTW_START__
This advice may be helpful: Run `malvin --advice <advice_tag>`
__MALVIN_BTW_END__
where you replace <advice_tag> is the advice tag you think is relevant.

OTOH, if it could benefit from a fact returned by the 'facts' tag, the write
__MALVIN_BTW_START__
This fact may be helpful: Run `malvin --advice facts | grep <fact_id>`
__MALVIN_BTW_END__
where you replace <fact_id> with the fact_id of the one fact you think is relevant.

Do not offer and advice/facts that you have offered before. See {{ lite_trace_path }} for
your previous traces.
