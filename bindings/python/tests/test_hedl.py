"""Tests for HEDL Python bindings."""
import os
import sys
import unittest

# Set library path before importing hedl
lib_path = os.path.join(os.path.dirname(__file__), '..', '..', '..', 'target', 'release')
os.environ['HEDL_LIB_PATH'] = lib_path

from hedl import parse, validate, from_json, from_yaml, from_xml, Document, Diagnostics, HedlError
from fixtures import fixtures

# Use shared fixtures from common/fixtures directory
SAMPLE_HEDL = fixtures.basic_hedl
SAMPLE_JSON = fixtures.basic_json
SAMPLE_YAML = fixtures.basic_yaml
SAMPLE_XML = fixtures.basic_xml


class TestParse(unittest.TestCase):
    """Test parsing functionality."""

    def test_parse_valid(self):
        """Test parsing valid HEDL content."""
        doc = parse(SAMPLE_HEDL)
        self.assertIsInstance(doc, Document)
        doc.close()

    def test_parse_context_manager(self):
        """Test using Document as context manager."""
        with parse(SAMPLE_HEDL) as doc:
            self.assertIsInstance(doc, Document)

    def test_parse_invalid(self):
        """Test parsing invalid content raises error."""
        with self.assertRaises(HedlError):
            parse(fixtures.error_invalid_syntax)

    def test_validate_valid(self):
        """Test validating valid content."""
        self.assertTrue(validate(SAMPLE_HEDL))

    def test_validate_invalid(self):
        """Test validating invalid content."""
        self.assertFalse(validate(fixtures.error_invalid_syntax))


class TestDocumentProperties(unittest.TestCase):
    """Test Document properties."""

    def setUp(self):
        self.doc = parse(SAMPLE_HEDL)

    def tearDown(self):
        self.doc.close()

    def test_version(self):
        """Test version property."""
        version = self.doc.version
        self.assertEqual(version, (1, 0))

    def test_schema_count(self):
        """Test schema_count property."""
        self.assertEqual(self.doc.schema_count, 1)

    def test_root_item_count(self):
        """Test root_item_count property."""
        self.assertGreaterEqual(self.doc.root_item_count, 1)


class TestConversions(unittest.TestCase):
    """Test format conversion methods."""

    def setUp(self):
        self.doc = parse(SAMPLE_HEDL)

    def tearDown(self):
        self.doc.close()

    def test_canonicalize(self):
        """Test canonicalization."""
        canonical = self.doc.canonicalize()
        self.assertIsInstance(canonical, str)
        self.assertIn('%VERSION', canonical)

    def test_to_json(self):
        """Test JSON conversion."""
        json_str = self.doc.to_json()
        self.assertIsInstance(json_str, str)
        self.assertIn('users', json_str)

    def test_to_json_with_metadata(self):
        """Test JSON conversion with metadata."""
        json_str = self.doc.to_json(include_metadata=True)
        self.assertIsInstance(json_str, str)

    def test_to_yaml(self):
        """Test YAML conversion."""
        yaml_str = self.doc.to_yaml()
        self.assertIsInstance(yaml_str, str)

    def test_to_xml(self):
        """Test XML conversion."""
        xml_str = self.doc.to_xml()
        self.assertIsInstance(xml_str, str)
        self.assertIn('<', xml_str)

    def test_to_csv(self):
        """Test CSV conversion."""
        csv_str = self.doc.to_csv()
        self.assertIsInstance(csv_str, str)

    def test_to_cypher(self):
        """Test Neo4j Cypher conversion."""
        cypher = self.doc.to_cypher()
        self.assertIsInstance(cypher, str)

    def test_to_parquet(self):
        """Test Parquet conversion."""
        data = self.doc.to_parquet()
        self.assertIsInstance(data, bytes)
        self.assertGreater(len(data), 0)


class TestFromFormats(unittest.TestCase):
    """Test parsing from other formats."""

    def test_from_json(self):
        """Test parsing from JSON."""
        doc = from_json(SAMPLE_JSON)
        self.assertIsInstance(doc, Document)
        hedl = doc.canonicalize()
        self.assertIn('users', hedl)
        doc.close()

    def test_from_yaml(self):
        """Test parsing from YAML."""
        doc = from_yaml(SAMPLE_YAML)
        self.assertIsInstance(doc, Document)
        doc.close()

    def test_from_xml(self):
        """Test parsing from XML."""
        doc = from_xml(SAMPLE_XML)
        self.assertIsInstance(doc, Document)
        doc.close()


class TestLinting(unittest.TestCase):
    """Test linting functionality."""

    def test_lint(self):
        """Test linting a document."""
        with parse(SAMPLE_HEDL) as doc:
            diag = doc.lint()
            self.assertIsInstance(diag, Diagnostics)
            # Count should be >= 0
            self.assertGreaterEqual(len(diag), 0)
            diag.close()

    def test_diagnostics_iteration(self):
        """Test iterating over diagnostics."""
        with parse(SAMPLE_HEDL) as doc:
            with doc.lint() as diag:
                items = list(diag)
                self.assertIsInstance(items, list)


class TestMemoryManagement(unittest.TestCase):
    """Test proper memory management."""

    def test_document_close(self):
        """Test closing document releases resources."""
        doc = parse(SAMPLE_HEDL)
        doc.close()
        # Should not raise on double close
        doc.close()

    def test_context_manager_closes(self):
        """Test context manager properly closes."""
        with parse(SAMPLE_HEDL) as doc:
            _ = doc.to_json()
        # doc should be closed after exiting context


if __name__ == '__main__':
    unittest.main()
