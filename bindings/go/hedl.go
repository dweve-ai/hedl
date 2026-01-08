// Package hedl provides Go bindings for HEDL (Hierarchical Entity Data Language).
//
// HEDL is a token-efficient data format optimized for LLM context windows,
// providing 5-10x compression compared to JSON.
//
// Example:
//
//	doc, err := hedl.Parse(`
//	%VERSION: 1.0
//	%STRUCT: User: [id, name]
//	---
//	users: @User
//	  | alice, Alice Smith
//	`, true)
//	if err != nil {
//	    log.Fatal(err)
//	}
//	defer doc.Close()
//
//	fmt.Println(doc.Version())
//	json, _ := doc.ToJSON(false)
//	fmt.Println(json)
//
// # Thread Safety Warning
//
// These bindings are NOT thread-safe. Document and Diagnostics objects must
// not be accessed concurrently from multiple goroutines. The underlying FFI
// library does not perform any internal locking. If you need concurrent
// access to HEDL documents, you must:
//
//  1. Use separate Document instances per goroutine, OR
//  2. Implement your own synchronization (sync.Mutex, sync.RWMutex) around all
//     Document and Diagnostics method calls
//
// Concurrent access without proper synchronization may result in:
//   - Memory corruption
//   - Use-after-free errors
//   - Segmentation faults
//   - Undefined behavior
//
// Example of safe concurrent usage:
//
//	var mu sync.Mutex
//	doc, _ := hedl.Parse(content, true)
//	defer doc.Close()
//
//	go func() {
//	    mu.Lock()
//	    defer mu.Unlock()
//	    json, _ := doc.ToJSON(false)
//	    fmt.Println(json)
//	}()
//
// # Resource Limits
//
// The HEDL_MAX_OUTPUT_SIZE environment variable controls the maximum size of
// output from conversion operations (ToJSON, ToYAML, ToXML, etc.).
//
// Default: 100 MB (conservative, may be too restrictive for many use cases)
// Recommended for data processing: 500 MB - 1 GB
// For large datasets: 1 GB - 5 GB
//
// Set before loading the package:
//
//	// In your shell (before running Go)
//	export HEDL_MAX_OUTPUT_SIZE=1073741824  // 1 GB
//
//	// Or in Go (must be set BEFORE package init)
//	os.Setenv("HEDL_MAX_OUTPUT_SIZE", "1073741824")  // 1 GB
//	import "hedl"
//
// Use Cases:
//   - Small configs: 10-50 MB (default may suffice)
//   - Medium datasets: 100-500 MB (set to 524288000 for 500 MB)
//   - Large datasets: 500 MB - 5 GB (set to 1073741824+ for 1 GB+)
//   - No practical limit: set to a very high value like 10737418240 (10 GB)
//
// When the limit is exceeded, operations will return HedlError with code
// ErrAlloc and a message suggesting to increase HEDL_MAX_OUTPUT_SIZE.
package hedl

