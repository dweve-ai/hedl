// Package hedl provides test fixtures for HEDL Go bindings.
//
// This package provides access to shared test fixtures stored in the
// bindings/common/fixtures directory, eliminating test data duplication
// across language bindings.
package hedl

import (
	"encoding/json"
	"os"
	"path/filepath"
	"runtime"
)

// FixtureManifest represents the structure of the manifest.json file.
type FixtureManifest struct {
	Fixtures map[string]FixtureEntry `json:"fixtures"`
	Errors   map[string]ErrorEntry   `json:"errors"`
}

// FixtureEntry represents a fixture entry in the manifest.
type FixtureEntry struct {
	Description string            `json:"description"`
	Files       map[string]string `json:"files"`
}

// ErrorEntry represents an error fixture entry in the manifest.
type ErrorEntry struct {
	Description   string `json:"description"`
	File          string `json:"file"`
	ExpectedError bool   `json:"expected_error"`
}

// Fixtures provides access to common HEDL test fixtures.
//
// All fixtures are loaded from bindings/common/fixtures directory
// to ensure consistency across language bindings.
type Fixtures struct {
	fixturesDir string
	manifest    FixtureManifest
}

// NewFixtures creates a new Fixtures instance and loads the manifest.
func NewFixtures() (*Fixtures, error) {
	// Get the directory of this source file
	_, filename, _, ok := runtime.Caller(0)
	if !ok {
		return nil, &HedlError{Message: "failed to get caller information"}
	}

	// Path to common fixtures directory
	// From bindings/go/fixtures.go -> bindings/common/fixtures
	fixturesDir := filepath.Join(filepath.Dir(filename), "..", "common", "fixtures")

	// Load manifest
	manifestPath := filepath.Join(fixturesDir, "manifest.json")
	manifestData, err := os.ReadFile(manifestPath)
	if err != nil {
		return nil, &HedlError{Message: "failed to read manifest: " + err.Error()}
	}

	var manifest FixtureManifest
	if err := json.Unmarshal(manifestData, &manifest); err != nil {
		return nil, &HedlError{Message: "failed to parse manifest: " + err.Error()}
	}

	return &Fixtures{
		fixturesDir: fixturesDir,
		manifest:    manifest,
	}, nil
}

// readFile reads a fixture file and returns its contents.
func (f *Fixtures) readFile(filename string) (string, error) {
	filepath := filepath.Join(f.fixturesDir, filename)
	data, err := os.ReadFile(filepath)
	if err != nil {
		return "", &HedlError{Message: "failed to read fixture: " + err.Error()}
	}
	return string(data), nil
}

// Basic fixtures

// BasicHEDL returns the basic HEDL sample document.
func (f *Fixtures) BasicHEDL() (string, error) {
	return f.readFile(f.manifest.Fixtures["basic"].Files["hedl"])
}

// BasicJSON returns the basic JSON sample document.
func (f *Fixtures) BasicJSON() (string, error) {
	return f.readFile(f.manifest.Fixtures["basic"].Files["json"])
}

// BasicYAML returns the basic YAML sample document.
func (f *Fixtures) BasicYAML() (string, error) {
	return f.readFile(f.manifest.Fixtures["basic"].Files["yaml"])
}

// BasicXML returns the basic XML sample document.
func (f *Fixtures) BasicXML() (string, error) {
	return f.readFile(f.manifest.Fixtures["basic"].Files["xml"])
}

// Type-specific fixtures

// ScalarsHEDL returns a HEDL document with various scalar types.
func (f *Fixtures) ScalarsHEDL() (string, error) {
	return f.readFile(f.manifest.Fixtures["scalars"].Files["hedl"])
}

// NestedHEDL returns a HEDL document with nested structures.
func (f *Fixtures) NestedHEDL() (string, error) {
	return f.readFile(f.manifest.Fixtures["nested"].Files["hedl"])
}

// ListsHEDL returns a HEDL document with lists and arrays.
func (f *Fixtures) ListsHEDL() (string, error) {
	return f.readFile(f.manifest.Fixtures["lists"].Files["hedl"])
}

// Performance fixtures

// LargeHEDL returns a large HEDL document for performance testing.
func (f *Fixtures) LargeHEDL() (string, error) {
	return f.readFile(f.manifest.Fixtures["large"].Files["hedl"])
}

// Error fixtures

// ErrorInvalidSyntax returns invalid HEDL syntax for error testing.
func (f *Fixtures) ErrorInvalidSyntax() (string, error) {
	return f.readFile(f.manifest.Errors["invalid_syntax"].File)
}

// ErrorMalformed returns a malformed HEDL document for error testing.
func (f *Fixtures) ErrorMalformed() (string, error) {
	return f.readFile(f.manifest.Errors["malformed"].File)
}

// Utility methods

// GetFixture returns a specific fixture by category and format.
//
// Example:
//
//	fixtures, _ := NewFixtures()
//	hedl, _ := fixtures.GetFixture("basic", "hedl")
func (f *Fixtures) GetFixture(category, format string) (string, error) {
	if entry, ok := f.manifest.Fixtures[category]; ok {
		if filename, ok := entry.Files[format]; ok {
			return f.readFile(filename)
		}
	}

	return "", &HedlError{Message: "fixture not found: category=" + category + ", format=" + format}
}

// GetErrorFixture returns an error fixture by type.
func (f *Fixtures) GetErrorFixture(errorType string) (string, error) {
	if entry, ok := f.manifest.Errors[errorType]; ok {
		return f.readFile(entry.File)
	}

	return "", &HedlError{Message: "error fixture not found: " + errorType}
}

// Global fixtures instance for convenient access
var globalFixtures *Fixtures

func init() {
	var err error
	globalFixtures, err = NewFixtures()
	if err != nil {
		panic("failed to initialize fixtures: " + err.Error())
	}
}

// GetGlobalFixtures returns the global fixtures instance.
func GetGlobalFixtures() *Fixtures {
	return globalFixtures
}
