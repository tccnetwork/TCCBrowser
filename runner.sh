#!/usr/bin/env bash
# runner.sh — file-based command runner for Claude (allowlist-gated).
#
# WHY THIS EXISTS
#   Claude Code's sandbox (or a CI/remote agent runtime) may not let the
#   assistant execute shell commands directly. This script offers a host-side
#   escape hatch: Claude drops a `.cmd` file into .cmd-queue/, this daemon
#   picks it up, validates against an allowlist, executes, and writes the
#   captured stdout/stderr to .cmd-results/<id>.log for Claude to read back.
#
# USAGE
#   bash runner.sh                # foreground watch (Ctrl+C to stop)
#   bash runner.sh start          # spawn daemon in background (idempotent)
#   bash runner.sh stop           # kill the daemon if running
#   bash runner.sh status         # report running/stopped + PID
#   bash runner.sh exec "<cmd>"   # ensure-running, enqueue, wait, print result
#
#   `exec` is the one-shot Claude-friendly form: it auto-starts the daemon
#   if needed, drops the command, polls for the result file (default 30s
#   timeout via $RUNNER_EXEC_TIMEOUT), prints it, and returns the exit code.
#
# SECURITY
#   - Only commands matching ALLOWLIST_EXACT (whole-line equality) or
#     ALLOWLIST_PREFIX (prefix match) run. Anything else logs REJECTED.
#   - Edit the arrays below to add commands. Each addition widens the trust
#     boundary — review what the command could do with the daemon's privileges.
#   - The daemon runs in this script's directory (cd "$(dirname "$0")").
#     Relative paths in commands resolve from there.
#   - Audit trail: every receipt/exec/reject is appended to
#     .cmd-results/audit.log with timestamp.

set -u
cd "$(dirname "$0")"

# Commands allowed by whole-line exact match. Add yours below.
declare -a ALLOWLIST_EXACT=(
  # === Example entries (uncomment / replace with your project's commands) ===
  # "npm test"
  # "npm run build"
  # "npx tsc --noEmit"
  # "cargo build"
  # "cargo test"
  # "pytest -q"
  # "make check"
)

# Commands allowed by prefix match. Useful for read-only utilities + tools
# whose argument list is variable (grep patterns, file paths, etc.).
declare -a ALLOWLIST_PREFIX=(
  # Read-only utilities (safe, enabled by default for a_tcc)
  "ls "
  "cat "
  "head "
  "tail "
  "grep "
  "wc "
  "find "
  "sqlite3 logs/devlog.sqlite "
  # === Enable when the project defines them (build/test) ===
  # "npm test"
  # "npm run build"
  # "npx tsc --noEmit"
  # "curl -sS http://localhost:"
)

QUEUE_DIR=".cmd-queue"
RESULT_DIR=".cmd-results"
PID_FILE="$RESULT_DIR/runner.pid"
DAEMON_LOG="$RESULT_DIR/daemon.log"
mkdir -p "$QUEUE_DIR" "$RESULT_DIR"
AUDIT="$RESULT_DIR/audit.log"

is_allowed() {
  local cmd="$1"
  # The `${arr[@]+"${arr[@]}"}` idiom safely expands empty arrays under `set -u`
  # on macOS's stock bash 3.2 (which otherwise errors with "unbound variable").
  for a in ${ALLOWLIST_EXACT[@]+"${ALLOWLIST_EXACT[@]}"}; do
    if [[ "$cmd" == "$a" ]]; then
      return 0
    fi
  done
  for p in ${ALLOWLIST_PREFIX[@]+"${ALLOWLIST_PREFIX[@]}"}; do
    if [[ "$cmd" == "$p"* ]]; then
      return 0
    fi
  done
  return 1
}

run_one() {
  local cmd_file="$1"
  local id; id="$(basename "$cmd_file" .cmd)"
  local cmd; cmd="$(head -n1 "$cmd_file")"
  local result_file="$RESULT_DIR/$id.log"
  local stamp; stamp="$(date '+%Y-%m-%d %H:%M:%S')"

  echo "[$stamp] received: $cmd (id=$id)" | tee -a "$AUDIT"

  if ! is_allowed "$cmd"; then
    {
      echo "=== id=$id @ $stamp ==="
      echo "$ $cmd"
      echo "REJECTED: command not in allowlist"
      echo "exit: 126"
    } > "$result_file"
    echo "[$stamp] REJECTED: $cmd" | tee -a "$AUDIT"
    rm "$cmd_file"
    return
  fi

  {
    echo "=== id=$id @ $stamp ==="
    echo "$ $cmd"
    echo "---"
  } > "$result_file"

  eval "$cmd" >> "$result_file" 2>&1
  local rc=$?

  {
    echo "---"
    echo "exit: $rc"
  } >> "$result_file"

  echo "[$stamp] done id=$id exit=$rc" | tee -a "$AUDIT"
  rm "$cmd_file"
}

