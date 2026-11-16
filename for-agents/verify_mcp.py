"""Exercise actual local Orchestrata stdio using Python's standard library."""

import argparse
import json
from pathlib import Path
from subprocess import PIPE, Popen


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("binary", type=Path)
    parser.add_argument("--revision", required=True, help="Git revision used to build the binary")
    args = parser.parse_args()

    print("Orchestrata MCP · local, synthetic examples only", flush=True)
    print(f"Build revision: {args.revision}", flush=True)
    with Popen([str(args.binary.resolve())], stdin=PIPE, stdout=PIPE, text=True) as process:
        def send(request):
            assert process.stdin is not None
            process.stdin.write(json.dumps(request, separators=(",", ":")) + "\n")
            process.stdin.flush()

        def call(index, method, params):
            send({"jsonrpc": "2.0", "id": index, "method": method, "params": params})
            response = json.loads(process.stdout.readline())
            if response.get("id") != index or "error" in response:
                raise RuntimeError(f"{method} did not return the expected MCP response")
            return response["result"]

        init = call(1, "initialize", {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "orchestrata-local-check", "version": "1"},
        })
        print("initialize →", init["protocolVersion"], flush=True)

        send({"jsonrpc": "2.0", "method": "notifications/initialized"})
        tools = [tool["name"] for tool in call(2, "tools/list", {})["tools"]]
        assert "list_lead_examples" in tools and "preview_lead_example" in tools
        for name in tools:
            print("tools/list →", name, flush=True)

        catalog = call(3, "tools/call", {"name": "list_lead_examples", "arguments": {}})
        listing = catalog["structuredContent"]
        ids = listing["lead_ids"]
        assert not catalog.get("isError") and listing["synthetic"] is True
        assert [ids[0], ids[-1], len(ids)] == ["L-001", "L-060", 60]
        print(f"list_lead_examples → {len(ids)} synthetic IDs ({ids[0]} to {ids[-1]})", flush=True)

        preview = call(4, "tools/call", {
            "name": "preview_lead_example", "arguments": {"lead_id": ids[0]},
        })
        decision = preview["structuredContent"]
        assert not preview.get("isError")
        assert decision == {
            "lead_id": "L-001", "outcome": "created", "reason": "new",
            "assigned_representative_id": "REP-08", "contact_id": "CT-001",
        }
        print("preview_lead_example →", json.dumps(decision, indent=2), flush=True)
        print("PASS · no real leads or records were created", flush=True)

        assert process.stdin is not None
        process.stdin.close()
        if process.wait(timeout=5):
            raise RuntimeError("Server did not exit cleanly")


if __name__ == "__main__":
    main()