/*
#cgo LDFLAGS: -lhedl_ffi
#cgo darwin LDFLAGS: -L${SRCDIR}/../../target/release
#cgo linux LDFLAGS: -L${SRCDIR}/../../target/release

#include <stdlib.h>
#include <stdint.h>

// Error codes
#define HEDL_OK                0
#define HEDL_ERR_NULL_PTR     -1
#define HEDL_ERR_INVALID_UTF8 -2
#define HEDL_ERR_PARSE        -3
#define HEDL_ERR_CANONICALIZE -4
#define HEDL_ERR_JSON         -5
#define HEDL_ERR_ALLOC        -6
#define HEDL_ERR_YAML         -7
#define HEDL_ERR_XML          -8
#define HEDL_ERR_CSV          -9
#define HEDL_ERR_PARQUET      -10
#define HEDL_ERR_LINT         -11
#define HEDL_ERR_NEO4J        -12

// Opaque types
typedef struct HedlDocument HedlDocument;
typedef struct HedlDiagnostics HedlDiagnostics;

// Error handling
extern const char* hedl_get_last_error(void);

// Memory management
extern void hedl_free_string(char* s);
extern void hedl_free_document(HedlDocument* doc);
extern void hedl_free_diagnostics(HedlDiagnostics* diag);
extern void hedl_free_bytes(uint8_t* data, size_t len);

// Parsing
extern int hedl_parse(const char* input, int input_len, int strict, HedlDocument** out_doc);
extern int hedl_validate(const char* input, int input_len, int strict);

// Document info
extern int hedl_get_version(const HedlDocument* doc, int* major, int* minor);
extern int hedl_schema_count(const HedlDocument* doc);
extern int hedl_alias_count(const HedlDocument* doc);
extern int hedl_root_item_count(const HedlDocument* doc);

// Canonicalization
extern int hedl_canonicalize(const HedlDocument* doc, char** out_str);

// JSON
extern int hedl_to_json(const HedlDocument* doc, int include_metadata, char** out_str);
extern int hedl_from_json(const char* json, int json_len, HedlDocument** out_doc);

// YAML
extern int hedl_to_yaml(const HedlDocument* doc, int include_metadata, char** out_str);
extern int hedl_from_yaml(const char* yaml, int yaml_len, HedlDocument** out_doc);

// XML
extern int hedl_to_xml(const HedlDocument* doc, char** out_str);
extern int hedl_from_xml(const char* xml, int xml_len, HedlDocument** out_doc);

// CSV
extern int hedl_to_csv(const HedlDocument* doc, char** out_str);

// Parquet
extern int hedl_to_parquet(const HedlDocument* doc, uint8_t** out_data, size_t* out_len);
extern int hedl_from_parquet(const uint8_t* data, size_t len, HedlDocument** out_doc);

// Neo4j
extern int hedl_to_neo4j_cypher(const HedlDocument* doc, int use_merge, char** out_str);

// Linting
extern int hedl_lint(const HedlDocument* doc, HedlDiagnostics** out_diag);
extern int hedl_diagnostics_count(const HedlDiagnostics* diag);
extern int hedl_diagnostics_get(const HedlDiagnostics* diag, int index, char** out_str);
extern int hedl_diagnostics_severity(const HedlDiagnostics* diag, int index);
*/
import "C"
import (
	"errors"
	"fmt"
	"os"
	"runtime"
	"strconv"
	"unsafe"
)

// Resource limits
// Default is 100MB, which may be too restrictive for many real-world scenarios.
// Recommended: 500MB-1GB for data processing, higher for large datasets.
// Set HEDL_MAX_OUTPUT_SIZE environment variable before importing to customize.
var maxOutputSize int64

func init() {
	// Default: 100MB
	maxOutputSize = 104857600
	if env := os.Getenv("HEDL_MAX_OUTPUT_SIZE"); env != "" {
		if size, err := strconv.ParseInt(env, 10, 64); err == nil {
			maxOutputSize = size
		}
	}
}

// Error codes
const (
	ErrNullPtr     = -1
	ErrInvalidUTF8 = -2
	ErrParse       = -3
	ErrCanonicalize = -4
	ErrJSON        = -5
	ErrAlloc       = -6
	ErrYAML        = -7
	ErrXML         = -8
	ErrCSV         = -9
	ErrParquet     = -10
	ErrLint        = -11
	ErrNeo4j       = -12
)

// Severity levels for diagnostics
const (
	SeverityHint    = 0
	SeverityWarning = 1
	SeverityError   = 2
)

// HedlError represents an error from HEDL operations.
type HedlError struct {
	Message string
	Code    int
}

func (e *HedlError) Error() string {
	return e.Message
}

func newError(code C.int) error {
	errStr := C.hedl_get_last_error()
	var msg string
	if errStr != nil {
		msg = C.GoString(errStr)
	} else {
		msg = fmt.Sprintf("HEDL error code %d", code)
	}
	return &HedlError{Message: msg, Code: int(code)}
}

