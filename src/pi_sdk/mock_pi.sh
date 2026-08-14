#!/bin/sh
# Offline mock for `pi --rpc` (unit tests). Speaks Pi JSONL RPC.
# Usage: mock_pi.sh --rpc ... (flags ignored)

set -eu

for arg in "$@"; do
  case "$arg" in
    --version|-v)
      printf '%s\n' "pi 0.1.23 (mock)"
      exit 0
      ;;
  esac
done

while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
  typ=$(printf '%s' "$line" | sed -n 's/.*"type"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
  case "$typ" in
    new_session)
      if [ "${MOCK_PI_HANG_NEW_SESSION:-}" = "1" ]; then
        sleep 60
      fi
      printf '%s\n' "{\"id\":\"$id\",\"type\":\"response\",\"command\":\"new_session\",\"success\":true,\"data\":{}}"
      ;;
    prompt)
      msg=$(printf '%s' "$line" | sed -n 's/.*"message"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
      case "$msg" in
        *AGENT_END_BEFORE_ACK*)
          # agent_end before prompt response ACK (ordering regression).
          printf '%s\n' "{\"type\":\"agent_end\",\"error\":null,\"messages\":[{\"role\":\"user\",\"content\":\"$msg\"},{\"role\":\"assistant\",\"content\":[{\"type\":\"text\",\"text\":\"early-end\"}],\"usage\":{\"input\":1,\"output\":1,\"cacheRead\":0,\"cacheWrite\":0}}]}"
          printf '%s\n' "{\"id\":\"$id\",\"type\":\"response\",\"command\":\"prompt\",\"success\":true,\"data\":{}}"
          ;;
        *EMPTY_ASSISTANT_RESULT*)
          printf '%s\n' "{\"id\":\"$id\",\"type\":\"response\",\"command\":\"prompt\",\"success\":true,\"data\":{}}"
          printf '%s\n' '{"type":"agent_start","sessionId":"mock"}'
          # No assistant text → map_agent_end result is None.
          printf '%s\n' "{\"type\":\"agent_end\",\"error\":null,\"messages\":[{\"role\":\"user\",\"content\":\"$msg\"}]}"
          ;;
        *)
          printf '%s\n' "{\"id\":\"$id\",\"type\":\"response\",\"command\":\"prompt\",\"success\":true,\"data\":{}}"
          printf '%s\n' '{"type":"agent_start","sessionId":"mock"}'
          printf '%s\n' "{\"type\":\"message_update\",\"assistantMessageEvent\":{\"type\":\"text_delta\",\"delta\":\"echo:$msg\"}}"
          printf '%s\n' '{"type":"tool_execution_start","toolCallId":"t1","toolName":"ls"}'
          printf '%s\n' '{"type":"tool_execution_end","toolCallId":"t1","toolName":"ls"}'
          printf '%s\n' "{\"type\":\"agent_end\",\"error\":null,\"messages\":[{\"role\":\"user\",\"content\":\"$msg\"},{\"role\":\"assistant\",\"content\":[{\"type\":\"text\",\"text\":\"echo:$msg\"}],\"usage\":{\"input\":3,\"output\":2,\"cacheRead\":0,\"cacheWrite\":0}}]}"
          ;;
      esac
      ;;
    abort)
      printf '%s\n' "{\"id\":\"$id\",\"type\":\"response\",\"command\":\"abort\",\"success\":true,\"data\":{}}"
      ;;
    *)
      printf '%s\n' "{\"id\":\"$id\",\"type\":\"response\",\"command\":\"$typ\",\"success\":false,\"error\":\"unknown command\"}"
      ;;
  esac
done