watch_loop() {
  echo "runner.sh watching $QUEUE_DIR/  (PID $$)"
  echo "    EXACT allowlist:  ${#ALLOWLIST_EXACT[@]} commands"
  echo "    PREFIX allowlist: ${#ALLOWLIST_PREFIX[@]} patterns"
  echo "    Audit log:        $AUDIT"

  while true; do
    for f in "$QUEUE_DIR"/*.cmd; do
      [[ -f "$f" ]] || continue
      run_one "$f"
    done
    sleep 1
  done
}

# Return 0 if daemon is alive (PID file exists AND process running), else 1.
# Cleans up stale PID file as a side effect.
is_running() {
  [[ -f "$PID_FILE" ]] || return 1
  local pid; pid="$(cat "$PID_FILE" 2>/dev/null || echo)"
  if [[ -z "$pid" ]] || ! kill -0 "$pid" 2>/dev/null; then
    rm -f "$PID_FILE"
    return 1
  fi
  return 0
}

cmd_start() {
  if is_running; then
    echo "runner: already running (PID $(cat "$PID_FILE"))"
    return 0
  fi
  # Spawn this script in foreground-watch mode, detached.
  nohup "$BASH" "$0" >> "$DAEMON_LOG" 2>&1 &
  local pid=$!
  echo "$pid" > "$PID_FILE"
  # Give it a moment to either crash or settle.
  sleep 0.3
  if kill -0 "$pid" 2>/dev/null; then
    echo "runner: started (PID $pid, log $DAEMON_LOG)"
  else
    rm -f "$PID_FILE"
    echo "runner: failed to start — check $DAEMON_LOG"
    return 1
  fi
}

cmd_stop() {
  if ! is_running; then
    echo "runner: not running"
    return 0
  fi
  local pid; pid="$(cat "$PID_FILE")"
  kill "$pid" 2>/dev/null || true
  # Wait briefly for clean shutdown.
  local i=0
  while kill -0 "$pid" 2>/dev/null && (( i < 20 )); do
    sleep 0.1
    i=$((i + 1))
  done
  if kill -0 "$pid" 2>/dev/null; then
    kill -9 "$pid" 2>/dev/null || true
  fi
  rm -f "$PID_FILE"
  echo "runner: stopped (was PID $pid)"
}

cmd_status() {
  if is_running; then
    echo "runner: RUNNING (PID $(cat "$PID_FILE"))"
    echo "    queue:  $QUEUE_DIR/"
    echo "    audit:  $AUDIT"
    return 0
  fi
  echo "runner: stopped"
  return 1
}

cmd_exec() {
  local cmd="${1:-}"
  if [[ -z "$cmd" ]]; then
    echo "usage: runner.sh exec \"<command>\"" >&2
    return 2
  fi
  is_running || cmd_start >/dev/null || {
    echo "runner: cannot start daemon" >&2
    return 1
  }

  local id="exec-$(date +%s)-$$"
  local result_file="$RESULT_DIR/$id.log"
  echo "$cmd" > "$QUEUE_DIR/$id.cmd"

  local timeout="${RUNNER_EXEC_TIMEOUT:-30}"
  local waited=0
  while [[ ! -f "$result_file" ]] && (( waited < timeout * 10 )); do
    sleep 0.1
    waited=$((waited + 1))
  done

  if [[ ! -f "$result_file" ]]; then
    echo "runner: timeout after ${timeout}s waiting for $id" >&2
    return 124
  fi

  cat "$result_file"
  local last_line; last_line="$(tail -n1 "$result_file")"
  case "$last_line" in
    "exit: "*) return "${last_line#exit: }" ;;
    *) return 0 ;;
  esac
}

case "${1:-}" in
  start)  shift; cmd_start "$@" ;;
  stop)   shift; cmd_stop "$@" ;;
  status) shift; cmd_status "$@" ;;
  exec)   shift; cmd_exec "$@" ;;
  "")     watch_loop ;;
  *)
    echo "usage: $0 [start|stop|status|exec \"<command>\"]" >&2
    echo "       (no arg = foreground watch)" >&2
    exit 2
    ;;
esac
