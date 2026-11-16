# Connect an agent to Orchestrata

This local, read-only example lists recipes and previews a lead decision. It does not run a solve or create CRM records. The website does not host the solver or an MCP endpoint. A chat subscription may not include API access; your client may require its own sign-in or workspace approval.

## 1. Prepare the tool

In a Linux or macOS shell with GitHub CLI and Cargo:

```sh
gh repo clone ilstarno/Orchestrata
cd Orchestrata
git switch --detach c841b36ad4d8fd1f246a5cefe96a05a72d8784fe
cargo build --locked --manifest-path backend/Cargo.toml -p orchestrata-mcp
ORCHESTRATA_MCP="$PWD/backend/target/debug/orchestrata-mcp"
test -x "$ORCHESTRATA_MCP"
```

Build the executable where the agent runs. The agent starts it; don't launch it separately or pass it a transport flag.

## 2. Connect your client

### Codex CLI

```sh
codex mcp add orchestrata -- "$ORCHESTRATA_MCP"
codex mcp get orchestrata
```

Confirm that `orchestrata` is enabled. Use `/mcp` in an active Codex session. [Official Codex instructions](https://learn.chatgpt.com/docs/extend/mcp?surface=cli).

### Claude Code

From your project folder:

```sh
claude mcp add --transport stdio --scope local orchestrata -- "$ORCHESTRATA_MCP"
claude mcp list
```

Start Claude Code in the same folder and check `/mcp`. [Official Claude Code instructions](https://code.claude.com/docs/en/mcp).

### Cursor

Merge the `orchestrata` entry into your project's `.cursor/mcp.json`; don't overwrite other tools. Replace the path with the executable's absolute path.

```json
{
  "mcpServers": {
    "orchestrata": {
      "type": "stdio",
      "command": "/absolute/path/to/Orchestrata/backend/target/debug/orchestrata-mcp",
      "args": []
    }
  }
}
```

Run `agent mcp list` and `agent mcp list-tools orchestrata`. [Official Cursor instructions](https://cursor.com/docs/mcp).

### Grok Bot

Build the executable in the approved Grok Bot cloud computer, not on your laptop. Select **Connect apps → Add a custom connector → Command**, specify the cloud-side executable with no arguments, and check the installed list. Check who can use the connector before sharing. [Official Grok Bot instructions](https://docs.x.ai/grok-bot/computer-and-apps#connect-an-app).

### Grok Build CLI

```sh
grok mcp add orchestrata -- "$ORCHESTRATA_MCP"
grok mcp list
grok mcp doctor orchestrata
```

Use `/mcps` to see the tools. [Official Grok Build instructions](https://docs.x.ai/build/features/mcp-servers).

## 3. Preview one result

Use this prompt with the connected client:

> Use list_lead_examples, then preview_lead_example with lead_id L-001. Show the outcome, assigned representative and contact ID. Do not create a record or send it anywhere.

Expected preview: REP-08, contact CT-001, tool outcome `created` (would create a contact; no record is written). Use only sample or anonymized inputs when testing with a hosted model.

### Run the same check without a model account

Save [the local verifier](verify_mcp.py) and run:

```sh
python3 verify_mcp.py "$ORCHESTRATA_MCP" --revision c841b36ad4d8fd1f246a5cefe96a05a72d8784fe
```

See the [recorded response](verified-stdio.txt). The inline terminal images show actual Cargo output, Codex CLI registration, and local Orchestrata tool output, presented as excerpts. They do not show a signed-in agent conversation. The Claude Code, Cursor and Grok action cards summarize the vendor instructions; they are not captures from those clients.

For assignments over HTTP, use [the separate solve API](../docs/solve-api/), `POST /api/v1/solve`.
