"""Read a local Asterisk queue_log sample and emit only allowlisted counts."""
import argparse
import json

MAX_LINE = 16384
MAX_LINES = 100000
QUEUE_IDS = {"billing-voice", "accounts-voice", "technical-voice", "customer-voice"}


def counts(stream):
    result = {q: 0 for q in sorted(QUEUE_IDS)}
    for _ in range(MAX_LINES):
        line = stream.readline(MAX_LINE + 1)
        if not line:
            return result
        if len(line) > MAX_LINE:
            raise ValueError("Input line exceeds the sample import limit")
        fields = line.rstrip(b"\r\n").split(b"|", 5)
        if len(fields) < 5 or not fields[0].isdigit():
            raise ValueError("Expected queue_log header fields")
        if fields[4] != b"ENTERQUEUE":
            continue
        queue = fields[2].decode("ascii", errors="replace")
        if queue in result:
            result[queue] += 1
    if stream.read(1):
        raise ValueError("Input exceeds the sample import limit")
    return result


def main():
    args = argparse.ArgumentParser(description=__doc__)
    args.add_argument("sample_path")
    path = args.parse_args().sample_path
    try:
        with open(path, "rb") as source:
            aggregate = counts(source)
    except (OSError, ValueError):
        args.exit(2, "Could not read a bounded queue_log sample. No rows were emitted.\n")
    print(json.dumps({"sampleVoiceQueueEntryEvents": aggregate}, sort_keys=True))


if __name__ == "__main__":
    main()
