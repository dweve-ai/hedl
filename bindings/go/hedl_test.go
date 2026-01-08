package hedl

import (
	"testing"
)

// Use shared fixtures from common/fixtures directory
var (
	sampleHEDL string
	sampleJSON string
	sampleYAML string
	sampleXML  string
)

func init() {
	fixtures := GetGlobalFixtures()

	var err error
	sampleHEDL, err = fixtures.BasicHEDL()
	if err != nil {
		panic("failed to load sampleHEDL: " + err.Error())
	}

	sampleJSON, err = fixtures.BasicJSON()
	if err != nil {
		panic("failed to load sampleJSON: " + err.Error())
	}

	sampleYAML, err = fixtures.BasicYAML()
	if err != nil {
		panic("failed to load sampleYAML: " + err.Error())
	}

	sampleXML, err = fixtures.BasicXML()
	if err != nil {
		panic("failed to load sampleXML: " + err.Error())
	}
}

func TestParse(t *testing.T) {
	doc, err := Parse(sampleHEDL, true)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	defer doc.Close()

	if doc == nil {
		t.Fatal("Expected non-nil document")
	}
}

func TestParseInvalid(t *testing.T) {
	fixtures := GetGlobalFixtures()
	invalidSyntax, err := fixtures.ErrorInvalidSyntax()
	if err != nil {
		t.Fatalf("Failed to load fixture: %v", err)
	}

	_, err = Parse(invalidSyntax, true)
	if err == nil {
		t.Fatal("Expected error for invalid content")
	}
}

func TestValidate(t *testing.T) {
	if !Validate(sampleHEDL, true) {
		t.Fatal("Expected valid content to pass validation")
	}
}

func TestValidateInvalid(t *testing.T) {
	fixtures := GetGlobalFixtures()
	invalidSyntax, err := fixtures.ErrorInvalidSyntax()
	if err != nil {
		t.Fatalf("Failed to load fixture: %v", err)
	}

	if Validate(invalidSyntax, true) {
		t.Fatal("Expected invalid content to fail validation")
	}
}

func TestDocumentVersion(t *testing.T) {
	doc, err := Parse(sampleHEDL, true)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	defer doc.Close()

	major, minor, err := doc.Version()
	if err != nil {
		t.Fatalf("Version failed: %v", err)
	}
	if major != 1 || minor != 0 {
		t.Fatalf("Expected version 1.0, got %d.%d", major, minor)
	}
}

func TestDocumentSchemaCount(t *testing.T) {
	doc, err := Parse(sampleHEDL, true)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	defer doc.Close()

	count, err := doc.SchemaCount()
	if err != nil {
		t.Fatalf("SchemaCount failed: %v", err)
	}
	if count != 1 {
		t.Fatalf("Expected 1 schema, got %d", count)
	}
}

func TestCanonicalize(t *testing.T) {
	doc, err := Parse(sampleHEDL, true)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	defer doc.Close()

	canonical, err := doc.Canonicalize()
	if err != nil {
		t.Fatalf("Canonicalize failed: %v", err)
	}
	if len(canonical) == 0 {
		t.Fatal("Expected non-empty canonical output")
	}
}

func TestToJSON(t *testing.T) {
	doc, err := Parse(sampleHEDL, true)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	defer doc.Close()

	json, err := doc.ToJSON(false)
	if err != nil {
		t.Fatalf("ToJSON failed: %v", err)
	}
	if len(json) == 0 {
		t.Fatal("Expected non-empty JSON output")
	}
}

func TestToYAML(t *testing.T) {
	doc, err := Parse(sampleHEDL, true)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	defer doc.Close()

	yaml, err := doc.ToYAML(false)
	if err != nil {
		t.Fatalf("ToYAML failed: %v", err)
	}
	if len(yaml) == 0 {
		t.Fatal("Expected non-empty YAML output")
	}
}

func TestToXML(t *testing.T) {
	doc, err := Parse(sampleHEDL, true)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	defer doc.Close()

	xml, err := doc.ToXML()
	if err != nil {
		t.Fatalf("ToXML failed: %v", err)
	}
	if len(xml) == 0 {
		t.Fatal("Expected non-empty XML output")
	}
}

func TestToCSV(t *testing.T) {
	doc, err := Parse(sampleHEDL, true)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	defer doc.Close()

	csv, err := doc.ToCSV()
	if err != nil {
		t.Fatalf("ToCSV failed: %v", err)
	}
	if len(csv) == 0 {
		t.Fatal("Expected non-empty CSV output")
	}
}

func TestToParquet(t *testing.T) {
	doc, err := Parse(sampleHEDL, true)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	defer doc.Close()

	data, err := doc.ToParquet()
	if err != nil {
		t.Fatalf("ToParquet failed: %v", err)
	}
	if len(data) == 0 {
		t.Fatal("Expected non-empty Parquet output")
	}
}

func TestToCypher(t *testing.T) {
	doc, err := Parse(sampleHEDL, true)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	defer doc.Close()

	cypher, err := doc.ToCypher(true)
	if err != nil {
		t.Fatalf("ToCypher failed: %v", err)
	}
	if len(cypher) == 0 {
		t.Fatal("Expected non-empty Cypher output")
	}
}

func TestFromJSON(t *testing.T) {
	doc, err := FromJSON(sampleJSON)
	if err != nil {
		t.Fatalf("FromJSON failed: %v", err)
	}
	defer doc.Close()

	canonical, err := doc.Canonicalize()
	if err != nil {
		t.Fatalf("Canonicalize failed: %v", err)
	}
	if len(canonical) == 0 {
		t.Fatal("Expected non-empty canonical output")
	}
}

func TestFromYAML(t *testing.T) {
	doc, err := FromYAML(sampleYAML)
	if err != nil {
		t.Fatalf("FromYAML failed: %v", err)
	}
	defer doc.Close()
}

func TestFromXML(t *testing.T) {
	doc, err := FromXML(sampleXML)
	if err != nil {
		t.Fatalf("FromXML failed: %v", err)
	}
	defer doc.Close()
}

func TestLint(t *testing.T) {
	doc, err := Parse(sampleHEDL, true)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	defer doc.Close()

	diag, err := doc.Lint()
	if err != nil {
		t.Fatalf("Lint failed: %v", err)
	}
	defer diag.Close()

	// Count should be >= 0
	if diag.Count() < 0 {
		t.Fatal("Expected count >= 0")
	}
}

func TestDoubleClose(t *testing.T) {
	doc, err := Parse(sampleHEDL, true)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	doc.Close()
	doc.Close() // Should not panic
}
