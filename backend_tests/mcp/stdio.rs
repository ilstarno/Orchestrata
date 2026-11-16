use std::{process::Stdio, time::Duration};

use serde_json::{Value, json};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{ChildStdin, ChildStdout, Command},
    time::timeout,
};

const LIMIT: Duration = Duration::from_secs(8);

#[tokio::test]
async fn initializes_and_exposes_only_read_only_built_in_examples() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_orchestrata-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut writer = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());

    send(
        &mut writer,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "local-recipe-smoke", "version": "1"},
            }
        }),
    )
    .await;
    let init = response(&mut reader, 1).await;
    assert_eq!(init["result"]["protocolVersion"], "2025-11-25");
    assert!(init["result"]["capabilities"]["tools"].is_object());
    send(
        &mut writer,
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
    )
    .await;

    send(
        &mut writer,
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
    )
    .await;
    let listed = response(&mut reader, 2).await;
    let tools = listed["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 5);
    for name in [
        "list_recipes",
        "describe_recipe",
        "get_recipe_example",
        "list_lead_examples",
        "preview_lead_example",
    ] {
        let tool = tools.iter().find(|t| t["name"] == name).unwrap();
        assert_eq!(tool["annotations"]["readOnlyHint"], true);
        assert_eq!(tool["annotations"]["openWorldHint"], false);
    }

    let list = call(&mut writer, &mut reader, 3, "list_recipes", json!({})).await;
    let entries = list["result"]["structuredContent"]["recipes"]
        .as_array()
        .unwrap();
    assert_eq!(entries.len(), orchestrata_recipes::builtin_recipes().len());
    assert!(entries.iter().any(|entry| entry["id"] == "field-service"));

    let describe = call(
        &mut writer,
        &mut reader,
        4,
        "describe_recipe",
        json!({"id":"field-service"}),
    )
    .await;
    let description = &describe["result"]["structuredContent"];
    assert_eq!(description["id"], "field-service");
    assert_eq!(description["inputs"][0]["type"], "string-list");

    let example = call(
        &mut writer,
        &mut reader,
        5,
        "get_recipe_example",
        json!({"id":"field-service"}),
    )
    .await;
    let entry = &example["result"]["structuredContent"];
    let expected: Value = serde_json::from_str(
        orchestrata_recipes::builtin_recipe("field-service")
            .unwrap()
            .problem_json(),
    )
    .unwrap();
    assert_eq!(entry["synthetic"], true);
    assert_eq!(entry["problem"], expected);

    let rejected = call(
        &mut writer,
        &mut reader,
        6,
        "get_recipe_example",
        json!({"id":"not-a-recipe"}),
    )
    .await;
    assert_eq!(rejected["result"]["isError"], true);
    assert!(rejected["result"]["structuredContent"].is_null());
    assert_lead_preview_tools(&mut writer, &mut reader).await;
    drop(writer);
    assert!(
        timeout(LIMIT, child.wait())
            .await
            .unwrap()
            .unwrap()
            .success()
    );
}

async fn assert_lead_preview_tools(writer: &mut ChildStdin, reader: &mut BufReader<ChildStdout>) {
    let expected: Vec<Value> =
        serde_json::from_str(include_str!("../api/lead-preview-expected.json")).unwrap();
    let ids = expected
        .iter()
        .map(|example| example["lead_id"].clone())
        .collect::<Vec<_>>();
    let listed = call(writer, reader, 7, "list_lead_examples", json!({})).await;
    assert_eq!(
        listed["result"]["structuredContent"],
        json!({"synthetic": true, "lead_ids": ids})
    );

    for (index, expected) in expected.iter().enumerate() {
        let lead_id = expected["lead_id"].as_str().unwrap();
        let listed = call(
            writer,
            reader,
            100 + index as u64,
            "preview_lead_example",
            json!({"lead_id": lead_id}),
        )
        .await;
        assert_eq!(
            &listed["result"]["structuredContent"], expected,
            "{lead_id}"
        );
        assert!(
            !listed["result"]["structuredContent"]
                .to_string()
                .contains('@')
        );
    }

    let unknown = call(
        writer,
        reader,
        8,
        "preview_lead_example",
        json!({"lead_id":"L-061"}),
    )
    .await;
    assert_eq!(unknown["result"]["isError"], true);
    assert_eq!(
        unknown["result"]["structuredContent"],
        json!({"code":"unknown_lead_example"})
    );

    for (index, (tool, arguments)) in [
        (
            "preview_lead_example",
            json!({"lead_id":"L-001", "email":"not-an-input-field"}),
        ),
        ("preview_lead_example", json!({"lead_id":1})),
        ("list_lead_examples", json!({"search":"not-an-input-field"})),
    ]
    .into_iter()
    .enumerate()
    {
        let rejected = call(writer, reader, 20 + index as u64, tool, arguments).await;
        assert!(rejected.get("error").is_some() || rejected["result"]["isError"] == true);
        assert!(rejected["result"]["structuredContent"].is_null());
    }
}

async fn send(writer: &mut ChildStdin, value: Value) {
    let mut data = serde_json::to_vec(&value).unwrap();
    data.push(b'\n');
    timeout(LIMIT, writer.write_all(&data))
        .await
        .unwrap()
        .unwrap();
}

async fn response(reader: &mut BufReader<ChildStdout>, id: u64) -> Value {
    let mut line = String::new();
    assert!(
        timeout(LIMIT, reader.read_line(&mut line))
            .await
            .unwrap()
            .unwrap()
            > 0
    );
    let value: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(value["id"], id);
    value
}

async fn call(
    writer: &mut ChildStdin,
    reader: &mut BufReader<ChildStdout>,
    id: u64,
    name: &str,
    arguments: Value,
) -> Value {
    send(
        writer,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": { "name": name, "arguments": arguments },
        }),
    )
    .await;
    response(reader, id).await
}
