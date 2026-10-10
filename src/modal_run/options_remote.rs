use std::ffi::OsString;

use super::super::{MODAL_REMOTE, MODAL_REMOTE_ID, MODAL_SERVICE, REMOTE_FLAG, REMOTE_MODAL_ARG};
use super::{ModalOptions, parse_modal_spec};

fn check_remote_id(id: &str) -> Result<(), String> {
    const LISTS: &str = "`malvin admin remotes` lists the available remotes, and `[aliases.remotes]` in ~/.malvinconf/config.toml defines aliases";
    match id.split_once(':') {
        Some((MODAL_REMOTE, MODAL_SERVICE)) => Ok(()),
        Some((MODAL_REMOTE, service)) => Err(format!(
            "unknown {MODAL_REMOTE} service `{service}` (services: {MODAL_SERVICE}); {LISTS}"
        )),
        Some((provider, _)) => Err(format!("unknown remote provider `{provider}`; {LISTS}")),
        None if id == MODAL_REMOTE => Err(format!(
            "remote `{id}` needs a service, as PROVIDER:SERVICE; use `{MODAL_REMOTE_ID}`"
        )),
        None => Err(format!(
            "unknown remote `{id}` (remotes are PROVIDER:SERVICE, such as `{MODAL_REMOTE_ID}`); {LISTS}"
        )),
    }
}

pub fn parse_remote_value(value: &str) -> Result<ModalOptions, String> {
    let (id, spec) = match value.split_once('[') {
        Some((id, rest)) => {
            let (spec, tail) = rest
                .split_once(']')
                .ok_or_else(|| format!("`{REMOTE_FLAG}={value}` is missing its closing `]`"))?;
            if !tail.is_empty() {
                return Err(format!(
                    "`{REMOTE_FLAG}={value}` has unexpected text `{tail}` after its closing `]`"
                ));
            }
            (id, Some(spec))
        }
        None => (value, None),
    };
    check_remote_id(id)?;
    spec.map_or_else(
        || Ok(ModalOptions::default()),
        |spec| {
            parse_modal_spec(spec)
                .map_err(|e| format!("`{REMOTE_FLAG}={MODAL_REMOTE_ID}[...]`: {e}"))
        },
    )
}

fn remote_value(
    arg: &str,
    rest: &mut impl Iterator<Item = OsString>,
) -> Option<Result<String, String>> {
    if arg == REMOTE_FLAG {
        let missing =
            || format!("`{REMOTE_FLAG}` needs a value; `malvin admin remotes` lists them");
        return Some(
            rest.next()
                .and_then(|v| v.into_string().ok())
                .ok_or_else(missing),
        );
    }
    arg.strip_prefix(REMOTE_FLAG)?
        .strip_prefix('=')
        .map(|v| Ok(v.to_string()))
}

pub fn expand_remote_alias(
    value: &str,
    alias: impl Fn(&str) -> Option<String>,
) -> Result<String, String> {
    let (id, has_spec) = value
        .split_once('[')
        .map_or((value, false), |(id, _)| (id, true));
    if id.contains(':') {
        return Ok(value.to_string());
    }
    match (has_spec, alias(id.trim())) {
        (false, Some(expanded)) => Ok(expanded),
        (true, Some(_)) => Err(format!(
            "remote alias `{}` cannot take `[...]` suboptions: an alias matches only the whole value; put the suboptions in the alias, or write the full remote such as `{MODAL_REMOTE_ID}[...]`",
            id.trim()
        )),
        (_, None) => Ok(value.to_string()),
    }
}

pub fn extract_modal_options(
    raw: Vec<OsString>,
    alias: impl Fn(&str) -> Option<String>,
) -> Result<(Vec<OsString>, ModalOptions), String> {
    let mut opts = None;
    let mut out = Vec::with_capacity(raw.len());
    let mut scanning = true;
    let mut args = raw.into_iter();
    while let Some(arg) = args.next() {
        scanning &= arg.as_os_str() != "--";
        let value = arg
            .to_str()
            .filter(|_| scanning)
            .and_then(|text| remote_value(text, &mut args));
        let Some(value) = value else {
            out.push(arg);
            continue;
        };
        let typed = value?;
        let value = expand_remote_alias(&typed, &alias)?;
        let parsed = parse_remote_value(&value).map_err(|e| {
            if value == typed {
                e
            } else {
                format!("remote alias `{typed}` = `{value}`: {e}")
            }
        })?;
        if opts.replace(parsed).is_some() {
            return Err(format!("`{REMOTE_FLAG}` is given more than once"));
        }
        out.push(OsString::from(REMOTE_MODAL_ARG));
    }
    Ok((out, opts.unwrap_or_default()))
}
