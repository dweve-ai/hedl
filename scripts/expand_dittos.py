#!/usr/bin/env python3
"""
Expand ditto (^) values in HEDL files and convert to v2.0 format.
This script reads a HEDL file and expands all ^ ditto references to explicit values.
"""

import re
import sys
from pathlib import Path
from typing import Optional


def parse_csv_value(s: str, start: int) -> tuple[str, int]:
    """Parse a single CSV-like value, handling quotes and nested brackets."""
    if start >= len(s):
        return "", start

    c = s[start]

    # Quoted string
    if c == '"':
        end = start + 1
        result = '"'
        while end < len(s):
            if s[end] == '"':
                result += '"'
                end += 1
                break
            if s[end] == '\\' and end + 1 < len(s):
                result += s[end:end+2]
                end += 2
            else:
                result += s[end]
                end += 1
        return result, end

    # Nested list/object with brackets
    if c in '[{':
        close = ']' if c == '[' else '}'
        depth = 1
        end = start + 1
        while end < len(s) and depth > 0:
            if s[end] == c:
                depth += 1
            elif s[end] == close:
                depth -= 1
            elif s[end] == '"':
                # Skip quoted string inside
                end += 1
                while end < len(s):
                    if s[end] == '"':
                        break
                    if s[end] == '\\' and end + 1 < len(s):
                        end += 1
                    end += 1
            end += 1
        return s[start:end], end

    # Regular value (until comma)
    end = start
    while end < len(s) and s[end] != ',':
        end += 1
    return s[start:end], end


def split_row_values(row: str) -> list[str]:
    """Split a row into values, properly handling quotes and brackets."""
    values = []
    pos = 0
    row = row.strip()

    while pos < len(row):
        val, pos = parse_csv_value(row, pos)
        values.append(val)
        # Skip comma
        if pos < len(row) and row[pos] == ',':
            pos += 1
        elif pos < len(row):
            # No comma found but more content - shouldn't happen in well-formed data
            pos += 1

    return values


def join_row_values(values: list[str]) -> str:
    """Join values back into a row string."""
    return ','.join(values)


