#!/usr/bin/env python3
"""Validate beaterOS contract examples against the JSON Schemas.

Dependency-free by design: it implements the small subset of JSON Schema
(draft 2020-12) that these contracts use, so it runs in CI and on any
reviewer's machine with a stock Python 3 and no network. If the optional
`jsonschema` package is installed, it is used as an independent cross-check
and any disagreement is reported as a failure.

Usage:
    python3 contracts/validate.py            # validate everything
    python3 contracts/validate.py --quiet    # only print the summary/failures

Exit code is 0 iff every valid example passes AND every fixture under
examples/invalid/ is correctly rejected.

Naming contract:
  examples/<name>.example.json          -> must PASS schemas/<name>.schema.json
  examples/invalid/<name>.<reason>.json -> must FAIL schemas/<name>.schema.json
"""
from __future__ import annotations

import json
import os
import re
import sys
from typing import Any

HERE = os.path.dirname(os.path.abspath(__file__))
SCHEMA_DIR = os.path.join(HERE, "schemas")
EXAMPLE_DIR = os.path.join(HERE, "examples")

_DATE_TIME_RE = re.compile(
    r"^\d{4}-\d{2}-\d{2}[Tt]\d{2}:\d{2}:\d{2}(\.\d+)?([Zz]|[+-]\d{2}:\d{2})$"
)


class SchemaError(Exception):
    pass


def load_schemas() -> dict[str, Any]:
    schemas: dict[str, Any] = {}
    for fn in os.listdir(SCHEMA_DIR):
        if fn.endswith(".schema.json"):
            with open(os.path.join(SCHEMA_DIR, fn)) as f:
                schemas[fn] = json.load(f)
    return schemas


class Validator:
    """Minimal JSON Schema validator over a registry of schema documents."""

    def __init__(self, schemas: dict[str, Any]):
        self.schemas = schemas

    def resolve_ref(self, ref: str, current_doc: Any) -> Any:
        # Supports "file.schema.json#/$defs/Name" and "#/$defs/Name".
        file_part, _, pointer = ref.partition("#")
        doc = self.schemas[file_part] if file_part else current_doc
        node = doc
        for token in pointer.split("/"):
            if token == "":
                continue
            token = token.replace("~1", "/").replace("~0", "~")
            if not isinstance(node, dict) or token not in node:
                raise SchemaError(f"unresolvable $ref: {ref}")
            node = node[token]
        return doc, node

    def validate(self, instance: Any, schema: Any, doc: Any, path: str,
                 errors: list[str]) -> None:
        if "$ref" in schema:
            ref_doc, target = self.resolve_ref(schema["$ref"], doc)
            self.validate(instance, target, ref_doc, path, errors)
            return

        if "oneOf" in schema:
            matches = 0
            for sub in schema["oneOf"]:
                sub_errors: list[str] = []
                self.validate(instance, sub, doc, path, sub_errors)
                if not sub_errors:
                    matches += 1
            if matches != 1:
                errors.append(f"{path}: matched {matches} of oneOf (expected exactly 1)")
            return
        if "anyOf" in schema:
            for sub in schema["anyOf"]:
                sub_errors: list[str] = []
                self.validate(instance, sub, doc, path, sub_errors)
                if not sub_errors:
                    return
            errors.append(f"{path}: matched none of anyOf")
            return

        if "const" in schema and instance != schema["const"]:
            errors.append(f"{path}: {instance!r} != const {schema['const']!r}")

        if "enum" in schema and instance not in schema["enum"]:
            errors.append(f"{path}: {instance!r} not in enum {schema['enum']}")

        t = schema.get("type")
        if t is not None and not self._type_ok(instance, t):
            errors.append(f"{path}: expected type {t}, got {_json_type(instance)}")
            return  # further keyword checks assume the type held

        if _json_type(instance) == "string":
            if schema.get("format") == "date-time" and not _DATE_TIME_RE.match(instance):
                errors.append(f"{path}: {instance!r} is not an RFC 3339 date-time")
            if "pattern" in schema and not re.search(schema["pattern"], instance):
                errors.append(f"{path}: {instance!r} does not match pattern {schema['pattern']}")

        if isinstance(instance, bool):
            number = None  # bool is not a number here
        elif isinstance(instance, int):
            number = instance
        elif isinstance(instance, float):
            number = instance
        else:
            number = None
        if number is not None:
            if "minimum" in schema and number < schema["minimum"]:
                errors.append(f"{path}: {number} < minimum {schema['minimum']}")
            if "maximum" in schema and number > schema["maximum"]:
                errors.append(f"{path}: {number} > maximum {schema['maximum']}")

        if isinstance(instance, list):
            if "minItems" in schema and len(instance) < schema["minItems"]:
                errors.append(f"{path}: fewer than minItems {schema['minItems']}")
            if schema.get("uniqueItems"):
                seen = [json.dumps(x, sort_keys=True) for x in instance]
                if len(set(seen)) != len(seen):
                    errors.append(f"{path}: items are not unique")
            item_schema = schema.get("items")
            if item_schema is not None:
                for i, item in enumerate(instance):
                    self.validate(item, item_schema, doc, f"{path}[{i}]", errors)

        if isinstance(instance, dict):
            props = schema.get("properties", {})
            for req in schema.get("required", []):
                if req not in instance:
                    errors.append(f"{path}: missing required property '{req}'")
            addl = schema.get("additionalProperties", True)
            for key, val in instance.items():
                if key in props:
                    self.validate(val, props[key], doc, f"{path}.{key}", errors)
                elif addl is False:
                    errors.append(f"{path}: additional property '{key}' not allowed")
                elif isinstance(addl, dict):
                    self.validate(val, addl, doc, f"{path}.{key}", errors)

    @staticmethod
    def _type_ok(instance: Any, t: Any) -> bool:
        types = t if isinstance(t, list) else [t]
        return any(_matches_type(instance, one) for one in types)


