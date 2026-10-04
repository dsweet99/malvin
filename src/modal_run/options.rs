use std::ffi::OsString;

use super::{MODAL_REMOTE, REMOTE_FLAG, REMOTE_MODAL_ARG};

pub const DEFAULT_NCPU: u64 = 1;
pub const DEFAULT_TIMEOUT_S: u64 = 30 * 60;
pub const DEFAULT_MEMORY_GB: u64 = 8;
pub const MAX_TIMEOUT_S: u64 = 24 * 3600;
const KEYS: &str = "gpu, ncpu, mem, timeout";

pub struct Suboption {
    pub key: &'static str,
    pub values: &'static str,
    pub default: &'static str,
    pub meaning: &'static [&'static str],
}

pub const SUBOPTIONS: [Suboption; 4] = [
    Suboption {
        key: "gpu",
        values: "none|TYPE[:COUNT]",
        default: "none",
        meaning: &["GPUs to attach, e.g. A100 or T4:2 (COUNT defaults to 1)", "TYPE: see GPU types below"],
    },
    Suboption {
        key: "ncpu",
        values: "N",
        default: "1",
        meaning: &["CPU cores; N is a positive whole number"],
    },
    Suboption {
        key: "mem",
        values: "N[G|GB|GiB]",
        default: "8",
        meaning: &["memory in GiB; N is a positive whole number, e.g. 16 or 16G"],
    },
    Suboption {
        key: "timeout",
        values: "N[s|m|h]",
        default: "30m",
        meaning: &["Sandbox lifetime; a bare N is minutes; at most 24h", "e.g. 90s, 45m, or 2h"],
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuChoice {
    None,
    Gpu(String),
}

impl GpuChoice {
    #[must_use]
    pub fn into_option(self) -> Option<String> {
        match self {
            Self::None => None,
            Self::Gpu(gpu) => Some(gpu),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModalOptions {
    pub gpu: Option<GpuChoice>,
    pub ncpu: Option<u64>,
    pub memory_gb: Option<u64>,
    pub timeout_s: Option<u64>,
}

pub fn parse_gpu(raw: &str) -> Result<GpuChoice, String> {
    let raw = raw.trim();
    if raw.eq_ignore_ascii_case("none") {
        return Ok(GpuChoice::None);
    }
    let (kind, count) = raw.split_once(':').unwrap_or((raw, "1"));
    let kind_ok = !kind.is_empty() && kind.chars().all(|c| c.is_ascii_alphanumeric() || "-!+".contains(c));
    let count_ok = count.parse::<u64>().is_ok_and(|n| n > 0);
    if kind_ok && count_ok {
        return Ok(GpuChoice::Gpu(raw.to_string()));
    }
    Err(format!(
        "gpu `{raw}` is not valid: use `none`, a GPU type such as `A100`, or TYPE:COUNT such as `T4:2`"
    ))
}

pub fn parse_ncpu(raw: &str) -> Result<u64, String> {
    match raw.trim().parse::<u64>() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(format!("ncpu `{raw}` is not valid: use a positive whole number of CPUs")),
    }
}

pub fn parse_memory(raw: &str) -> Result<u64, String> {
    let text = raw.trim();
    let lower = text.to_ascii_lowercase();
    let digits = ["gib", "gb", "g"]
        .iter()
        .find_map(|unit| lower.strip_suffix(unit))
        .unwrap_or(&lower);
    match digits.parse::<u64>() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(format!(
            "mem `{raw}` is not valid: use a positive whole number of GiB, such as `8` or `16G`"
        )),
    }
}

pub fn parse_timeout(raw: &str) -> Result<u64, String> {
    let text = raw.trim();
    let (digits, unit_s) = match text.char_indices().last() {
        Some((i, 's')) => (&text[..i], 1),
        Some((i, 'm')) => (&text[..i], 60),
        Some((i, 'h')) => (&text[..i], 3600),
        _ => (text, 60),
    };
    let secs = digits
        .parse::<u64>()
        .ok()
        .and_then(|n| n.checked_mul(unit_s))
        .filter(|&s| s > 0)
        .ok_or_else(|| {
            format!("timeout `{raw}` is not valid: use minutes (`30`) or a number with s, m, or h (`90s`, `45m`, `2h`)")
        })?;
    if secs > MAX_TIMEOUT_S {
        return Err(format!("timeout `{raw}` is too long: Modal Sandboxes live at most 24 hours"));
    }
    Ok(secs)
}

pub fn parse_modal_spec(spec: &str) -> Result<ModalOptions, String> {
    let mut opts = ModalOptions::default();
    for item in spec.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let (key, value) = item
            .split_once('=')
            .ok_or_else(|| format!("`{item}` needs the form key=value (keys: {KEYS})"))?;
        if set_suboption(&mut opts, key.trim(), value)? {
            return Err(format!("suboption `{}` is given more than once", key.trim()));
        }
    }
    Ok(opts)
}

fn set_suboption(opts: &mut ModalOptions, key: &str, value: &str) -> Result<bool, String> {
    Ok(match key {
        "gpu" => opts.gpu.replace(parse_gpu(value)?).is_some(),
        "ncpu" => opts.ncpu.replace(parse_ncpu(value)?).is_some(),
        "mem" => opts.memory_gb.replace(parse_memory(value)?).is_some(),
        "timeout" => opts.timeout_s.replace(parse_timeout(value)?).is_some(),
        other => return Err(format!("unknown suboption `{other}` (keys: {KEYS})")),
    })
}

pub fn parse_remote_value(value: &str) -> Result<ModalOptions, String> {
    let (name, spec) = match value.split_once('[') {
        Some((name, rest)) => {
            let spec = rest
                .strip_suffix(']')
                .ok_or_else(|| format!("`{REMOTE_FLAG}={value}` is missing its closing `]`"))?;
            (name, Some(spec))
        }
        None => (value, None),
    };
    if name != MODAL_REMOTE {
        return Err(format!(
            "unknown remote `{name}`; `malvin admin remotes` lists the available remotes, and `[aliases.remotes]` in ~/.malvinconf/config.toml defines aliases"
        ));
    }
    spec.map_or_else(
        || Ok(ModalOptions::default()),
        |spec| parse_modal_spec(spec).map_err(|e| format!("`{REMOTE_FLAG}={MODAL_REMOTE}[...]`: {e}")),
    )
}

fn remote_value(arg: &str, rest: &mut impl Iterator<Item = OsString>) -> Option<Result<String, String>> {
    if arg == REMOTE_FLAG {
        let missing = || format!("`{REMOTE_FLAG}` needs a value; `malvin admin remotes` lists them");
        return Some(rest.next().and_then(|v| v.into_string().ok()).ok_or_else(missing));
    }
    arg.strip_prefix(REMOTE_FLAG)?.strip_prefix('=').map(|v| Ok(v.to_string()))
}

pub fn expand_remote_alias(value: &str, alias: impl FnOnce(&str) -> Option<String>) -> String {
    let name = value.split_once('[').map_or(value, |(name, _)| name);
    if name == MODAL_REMOTE {
        return value.to_string();
    }
    alias(value.trim()).unwrap_or_else(|| value.to_string())
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
        let value = expand_remote_alias(&typed, &alias);
        let parsed = parse_remote_value(&value).map_err(|e| {
            if value == typed { e } else { format!("remote alias `{typed}` = `{value}`: {e}") }
        })?;
        if opts.replace(parsed).is_some() {
            return Err(format!("`{REMOTE_FLAG}` is given more than once"));
        }
        out.push(OsString::from(REMOTE_MODAL_ARG));
    }
    Ok((out, opts.unwrap_or_default()))
}