def expand_dittos_in_file(filepath: Path) -> str:
    """Read a HEDL file and expand all ditto values."""
    with open(filepath, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    output_lines = []

    # Track previous values per struct type (key = struct name)
    prev_values: dict[str, list[str]] = {}

    # Pattern for struct declarations
    struct_pattern = re.compile(r'^%(?:STRUCT|S):\s*(\w+):\s*\[([^\]]+)\]')
    # Pattern for section headers (e.g., "users: @User" or "users:@User")
    section_pattern = re.compile(r'^(\w+):\s*@(\w+)\s*$')
    # Pattern for data rows (e.g., " | id,val1,val2")
    row_pattern = re.compile(r'^(\s*)\|\s*(.+)$')
    # Pattern for count-prefixed inline rows (e.g., "@Comment#2:|row1|row2")
    count_row_pattern = re.compile(r'^(\s*)@(\w+)#(\d+):\|(.+)$')

    structs: dict[str, list[str]] = {}
    current_section_struct: Optional[str] = None
    in_body = False

    # First pass: collect struct definitions
    for line in lines:
        line_stripped = line.rstrip('\n')
        m = struct_pattern.match(line_stripped)
        if m:
            name, cols = m.groups()
            columns = [c.strip() for c in cols.split(',')]
            structs[name] = columns

    # Second pass: process line by line
    header_lines = []
    body_lines = []
    found_separator = False

    for line in lines:
        line = line.rstrip('\n')

        if line.strip() == '---':
            found_separator = True
            continue

        if not found_separator:
            header_lines.append(line)
        else:
            body_lines.append(line)

    # Convert headers to v2.0 format
    new_header = []
    has_version = False
    has_null = False
    has_quote = False

    for line in header_lines:
        # Skip empty lines at start
        if not line.strip() and not new_header:
            continue

        # Handle version
        if line.startswith('%VERSION:') or line.startswith('%V:'):
            if not has_version:
                new_header.append('%V:2.0')
                has_version = True
                if not has_null:
                    new_header.append('%NULL:~')
                    has_null = True
                if not has_quote:
                    new_header.append('%QUOTE:"')
                    has_quote = True
            continue

        # Skip old NULL/QUOTE
        if line.startswith('%NULL:') or line.startswith('%QUOTE:'):
            continue

        # Convert %STRUCT to %S
        if line.startswith('%STRUCT:'):
            m = re.match(r'^%STRUCT:\s*(\w+):\s*\[([^\]]+)\]', line)
            if m:
                name, cols = m.groups()
                new_header.append(f'%S:{name}:[{cols}]')
            else:
                new_header.append(line)
            continue

        # Keep %S as-is
        if line.startswith('%S:'):
            new_header.append(line)
            continue

        # Convert %NEST to %N
        if line.startswith('%NEST:'):
            m = re.match(r'^%NEST:\s*(\w+)\s*>\s*(\w+)', line)
            if m:
                parent, child = m.groups()
                new_header.append(f'%N:{parent}>{child}')
            else:
                new_header.append(line)
            continue

        # Keep %N as-is
        if line.startswith('%N:'):
            new_header.append(line)
            continue

        # Convert %COUNT to %C
        if line.startswith('%COUNT:'):
            new_header.append('%C:' + line[7:])
            continue

        # Keep %C as-is
        if line.startswith('%C:'):
            new_header.append(line)
            continue

        # Convert %ALIAS to %A
        if line.startswith('%ALIAS:'):
            new_header.append('%A:' + line[7:])
            continue

        # Keep %A as-is
        if line.startswith('%A:'):
            new_header.append(line)
            continue

        new_header.append(line)

    # Add version headers if missing
    if not has_version:
        new_header.insert(0, '%QUOTE:"')
        new_header.insert(0, '%NULL:~')
        new_header.insert(0, '%V:2.0')

    output_lines.extend(new_header)
    output_lines.append('---')

    # Process body lines, expanding dittos
    for line in body_lines:
        # Check for section header (e.g., "users: @User")
        sm = section_pattern.match(line)
        if sm:
            name, struct_name = sm.groups()
            current_section_struct = struct_name
            # Don't clear prev_values - dittos can span sections
            # Output in v2.0 format (no space after colon)
            output_lines.append(f'{name}:@{struct_name}')
            continue

        # Check for count-prefixed inline rows (e.g., @Comment#2:|...)
        cm = count_row_pattern.match(line)
        if cm:
            indent, struct_name, count, row_content = cm.groups()

            # Split by | to get multiple rows
            row_parts = row_content.split('|')
            expanded_rows = []

            for rp in row_parts:
                if not rp.strip():
                    continue

                values = split_row_values(rp.strip())
                prev = prev_values.get(struct_name, [])

                # Expand dittos
                expanded = []
                for i, v in enumerate(values):
                    if v == '^' and i < len(prev):
                        expanded.append(prev[i])
                    else:
                        expanded.append(v)

                # Update previous values for this struct type
                prev_values[struct_name] = expanded
                expanded_rows.append(join_row_values(expanded))

            output_lines.append(f'{indent}@{struct_name}#{count}:|' + '|'.join(expanded_rows))
            continue

        # Check for regular data row
        rm = row_pattern.match(line)
        if rm:
            indent, row_content = rm.groups()

            # Try to determine struct type from row ID pattern
            # Common patterns: user1, dept-001, proj-001, ms-001, task-0001, cmt-0001, emp-001
            struct_type = current_section_struct

            # Also check ID prefix to infer struct type
            values = split_row_values(row_content)
            if values:
                row_id = values[0]
                if row_id.startswith('dept-'):
                    struct_type = 'Department'
                elif row_id.startswith('proj-'):
                    struct_type = 'Project'
                elif row_id.startswith('ms-'):
                    struct_type = 'Milestone'
                elif row_id.startswith('task-'):
                    struct_type = 'Task'
                elif row_id.startswith('cmt-'):
                    struct_type = 'Comment'
                elif row_id.startswith('emp-'):
                    struct_type = 'Employee'
                elif row_id.startswith('org-'):
                    struct_type = 'Organization'
                elif row_id.startswith('avg-'):
                    struct_type = 'SeasonAverages'
                elif row_id.startswith('g-'):
                    struct_type = 'Game'
                elif row_id.startswith('pt-'):
                    struct_type = 'Player'
                # Add more patterns as needed

            if struct_type:
                prev = prev_values.get(struct_type, [])

                # Expand dittos
                expanded = []
                for i, v in enumerate(values):
                    if v == '^' and i < len(prev):
                        expanded.append(prev[i])
                    else:
                        expanded.append(v)

                # Update previous values
                prev_values[struct_type] = expanded

                output_lines.append(f'{indent}|' + join_row_values(expanded))
            else:
                output_lines.append(line)
            continue

        # Pass through other lines unchanged
        output_lines.append(line)

    return '\n'.join(output_lines) + '\n'


def main():
    if len(sys.argv) < 2:
        print("Usage: expand_dittos.py <input_file> [output_file]")
        sys.exit(1)

    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2]) if len(sys.argv) > 2 else input_path

    result = expand_dittos_in_file(input_path)

    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(result)

    print(f"Expanded dittos in {input_path} -> {output_path}")


if __name__ == '__main__':
    main()
