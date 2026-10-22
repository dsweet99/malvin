export function claudeUsageRecord(usage) {
    const number = (key) => {
        const value = usage[key];
        return typeof value === "number" ? value : undefined;
    };
    const fields = [
        ["inputTokens", number("input_tokens")],
        ["outputTokens", number("output_tokens")],
        ["cacheReadTokens", number("cache_read_input_tokens")],
        ["cacheWriteTokens", number("cache_creation_input_tokens")],
        ["reasoningTokens", number("thinking_tokens")],
    ];
    return Object.fromEntries(fields.filter(([, value]) => value !== undefined));
}
