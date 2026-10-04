use std::ffi::OsString;

use super::MODAL_FLAG;

pub const DEFAULT_NCPU: u64 = 1;
pub const DEFAULT_TIMEOUT_S: u64 = 30 * 60;
pub const MAX_TIMEOUT_S: u64 = 24 * 3600;
const KEYS: &str = "gpu, ncpu, timeout";

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
    pub timeout_s: Option<u64>,
}

pub fn parse_gpu(raw: &str) -> Result<GpuChoice, String> {
    let raw = raw.trim();
    if raw.eq_ignore_ascii_case("none") {
        return Ok(GpuChoice::None);
    }
    let (kind, count) = raw.split_once(':').unwrap_or((raw, "1"));
    let kind_ok = !kind.is_empty() && kind.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
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
        let duplicate = match key.trim() {
            "gpu" => opts.gpu.replace(parse_gpu(value)?).is_some(),
            "ncpu" => opts.ncpu.replace(parse_ncpu(value)?).is_some(),
            "timeout" => opts.timeout_s.replace(parse_timeout(value)?).is_some(),
            other => return Err(format!("unknown suboption `{other}` (keys: {KEYS})")),
        };
        if duplicate {
            return Err(format!("suboption `{}` is given more than once", key.trim()));
        }
    }
    Ok(opts)
}

fn bracketed_spec(arg: &str) -> Option<Result<&str, String>> {
    let rest = arg.strip_prefix(MODAL_FLAG)?.strip_prefix('[')?;
    Some(
        rest.strip_suffix(']')
            .ok_or_else(|| format!("`{arg}` is missing its closing `]`")),
    )
}

pub fn extract_modal_options(raw: Vec<OsString>) -> Result<(Vec<OsString>, ModalOptions), String> {
    let mut opts = None;
    let mut out = Vec::with_capacity(raw.len());
    let mut scanning = true;
    for arg in raw {
        scanning &= arg.as_os_str() != "--";
        let spec = arg.to_str().filter(|_| scanning).and_then(bracketed_spec);
        let Some(spec) = spec else {
            out.push(arg);
            continue;
        };
        let parsed = parse_modal_spec(spec?).map_err(|e| format!("`--modal[...]`: {e}"))?;
        if opts.replace(parsed).is_some() {
            return Err("`--modal[...]` is given more than once".to_string());
        }
        out.push(OsString::from(MODAL_FLAG));
    }
    Ok((out, opts.unwrap_or_default()))
}
