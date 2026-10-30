pub(crate) fn sdk_tool_kind(name: Option<&str>) -> &'static str {
    match name.unwrap_or("").to_ascii_lowercase().as_str() {
        "read" | "read_file" | "imageview" => "read",
        "grep" | "search" | "glob" | "find" | "rg" | "websearch" | "webfetch" => "search",
        "shell" | "bash" | "powershell" | "execute" => "execute",
        "edit" | "write" | "strreplace" | "delete" | "applypatch" => "edit",
        _ => "other",
    }
}
