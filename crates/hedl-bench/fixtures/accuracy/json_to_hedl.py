#!/usr/bin/env python3
"""
JSON to HEDL v1.2 Converter

HEDL Format Rules:
- %V:1.2 - version
- %NULL:~ - null marker
- %QUOTE:" - quote character
- %S:Type:[field1,field2,...] - schema (fields match JSON EXACTLY)
- %C:Type.total=N - total count
- %C:Type.field:val1=N,val2=M - value distributions
- %N:Parent>Child - nesting relationships
- --- - separator between header and data
- |val1,val2,... - data row
- ~ - null value
- ^ - same as previous value in same column
- @ref - reference to another entity
- @Type#N: - nested block with N children
- "quoted" - values containing commas must be quoted

When nested:
- Child's foreign key field is IMPLICIT (not in schema or data)
- The relationship is expressed through physical nesting
"""

import json
import sys
from collections import Counter, defaultdict
from typing import Any, Optional
from pathlib import Path


class HEDLConverter:
    def __init__(self, config: dict):
        """
        config = {
            "root_entities": ["products", "customers", ...],  # Top-level entity collections
            "nesting": {
                "customers": {
                    "children": {
                        "addresses": {"fk": "customer", "json_key": "addresses"},
                        "orders": {
                            "fk": "customer",
                            "json_key": "orders",
                            "children": {
                                "order_items": {"fk": "order", "json_key": "order_items"},
                                "shipments": {"fk": "order", "json_key": "shipments"}
                            }
                        }
                    }
                },
                ...
            },
            "flat_entities": ["addresses"],  # Entities that appear at root level (not nested)
            "distributions": {
                "Order": ["status", "payment"],
                "Customer": ["tier"],
                ...
            },
            "type_names": {
                "orders": "Order",
                "customers": "Customer",
                ...
            },
            "references": {
                # field_name: True if it's a reference to another entity
                "product": True,
                "customer": True,
                ...
            }
        }
        """
        self.config = config
        self.data = {}
        self.schemas = {}
        self.counts = {}
        self.distributions = defaultdict(lambda: defaultdict(Counter))
        self.nesting_relationships = []

    def load_json(self, filepath: str):
        with open(filepath) as f:
            self.data = json.load(f)
        return self

    def get_type_name(self, json_key: str) -> str:
        """Convert JSON key to HEDL type name."""
        if "type_names" in self.config and json_key in self.config["type_names"]:
            return self.config["type_names"][json_key]
        # Default: capitalize and singularize
        name = json_key.rstrip('s')
        return name[0].upper() + name[1:]

    def get_fields_for_entity(self, json_key: str, exclude_fk: Optional[str] = None) -> list[str]:
        """Get field names for an entity, optionally excluding a foreign key."""
        if json_key not in self.data or not self.data[json_key]:
            return []

        sample = self.data[json_key][0]
        fields = list(sample.keys())

        if exclude_fk and exclude_fk in fields:
            fields.remove(exclude_fk)

        return fields

    def format_value(self, value: Any, prev_value: Any = None, is_reference: bool = False) -> str:
        """Format a value for HEDL output."""
        if value is None:
            return "~"

        # Check for "same as previous" optimization
        if prev_value is not None and value == prev_value:
            return "^"

        if isinstance(value, bool):
            return "true" if value else "false"

        if isinstance(value, (list, tuple)):
            # Format as array: [val1,val2,...]
            inner = ",".join(str(v) for v in value)
            return f"[{inner}]"

        if isinstance(value, dict):
            # Shouldn't happen in flat data, but handle it
            return json.dumps(value)

        str_val = str(value)

        # Add reference prefix if this is a reference field
        if is_reference and str_val and not str_val.startswith("@"):
            return f"@{str_val}"

        # Quote if contains comma
        if "," in str_val:
            return f'"{str_val}"'

        return str_val

    def compute_distributions(self):
        """Compute count distributions for configured fields."""
        dist_config = self.config.get("distributions", {})

        for type_name, fields in dist_config.items():
            # Find the JSON key for this type
            json_key = None
            for key, name in self.config.get("type_names", {}).items():
                if name == type_name:
                    json_key = key
                    break

            if not json_key or json_key not in self.data:
                continue

            for field in fields:
                counter = Counter()
                for item in self.data[json_key]:
                    if field in item:
                        val = item[field]
                        if val is not None:
                            counter[val] += 1
                        else:
                            counter["~"] += 1

                if counter:
                    self.distributions[type_name][field] = counter

    def build_schema_line(self, type_name: str, fields: list[str]) -> str:
        """Build a %S schema line."""
        return f"%S:{type_name}:[{','.join(fields)}]"

    def build_count_lines(self, type_name: str, total: int) -> list[str]:
        """Build %C count lines for a type."""
        lines = [f"%C:{type_name}.total={total}"]

        if type_name in self.distributions:
            for field, counter in self.distributions[type_name].items():
                parts = [f"{k}={v}" for k, v in counter.items()]
                lines.append(f"%C:{type_name}.{field}:{','.join(parts)}")

        return lines

    def get_children_for_parent(self, parent_key: str, parent_id: str, child_key: str, fk_field: str) -> list[dict]:
        """Get all children of a parent by foreign key."""
        if child_key not in self.data:
            return []
        return [item for item in self.data[child_key] if item.get(fk_field) == parent_id]

    def format_entity_row(self, item: dict, fields: list[str], prev_item: Optional[dict] = None,
                          references: Optional[set] = None) -> str:
        """Format a single entity as a HEDL row."""
        references = references or set()
        values = []

        for i, field in enumerate(fields):
            value = item.get(field)
            prev_value = prev_item.get(field) if prev_item else None
            is_ref = field in references
            values.append(self.format_value(value, prev_value, is_ref))

        return "|" + ",".join(values)

    def format_nested_children(self, children: list[dict], child_type: str, child_fields: list[str],
                               indent: str, references: set, child_config: dict) -> list[str]:
        """Format nested children with potential sub-nesting."""
        lines = []
        n = len(children)

        if n == 0:
            return lines

        # Check if children fit on one line (no sub-children and few children)
        has_sub_children = "children" in child_config

        if not has_sub_children and n <= 3:
            # Inline format: @Type#N:|row1|row2|row3
            rows = []
            prev = None
            for child in children:
                row = self.format_entity_row(child, child_fields, prev, references)
                rows.append(row)
                prev = child
            lines.append(f"{indent}@{child_type}#{n}:{rows[0]}{''.join(rows[1:])}")
        else:
            # Block format
            lines.append(f"{indent}@{child_type}#{n}:")
            prev = None
            for child in children:
                row = self.format_entity_row(child, child_fields, prev, references)
                lines.append(f"{indent}{row}")
                prev = child

                # Handle sub-children
                if has_sub_children:
                    for sub_key, sub_config in child_config["children"].items():
                        sub_fk = sub_config["fk"]
                        sub_json_key = sub_config.get("json_key", sub_key)
                        sub_type = self.get_type_name(sub_json_key)
                        sub_fields = self.get_fields_for_entity(sub_json_key, exclude_fk=sub_fk)
                        sub_children = self.get_children_for_parent(
                            child_type.lower() + "s", child["id"], sub_json_key, sub_fk
                        )

                        sub_lines = self.format_nested_children(
                            sub_children, sub_type, sub_fields,
                            indent + " ", references, sub_config
                        )
                        lines.extend(sub_lines)

        return lines

    def convert(self) -> str:
        """Convert loaded JSON to HEDL format."""
        self.compute_distributions()

        lines = []

        # Header
        lines.append("%V:1.2")
        lines.append("%NULL:~")
        lines.append('%QUOTE:"')

        # Build schemas and counts
        schema_lines = []
        count_lines = []
        nesting_lines = []

        references = set(self.config.get("references", {}).keys())
        nesting_config = self.config.get("nesting", {})
        flat_entities = set(self.config.get("flat_entities", []))

        # Process all entity types
        processed = set()

        def process_entity_tree(json_key: str, parent_type: Optional[str] = None,
                                fk_field: Optional[str] = None, config: dict = None):
            if json_key in processed:
                return
            processed.add(json_key)

            type_name = self.get_type_name(json_key)

            # Determine if FK should be excluded (nested under parent)
            exclude_fk = fk_field if parent_type else None
            fields = self.get_fields_for_entity(json_key, exclude_fk=exclude_fk)

            if fields:
                schema_lines.append(self.build_schema_line(type_name, fields))

            # Add nesting relationship
            if parent_type:
                nesting_lines.append(f"%N:{parent_type}>{type_name}")

            # Process children
            if config and "children" in config:
                for child_key, child_config in config["children"].items():
                    child_json_key = child_config.get("json_key", child_key)
                    process_entity_tree(child_json_key, type_name, child_config["fk"], child_config)

        # Process nested entities first
        for root_key, root_config in nesting_config.items():
            process_entity_tree(root_key, config=root_config)

        # Process flat/standalone entities (always process these)
        for json_key in flat_entities:
            if json_key not in processed and json_key in self.data:
                type_name = self.get_type_name(json_key)
                fields = self.get_fields_for_entity(json_key)
                if fields:
                    schema_lines.append(self.build_schema_line(type_name, fields))
                processed.add(json_key)

        # Process any remaining root entities
        for json_key in self.config.get("root_entities", []):
            if json_key not in processed:
                process_entity_tree(json_key)

        # Add schemas
        lines.extend(schema_lines)

        # Add counts
        for json_key in self.data:
            type_name = self.get_type_name(json_key)
            total = len(self.data[json_key])
            lines.extend(self.build_count_lines(type_name, total))

        # Add nesting relationships
        lines.extend(nesting_lines)

        # Separator
        lines.append("---")

        # Data section
        def write_nested_data(json_key: str, parent_id: Optional[str] = None,
                              fk_field: Optional[str] = None, indent: str = "",
                              config: dict = None):
            """Write data for an entity and its nested children."""
            data_lines = []
            type_name = self.get_type_name(json_key)

            # Get items (filtered by parent if nested)
            if parent_id and fk_field:
                items = self.get_children_for_parent("", parent_id, json_key, fk_field)
            else:
                items = self.data.get(json_key, [])

            if not items:
                return data_lines

            exclude_fk = fk_field if parent_id else None
            fields = self.get_fields_for_entity(json_key, exclude_fk=exclude_fk)

            # Write section header if root level
            if not parent_id:
                data_lines.append(f"{json_key}:@{type_name}")

            prev = None
            for item in items:
                row = self.format_entity_row(item, fields, prev, references)
                data_lines.append(f"{indent}{row}")
                prev = item

                # Write nested children
                if config and "children" in config:
                    for child_key, child_config in config["children"].items():
                        child_json_key = child_config.get("json_key", child_key)
                        child_fk = child_config["fk"]
                        child_type = self.get_type_name(child_json_key)
                        child_fields = self.get_fields_for_entity(child_json_key, exclude_fk=child_fk)

                        children = self.get_children_for_parent(
                            json_key, item["id"], child_json_key, child_fk
                        )

                        child_lines = self.format_nested_children(
                            children, child_type, child_fields,
                            indent + " ", references, child_config
                        )
                        data_lines.extend(child_lines)

            return data_lines

        # Write nested entities
        for root_key, root_config in nesting_config.items():
            data_lines = write_nested_data(root_key, config=root_config)
            lines.extend(data_lines)
            lines.append("")  # Blank line between sections

        # Write flat entities
        for json_key in flat_entities:
            if json_key in self.data:
                type_name = self.get_type_name(json_key)
                fields = self.get_fields_for_entity(json_key)
                lines.append(f"{json_key}:@{type_name}")

                prev = None
                for item in self.data[json_key]:
                    row = self.format_entity_row(item, fields, prev, references)
                    lines.append(row)
                    prev = item
                lines.append("")

        return "\n".join(lines)


def main():
    if len(sys.argv) < 3:
        print("Usage: python json_to_hedl.py <input.json> <output.hedl> [config.json]")
        print("\nconfig.json defines the conversion rules (nesting, distributions, etc.)")
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = sys.argv[2]
    config_file = sys.argv[3] if len(sys.argv) > 3 else None

    # Load config or use defaults
    if config_file:
        with open(config_file) as f:
            config = json.load(f)
    else:
        # Auto-detect flat structure
        with open(input_file) as f:
            data = json.load(f)

        config = {
            "root_entities": list(data.keys()),
            "nesting": {},
            "flat_entities": list(data.keys()),
            "distributions": {},
            "type_names": {},
            "references": {}
        }

    converter = HEDLConverter(config)
    converter.load_json(input_file)
    result = converter.convert()

    with open(output_file, 'w') as f:
        f.write(result)

    print(f"Converted {input_file} -> {output_file}")


if __name__ == "__main__":
    main()
