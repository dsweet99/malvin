#!/bin/sh
rpc_id() { printf '%s' "$1" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p'; }
while IFS= read -r line; do
  id=$(rpc_id "$line")
  case "$line" in
    *model/list*)
      printf '%s\n' "{\"id\":${id:-2},\"result\":{\"data\":[{\"id\":\"gpt-test\"},{\"id\":\"gpt-5.6-sol\"}]}}"
      ;;
    *initialize*)
      printf '%s\n' "{\"id\":${id:-1},\"result\":{}}"
      ;;
    *thread/start*)
      case "$line" in
        *\"ephemeral\":true*\"sandbox\":\"workspace-write\"*|*\"ephemeral\":true*\"sandbox\":\"danger-full-access\"*)
          case "$line" in
            *gpt-5.6-sol*) printf '%s\n' "{\"id\":${id:-2},\"result\":{\"thread\":{\"id\":\"thread-test\"}}}" ;;
            *) printf '%s\n' "{\"id\":${id:-2},\"error\":{\"message\":\"wrong model\"}}" ;;
          esac
          ;;
        *) printf '%s\n' "{\"id\":${id:-2},\"error\":{\"message\":\"missing sandbox\"}}" ;;
      esac
      ;;
    *turn/start*)
      printf '%s\n' "{\"id\":${id:-3},\"result\":{\"turn\":{\"id\":\"turn-test\",\"status\":\"inProgress\"}}}"
      printf '%s\n' '{"method":"turn/started","params":{"turn":{"id":"turn-test"}}}'
      if [ -n "${MALVIN_CODEX_HANG:-}" ]; then sleep 30; continue; fi
      if [ -n "${MALVIN_CODEX_FAIL_TURN:-}" ]; then
        printf '%s\n' '{"method":"turn/completed","params":{"turn":{"id":"turn-test","status":"failed","error":{"message":"auth"}}}}'
      else
        printf '%s\n' '{"method":"item/agentMessage/delta","params":{"turnId":"turn-test","delta":"hello"}}'
        printf '%s\n' '{"method":"item/completed","params":{"turnId":"turn-test","item":{"id":"a1","type":"agentMessage","text":"hello"}}}'
        printf '%s\n' '{"method":"turn/completed","params":{"turn":{"id":"turn-test","status":"completed","items":[{"type":"agentMessage","text":"hello"}]}}}'
      fi
      ;;
    *turn/interrupt*)
      case "$line" in
        *turnId*) printf '%s\n' "{\"id\":${id:-4},\"result\":{}}" ;;
        *) printf '%s\n' "{\"id\":${id:-4},\"error\":{\"code\":-32600,\"message\":\"missing field turnId\"}}" ;;
      esac
      ;;
    *thread/delete*)
      printf '%s\n' "{\"id\":${id:-5},\"result\":{}}"
      ;;
  esac
done