def _json_type(instance: Any) -> str:
    if instance is None:
        return "null"
    if isinstance(instance, bool):
        return "boolean"
    if isinstance(instance, int):
        return "integer"
    if isinstance(instance, float):
        return "number"
    if isinstance(instance, str):
        return "string"
    if isinstance(instance, list):
        return "array"
    if isinstance(instance, dict):
        return "object"
    return "unknown"


def _matches_type(instance: Any, t: str) -> bool:
    if t == "null":
        return instance is None
    if t == "boolean":
        return isinstance(instance, bool)
    if t == "integer":
        return isinstance(instance, int) and not isinstance(instance, bool)
    if t == "number":
        return isinstance(instance, (int, float)) and not isinstance(instance, bool)
    if t == "string":
        return isinstance(instance, str)
    if t == "array":
        return isinstance(instance, list)
    if t == "object":
        return isinstance(instance, dict)
    return False


def schema_for_example(filename: str) -> str:
    """Map an example filename to its schema filename."""
    base = os.path.basename(filename)
    if base.endswith(".example.json"):
        return base[: -len(".example.json")] + ".schema.json"
    # invalid fixtures: <contract>.<reason>.json -> <contract>.schema.json
    contract = base.split(".")[0]
    return contract + ".schema.json"


def _jsonschema_check(instance, schema_doc, schemas):
    """Optional independent cross-check using the jsonschema package."""
    try:
        import jsonschema
        from jsonschema.validators import validator_for
    except Exception:
        return None  # not installed; skip cross-check
    store = {name: doc for name, doc in schemas.items()}
    try:
        from referencing import Registry, Resource

        resources = [(name, Resource.from_contents(doc)) for name, doc in store.items()]
        registry = Registry().with_resources(resources)
        cls = validator_for(schema_doc)
        validator = cls(schema_doc, registry=registry)
    except Exception:
        # Fall back to legacy RefResolver for older jsonschema versions.
        resolver = jsonschema.RefResolver(base_uri="", referrer=schema_doc, store=store)
        cls = validator_for(schema_doc)
        validator = cls(schema_doc, resolver=resolver)
    return len(list(validator.iter_errors(instance))) == 0


def main() -> int:
    quiet = "--quiet" in sys.argv
    schemas = load_schemas()
    validator = Validator(schemas)

    valid_examples: list[str] = []
    invalid_examples: list[str] = []
    if os.path.isdir(EXAMPLE_DIR):
        for fn in sorted(os.listdir(EXAMPLE_DIR)):
            if fn.endswith(".json"):
                valid_examples.append(os.path.join(EXAMPLE_DIR, fn))
        inv_dir = os.path.join(EXAMPLE_DIR, "invalid")
        if os.path.isdir(inv_dir):
            for fn in sorted(os.listdir(inv_dir)):
                if fn.endswith(".json"):
                    invalid_examples.append(os.path.join(inv_dir, fn))

    failures: list[str] = []
    passes = 0

    for path in valid_examples:
        schema_name = schema_for_example(path)
        if schema_name not in schemas:
            failures.append(f"{os.path.basename(path)}: no schema '{schema_name}'")
            continue
        schema_doc = schemas[schema_name]
        with open(path) as f:
            instance = json.load(f)
        errors: list[str] = []
        validator.validate(instance, schema_doc, schema_doc, "$", errors)
        cross = _jsonschema_check(instance, schema_doc, schemas)
        if errors:
            failures.append(f"VALID example rejected: {os.path.basename(path)}\n    - " + "\n    - ".join(errors))
        elif cross is False:
            failures.append(f"cross-check disagreement (jsonschema rejected): {os.path.basename(path)}")
        else:
            passes += 1
            if not quiet:
                tag = " [+jsonschema]" if cross is True else ""
                print(f"  ok    {os.path.basename(path)} -> {schema_name}{tag}")

    for path in invalid_examples:
        schema_name = schema_for_example(path)
        if schema_name not in schemas:
            failures.append(f"{os.path.basename(path)}: no schema '{schema_name}'")
            continue
        schema_doc = schemas[schema_name]
        with open(path) as f:
            instance = json.load(f)
        errors = []
        validator.validate(instance, schema_doc, schema_doc, "$", errors)
        if not errors:
            failures.append(f"INVALID fixture accepted (should have failed): {os.path.basename(path)}")
        else:
            passes += 1
            if not quiet:
                print(f"  ok    {os.path.basename(path)} correctly rejected ({len(errors)} error(s))")

    total = len(valid_examples) + len(invalid_examples)
    print(f"\n{passes}/{total} checks passed "
          f"({len(valid_examples)} valid, {len(invalid_examples)} negative).")
    if failures:
        print(f"\n{len(failures)} FAILURE(S):")
        for fail in failures:
            print(f"- {fail}")
        return 1
    print("All contract examples validate. ✔")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