func checkOutputSize(data []byte) error {
	size := int64(len(data))
	if size > maxOutputSize {
		actualMB := float64(size) / 1048576.0
		limitMB := float64(maxOutputSize) / 1048576.0
		return &HedlError{
			Message: fmt.Sprintf("Output size (%.2fMB) exceeds limit (%.2fMB). Set HEDL_MAX_OUTPUT_SIZE to increase.", actualMB, limitMB),
			Code:    ErrAlloc,
		}
	}
	return nil
}

func checkStringOutputSize(s string) error {
	return checkOutputSize([]byte(s))
}

// Document represents a parsed HEDL document.
type Document struct {
	ptr *C.HedlDocument
}

// Diagnostics represents lint diagnostics.
type Diagnostics struct {
	ptr *C.HedlDiagnostics
}

// Diagnostic represents a single lint diagnostic.
type Diagnostic struct {
	Message  string
	Severity int
}

// Parse parses HEDL content into a Document.
//
// If strict is true, reference validation is enabled.
// The returned Document must be closed with Close() when done.
func Parse(content string, strict bool) (*Document, error) {
	cContent := C.CString(content)
	defer C.free(unsafe.Pointer(cContent))

	strictInt := 0
	if strict {
		strictInt = 1
	}

	var docPtr *C.HedlDocument
	result := C.hedl_parse(cContent, C.int(len(content)), C.int(strictInt), &docPtr)
	if result != 0 {
		return nil, newError(result)
	}

	doc := &Document{ptr: docPtr}
	runtime.SetFinalizer(doc, (*Document).Close)
	return doc, nil
}

// Validate validates HEDL content without creating a document.
func Validate(content string, strict bool) bool {
	cContent := C.CString(content)
	defer C.free(unsafe.Pointer(cContent))

	strictInt := 0
	if strict {
		strictInt = 1
	}

	result := C.hedl_validate(cContent, C.int(len(content)), C.int(strictInt))
	return result == 0
}

// FromJSON parses JSON content into a HEDL Document.
func FromJSON(content string) (*Document, error) {
	cContent := C.CString(content)
	defer C.free(unsafe.Pointer(cContent))

	var docPtr *C.HedlDocument
	result := C.hedl_from_json(cContent, C.int(len(content)), &docPtr)
	if result != 0 {
		return nil, newError(result)
	}

	doc := &Document{ptr: docPtr}
	runtime.SetFinalizer(doc, (*Document).Close)
	return doc, nil
}

// FromYAML parses YAML content into a HEDL Document.
func FromYAML(content string) (*Document, error) {
	cContent := C.CString(content)
	defer C.free(unsafe.Pointer(cContent))

	var docPtr *C.HedlDocument
	result := C.hedl_from_yaml(cContent, C.int(len(content)), &docPtr)
	if result != 0 {
		return nil, newError(result)
	}

	doc := &Document{ptr: docPtr}
	runtime.SetFinalizer(doc, (*Document).Close)
	return doc, nil
}

// FromXML parses XML content into a HEDL Document.
func FromXML(content string) (*Document, error) {
	cContent := C.CString(content)
	defer C.free(unsafe.Pointer(cContent))

	var docPtr *C.HedlDocument
	result := C.hedl_from_xml(cContent, C.int(len(content)), &docPtr)
	if result != 0 {
		return nil, newError(result)
	}

	doc := &Document{ptr: docPtr}
	runtime.SetFinalizer(doc, (*Document).Close)
	return doc, nil
}

// FromParquet parses Parquet content into a HEDL Document.
func FromParquet(data []byte) (*Document, error) {
	if len(data) == 0 {
		return nil, errors.New("empty parquet data")
	}

	var docPtr *C.HedlDocument
	result := C.hedl_from_parquet((*C.uint8_t)(unsafe.Pointer(&data[0])), C.size_t(len(data)), &docPtr)
	if result != 0 {
		return nil, newError(result)
	}

	doc := &Document{ptr: docPtr}
	runtime.SetFinalizer(doc, (*Document).Close)
	return doc, nil
}

