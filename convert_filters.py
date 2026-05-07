#!/usr/bin/env python3
"""Convert TOML filter files to a single HEDL filter document."""

import os
import tomllib
from pathlib import Path
from typing import Any


def escape_csv_field(value: Any) -> str:
    """Escape a value for HEDL CSV matrix format."""
    if value is None or value == "":
        return ""
    s = str(value)
    # If contains comma, newline, or quote, wrap in quotes and escape
    if ',' in s or '\n' in s or '"' in s:
        s = s.replace('\\', '\\\\')  # escape backslashes first
        s = s.replace('\n', '\\n')     # escape newlines
        s = s.replace('\t', '\\t')     # escape tabs
        s = s.replace('"', '""')        # escape quotes (CSV style)
        return f'"{s}"'
    return s


def main():
    filters_dir = Path("crates/hedl-filter/filters")
    toml_files = sorted(filters_dir.glob("*.toml"))

    filters = []
    line_rules = []
    replace_rules = []
    match_rules = []
    tests = []

    line_rule_idx = 0
    replace_rule_idx = 0
    match_rule_idx = 0
    test_idx = 0

    for toml_file in toml_files:
        with open(toml_file, "rb") as f:
            data = tomllib.load(f)

        # Each file should have [filters.<name>]
        filter_section = data.get("filters", {})
        for name, fdef in filter_section.items():
            filters.append({
                "name": name,
                "description": fdef.get("description", ""),
                "match_command": fdef.get("match_command", ""),
                "strip_ansi": "true" if fdef.get("strip_ansi", False) else "false",
                "truncate_lines_at": str(fdef.get("truncate_lines_at", "")),
                "head_lines": str(fdef.get("head_lines", "")),
                "tail_lines": str(fdef.get("tail_lines", "")),
                "max_lines": str(fdef.get("max_lines", "")),
                "on_empty": fdef.get("on_empty", ""),
                "filter_stderr": "true" if fdef.get("filter_stderr", False) else "false",
            })

            for pattern in fdef.get("strip_lines_matching", []):
                line_rule_idx += 1
                line_rules.append({
                    "id": f"lr{line_rule_idx}",
                    "filter_name": name,
                    "action": "strip",
                    "pattern": pattern,
                })

            for pattern in fdef.get("keep_lines_matching", []):
                line_rule_idx += 1
                line_rules.append({
                    "id": f"lr{line_rule_idx}",
                    "filter_name": name,
                    "action": "keep",
                    "pattern": pattern,
                })

            for r in fdef.get("replace", []):
                replace_rule_idx += 1
                replace_rules.append({
                    "id": f"rr{replace_rule_idx}",
                    "filter_name": name,
                    "pattern": r["pattern"],
                    "replacement": r["replacement"],
                })

            for m in fdef.get("match_output", []):
                match_rule_idx += 1
                match_rules.append({
                    "id": f"mr{match_rule_idx}",
                    "filter_name": name,
                    "pattern": m["pattern"],
                    "message": m["message"],
                    "unless": m.get("unless", ""),
                })

        # Tests are in [[tests.<name>]]
        test_section = data.get("tests", {})
        for filter_name, test_list in test_section.items():
            for t in test_list:
                test_idx += 1
                tests.append({
                    "id": f"t{test_idx}",
                    "filter_name": filter_name,
                    "test_name": t["name"],
                    "input": t["input"],
                    "expected": t["expected"],
                })

    # Generate HEDL output
    lines = []
    lines.append("%V:2.0")
    lines.append("%NULL:~")
    lines.append('%QUOTE:"')
    lines.append("%S:Filter:[name, description, match_command, strip_ansi, truncate_lines_at, head_lines, tail_lines, max_lines, on_empty, filter_stderr]")
    lines.append("%S:LineRule:[id, filter_name, action, pattern]")
    lines.append("%S:ReplaceRule:[id, filter_name, pattern, replacement]")
    lines.append("%S:MatchRule:[id, filter_name, pattern, message, unless]")
    lines.append("%S:FilterTest:[id, filter_name, test_name, input, expected]")
    lines.append("---")
    lines.append("")

    lines.append("filters: @Filter")
    for f in filters:
        row = " |" + ",".join([
            escape_csv_field(f["name"]),
            escape_csv_field(f["description"]),
            escape_csv_field(f["match_command"]),
            escape_csv_field(f["strip_ansi"]),
            escape_csv_field(f["truncate_lines_at"]),
            escape_csv_field(f["head_lines"]),
            escape_csv_field(f["tail_lines"]),
            escape_csv_field(f["max_lines"]),
            escape_csv_field(f["on_empty"]),
            escape_csv_field(f["filter_stderr"]),
        ])
        lines.append(row)

    if line_rules:
        lines.append("")
        lines.append("line_rules: @LineRule")
        for r in line_rules:
            row = " |" + ",".join([
                escape_csv_field(r["id"]),
                escape_csv_field(r["filter_name"]),
                escape_csv_field(r["action"]),
                escape_csv_field(r["pattern"]),
            ])
            lines.append(row)

    if replace_rules:
        lines.append("")
        lines.append("replace_rules: @ReplaceRule")
        for r in replace_rules:
            row = " |" + ",".join([
                escape_csv_field(r["id"]),
                escape_csv_field(r["filter_name"]),
                escape_csv_field(r["pattern"]),
                escape_csv_field(r["replacement"]),
            ])
            lines.append(row)

    if match_rules:
        lines.append("")
        lines.append("match_rules: @MatchRule")
        for r in match_rules:
            # Only include unless if it's not empty to avoid trailing comma issues
            fields = [
                escape_csv_field(r["id"]),
                escape_csv_field(r["filter_name"]),
                escape_csv_field(r["pattern"]),
                escape_csv_field(r["message"]),
            ]
            if r["unless"]:
                fields.append(escape_csv_field(r["unless"]))
            row = " |" + ",".join(fields)
            lines.append(row)

    if tests:
        lines.append("")
        lines.append("tests: @FilterTest")
        for t in tests:
            row = " |" + ",".join([
                escape_csv_field(t["id"]),
                escape_csv_field(t["filter_name"]),
                escape_csv_field(t["test_name"]),
                escape_csv_field(t["input"]),
                escape_csv_field(t["expected"]),
            ])
            lines.append(row)

    output = "\n".join(lines) + "\n"

    output_path = filters_dir / "filters.hedl"
    with open(output_path, "w") as f:
        f.write(output)

    print(f"Converted {len(toml_files)} TOML files to HEDL")
    print(f"  Filters: {len(filters)}")
    print(f"  Line rules: {len(line_rules)}")
    print(f"  Replace rules: {len(replace_rules)}")
    print(f"  Match rules: {len(match_rules)}")
    print(f"  Tests: {len(tests)}")
    print(f"Wrote: {output_path}")


if __name__ == "__main__":
    main()
