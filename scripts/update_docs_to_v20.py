#!/usr/bin/env python3
"""
Update documentation files to use v2.0 HEDL syntax.
Converts old directive syntax to compact v2.0 format.
"""

import re
import sys
from pathlib import Path


def update_hedl_examples(content: str) -> str:
    """Update HEDL examples in documentation to v2.0 syntax."""

    # Pattern for %VERSION: X.Y -> %V:2.0
    content = re.sub(
        r'%VERSION:\s*(\d+\.\d+)',
        r'%V:2.0',
        content
    )

    # Pattern for %STRUCT: Name: [cols] -> %S:Name:[cols]
    # Handle both with and without spaces
    content = re.sub(
        r'%STRUCT:\s*(\w+):\s*\[([^\]]+)\]',
        r'%S:\1:[\2]',
        content
    )

    # Pattern for %NEST: Parent > Child -> %N:Parent>Child
    content = re.sub(
        r'%NEST:\s*(\w+)\s*>\s*(\w+)',
        r'%N:\1>\2',
        content
    )

    # Pattern for %COUNT: -> %C:
    content = re.sub(
        r'%COUNT:\s*',
        r'%C:',
        content
    )

    # Pattern for %ALIAS: -> %A:
    content = re.sub(
        r'%ALIAS:\s*',
        r'%A:',
        content
    )

    # Update section headers to remove space before colon (users: @User -> users:@User)
    # Only in HEDL code blocks or examples
    # This is tricky - we need context. Skip for now to be safe.

    return content


def process_file(filepath: Path, dry_run: bool = False) -> bool:
    """Process a single markdown file. Returns True if changes were made."""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            original = f.read()
    except UnicodeDecodeError:
        print(f"Skipping {filepath} (encoding issue)")
        return False

    updated = update_hedl_examples(original)

    if updated != original:
        if not dry_run:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(updated)
        return True
    return False


def main():
    dry_run = '--dry-run' in sys.argv

    if len(sys.argv) < 2 or sys.argv[1] == '--dry-run':
        # Process all markdown files in project
        root = Path('.')
        files = list(root.glob('**/*.md'))
    else:
        files = [Path(p) for p in sys.argv[1:] if p != '--dry-run']

    changed = 0
    for filepath in files:
        if filepath.exists() and filepath.is_file():
            if process_file(filepath, dry_run):
                changed += 1
                print(f"{'Would update' if dry_run else 'Updated'}: {filepath}")

    print(f"\n{'Would update' if dry_run else 'Updated'} {changed} files")


if __name__ == '__main__':
    main()