// Close frees the document resources.
func (d *Document) Close() {
	if d.ptr != nil {
		C.hedl_free_document(d.ptr)
		d.ptr = nil
	}
}

// Version returns the HEDL version as (major, minor).
func (d *Document) Version() (int, int, error) {
	if d.ptr == nil {
		return 0, 0, errors.New("document closed")
	}

	var major, minor C.int
	result := C.hedl_get_version(d.ptr, &major, &minor)
	if result != 0 {
		return 0, 0, newError(result)
	}
	return int(major), int(minor), nil
}

// SchemaCount returns the number of schema definitions.
func (d *Document) SchemaCount() (int, error) {
	if d.ptr == nil {
		return 0, errors.New("document closed")
	}
	count := C.hedl_schema_count(d.ptr)
	if count < 0 {
		return 0, newError(count)
	}
	return int(count), nil
}

// AliasCount returns the number of alias definitions.
func (d *Document) AliasCount() (int, error) {
	if d.ptr == nil {
		return 0, errors.New("document closed")
	}
	count := C.hedl_alias_count(d.ptr)
	if count < 0 {
		return 0, newError(count)
	}
	return int(count), nil
}

// RootItemCount returns the number of root items.
func (d *Document) RootItemCount() (int, error) {
	if d.ptr == nil {
		return 0, errors.New("document closed")
	}
	count := C.hedl_root_item_count(d.ptr)
	if count < 0 {
		return 0, newError(count)
	}
	return int(count), nil
}

// Canonicalize converts the document to canonical HEDL form.
func (d *Document) Canonicalize() (string, error) {
	if d.ptr == nil {
		return "", errors.New("document closed")
	}

	var outStr *C.char
	result := C.hedl_canonicalize(d.ptr, &outStr)
	if result != 0 {
		return "", newError(result)
	}
	defer C.hedl_free_string(outStr)
	output := C.GoString(outStr)
	if err := checkStringOutputSize(output); err != nil {
		return "", err
	}
	return output, nil
}

// ToJSON converts the document to JSON.
func (d *Document) ToJSON(includeMetadata bool) (string, error) {
	if d.ptr == nil {
		return "", errors.New("document closed")
	}

	metaInt := 0
	if includeMetadata {
		metaInt = 1
	}

	var outStr *C.char
	result := C.hedl_to_json(d.ptr, C.int(metaInt), &outStr)
	if result != 0 {
		return "", newError(result)
	}
	defer C.hedl_free_string(outStr)
	output := C.GoString(outStr)
	if err := checkStringOutputSize(output); err != nil {
		return "", err
	}
	return output, nil
}

// ToYAML converts the document to YAML.
func (d *Document) ToYAML(includeMetadata bool) (string, error) {
	if d.ptr == nil {
		return "", errors.New("document closed")
	}

	metaInt := 0
	if includeMetadata {
		metaInt = 1
	}

	var outStr *C.char
	result := C.hedl_to_yaml(d.ptr, C.int(metaInt), &outStr)
	if result != 0 {
		return "", newError(result)
	}
	defer C.hedl_free_string(outStr)
	output := C.GoString(outStr)
	if err := checkStringOutputSize(output); err != nil {
		return "", err
	}
	return output, nil
}

// ToXML converts the document to XML.
func (d *Document) ToXML() (string, error) {
	if d.ptr == nil {
		return "", errors.New("document closed")
	}

	var outStr *C.char
	result := C.hedl_to_xml(d.ptr, &outStr)
	if result != 0 {
		return "", newError(result)
	}
	defer C.hedl_free_string(outStr)
	output := C.GoString(outStr)
	if err := checkStringOutputSize(output); err != nil {
		return "", err
	}
	return output, nil
}

// ToCSV converts the document to CSV.
func (d *Document) ToCSV() (string, error) {
	if d.ptr == nil {
		return "", errors.New("document closed")
	}

	var outStr *C.char
	result := C.hedl_to_csv(d.ptr, &outStr)
	if result != 0 {
		return "", newError(result)
	}
	defer C.hedl_free_string(outStr)
	output := C.GoString(outStr)
	if err := checkStringOutputSize(output); err != nil {
		return "", err
	}
	return output, nil
}

