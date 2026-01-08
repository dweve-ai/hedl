# Tests for HEDL Ruby bindings

require 'minitest/autorun'
require 'json'

# Set library path before requiring hedl
ENV['HEDL_LIB_PATH'] = File.expand_path('../../../target/release/libhedl_ffi.so', __dir__)

require_relative '../lib/hedl'
require_relative 'fixtures'

class TestHedl < Minitest::Test
  # Use shared fixtures from common/fixtures directory
  SAMPLE_HEDL = $hedl_fixtures.basic_hedl
  SAMPLE_JSON = $hedl_fixtures.basic_json
  SAMPLE_YAML = $hedl_fixtures.basic_yaml
  SAMPLE_XML = $hedl_fixtures.basic_xml

  def test_parse
    doc = Hedl.parse(SAMPLE_HEDL)
    assert_kind_of Hedl::Document, doc
    doc.close
  end

  def test_parse_invalid
    assert_raises(Hedl::Error) do
      Hedl.parse($hedl_fixtures.error_invalid_syntax)
    end
  end

  def test_validate
    assert Hedl.validate(SAMPLE_HEDL)
  end

  def test_validate_invalid
    refute Hedl.validate($hedl_fixtures.error_invalid_syntax)
  end

  def test_version
    doc = Hedl.parse(SAMPLE_HEDL)
    major, minor = doc.version
    assert_equal 1, major
    assert_equal 0, minor
    doc.close
  end

  def test_schema_count
    doc = Hedl.parse(SAMPLE_HEDL)
    assert_equal 1, doc.schema_count
    doc.close
  end

  def test_canonicalize
    doc = Hedl.parse(SAMPLE_HEDL)
    canonical = doc.canonicalize
    refute_empty canonical
    assert_includes canonical, '%VERSION'
    doc.close
  end

  def test_to_json
    doc = Hedl.parse(SAMPLE_HEDL)
    json = doc.to_json
    refute_empty json
    # Should be valid JSON
    JSON.parse(json)
    doc.close
  end

  def test_to_yaml
    doc = Hedl.parse(SAMPLE_HEDL)
    yaml = doc.to_yaml
    refute_empty yaml
    doc.close
  end

  def test_to_xml
    doc = Hedl.parse(SAMPLE_HEDL)
    xml = doc.to_xml
    refute_empty xml
    assert_includes xml, '<'
    doc.close
  end

  def test_to_csv
    doc = Hedl.parse(SAMPLE_HEDL)
    csv = doc.to_csv
    refute_empty csv
    doc.close
  end

  def test_to_cypher
    doc = Hedl.parse(SAMPLE_HEDL)
    cypher = doc.to_cypher
    refute_empty cypher
    doc.close
  end

  def test_to_parquet
    doc = Hedl.parse(SAMPLE_HEDL)
    data = doc.to_parquet
    refute_empty data
    doc.close
  end

  def test_from_json
    doc = Hedl.from_json(SAMPLE_JSON)
    assert_kind_of Hedl::Document, doc
    canonical = doc.canonicalize
    refute_empty canonical
    doc.close
  end

  def test_from_yaml
    doc = Hedl.from_yaml(SAMPLE_YAML)
    assert_kind_of Hedl::Document, doc
    doc.close
  end

  def test_from_xml
    doc = Hedl.from_xml(SAMPLE_XML)
    assert_kind_of Hedl::Document, doc
    doc.close
  end

  def test_lint
    doc = Hedl.parse(SAMPLE_HEDL)
    diag = doc.lint
    assert_kind_of Hedl::Diagnostics, diag
    assert diag.count >= 0
    diag.close
    doc.close
  end

  def test_double_close
    doc = Hedl.parse(SAMPLE_HEDL)
    doc.close
    doc.close # Should not raise
  end
end
