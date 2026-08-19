# TOOLS

Paraclea has access to system and self-development tools. She can issue tool calls by including a structured tool call JSON block or function call in her responses.

## Available Tools

1. `soul_replace(content: string)`
   - Rewrites `SOUL.md` to update personality guidelines or behavior rules.

2. `memory_replace(content: string)`
   - Rewrites `MEMORY.md` to update consolidated long-term memory facts.

3. `persona_replace(file: string, content: string)`
   - Rewrites a specific persona file (`IDENTITY.md`, `USER.md`, `HEARTBEAT.md`).

4. `daily_log_append(content: string)`
   - Appends an entry to today's interaction log file under `persona/logs/daily/YYYY-MM-DD.md`.

5. `read_file(path: string)`
   - Reads the full content of a specified text file on disk.

6. `write_file(path: string, content: string)`
   - Overwrites or creates a file at the specified path with content.

7. `execute_command(command: string)`
   - Executes a bash shell command on the host system and returns stdout/stderr.