// ToParquet converts the document to Parquet format.
func (d *Document) ToParquet() ([]byte, error) {
	if d.ptr == nil {
		return nil, errors.New("document closed")
	}

	var dataPtr *C.uint8_t
	var dataLen C.size_t
	result := C.hedl_to_parquet(d.ptr, &dataPtr, &dataLen)
	if result != 0 {
		return nil, newError(result)
	}
	defer C.hedl_free_bytes(dataPtr, dataLen)

	// Copy the data before freeing
	data := C.GoBytes(unsafe.Pointer(dataPtr), C.int(dataLen))
	if err := checkOutputSize(data); err != nil {
		return nil, err
	}
	return data, nil
}

// ToCypher converts the document to Neo4j Cypher queries.
func (d *Document) ToCypher(useMerge bool) (string, error) {
	if d.ptr == nil {
		return "", errors.New("document closed")
	}

	mergeInt := 0
	if useMerge {
		mergeInt = 1
	}

	var outStr *C.char
	result := C.hedl_to_neo4j_cypher(d.ptr, C.int(mergeInt), &outStr)
	if result != 0 {
		return "", newError(result)
	}
	defer C.hedl_free_string(outStr)
	output := C.GoString(outStr)
	if err := checkStringOutputSize(output); err != nil {
		return "", err
	}
	return output, nil
}

// Lint runs linting on the document.
func (d *Document) Lint() (*Diagnostics, error) {
	if d.ptr == nil {
		return nil, errors.New("document closed")
	}

	var diagPtr *C.HedlDiagnostics
	result := C.hedl_lint(d.ptr, &diagPtr)
	if result != 0 {
		return nil, newError(result)
	}

	diag := &Diagnostics{ptr: diagPtr}
	runtime.SetFinalizer(diag, (*Diagnostics).Close)
	return diag, nil
}

// Close frees the diagnostics resources.
func (d *Diagnostics) Close() {
	if d.ptr != nil {
		C.hedl_free_diagnostics(d.ptr)
		d.ptr = nil
	}
}

// Count returns the number of diagnostics.
func (d *Diagnostics) Count() int {
	if d.ptr == nil {
		return 0
	}
	count := C.hedl_diagnostics_count(d.ptr)
	if count < 0 {
		return 0
	}
	return int(count)
}

// Get returns the diagnostic at the given index.
func (d *Diagnostics) Get(index int) (*Diagnostic, error) {
	if d.ptr == nil {
		return nil, errors.New("diagnostics closed")
	}

	var msgStr *C.char
	result := C.hedl_diagnostics_get(d.ptr, C.int(index), &msgStr)
	if result != 0 {
		return nil, newError(result)
	}
	defer C.hedl_free_string(msgStr)

	severity := C.hedl_diagnostics_severity(d.ptr, C.int(index))
	return &Diagnostic{
		Message:  C.GoString(msgStr),
		Severity: int(severity),
	}, nil
}

// All returns all diagnostics.
func (d *Diagnostics) All() ([]*Diagnostic, error) {
	count := d.Count()
	result := make([]*Diagnostic, 0, count)
	for i := 0; i < count; i++ {
		diag, err := d.Get(i)
		if err != nil {
			return nil, err
		}
		result = append(result, diag)
	}
	return result, nil
}

// Errors returns all error messages.
func (d *Diagnostics) Errors() ([]string, error) {
	all, err := d.All()
	if err != nil {
		return nil, err
	}
	var result []string
	for _, diag := range all {
		if diag.Severity == SeverityError {
			result = append(result, diag.Message)
		}
	}
	return result, nil
}

// Warnings returns all warning messages.
func (d *Diagnostics) Warnings() ([]string, error) {
	all, err := d.All()
	if err != nil {
		return nil, err
	}
	var result []string
	for _, diag := range all {
		if diag.Severity == SeverityWarning {
			result = append(result, diag.Message)
		}
	}
	return result, nil
}
