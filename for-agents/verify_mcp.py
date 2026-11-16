"""Exercise actual local Orchestrata stdio using Python's standard library."""

import argparse
import json
from pathlib import Path
from subprocess import PIPE, Popen


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("binary", type=Path)
    args = parser.parse_args()

    print("Orchestrata MCP · local, synthetic examples only", flush=True)
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
        assert {"list_recipes", "describe_recipe", "get_recipe_example"} <= set(tools)
        for name in tools:
            print("tools/list →", name, flush=True)

        def read_tool(index, name, arguments):
            result = call(index, "tools/call", {"name": name, "arguments": arguments})
            if result.get("isError"):
                raise RuntimeError(f"{name} did not return a readable result")
            return result["structuredContent"]

        catalog = read_tool(3, "list_recipes", {})["recipes"]
        assert catalog, "The catalog should contain at least one recipe."
        recipe_id = catalog[0]["id"]
        description = read_tool(4, "describe_recipe", {"id": recipe_id})
        example = read_tool(5, "get_recipe_example", {"id": recipe_id})
        assert description["id"] == recipe_id and {"inputs", "outputs"} <= set(description)
        assert example["id"] == recipe_id and example.get("synthetic") is True
        assert isinstance(example["problem"], dict)
        print(f"PASS · {len(catalog)} recipes. Read fields and example for {recipe_id}.", flush=True)

        assert process.stdin is not None
        process.stdin.close()
        if process.wait(timeout=5):
            raise RuntimeError("Server did not exit cleanly")


if __name__ == "__main__":
    main()
