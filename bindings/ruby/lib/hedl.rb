# frozen_string_literal: true

# HEDL (Hierarchical Entity Data Language) Ruby Bindings
#
# A token-efficient data format optimized for LLM context windows.
#
# Usage:
#   require 'hedl'
#
#   doc = Hedl.parse('%VERSION: 1.0\n---\nkey: value')
#   puts doc.version  # => [1, 0]
#   puts doc.to_json
#   doc.close
#
# THREAD SAFETY WARNING:
# =====================
# These bindings are NOT thread-safe. Document and Diagnostics objects must
# not be accessed concurrently from multiple threads. The underlying FFI
# library does not perform any internal locking. If you need concurrent
# access to HEDL documents, you must:
#
# 1. Use separate Document instances per thread, OR
# 2. Implement your own synchronization (Mutex, Monitor, etc.) around all
#    Document and Diagnostics method calls
#
# Concurrent access without proper synchronization may result in:
# - Memory corruption
# - Use-after-free errors
# - Segmentation faults
# - Undefined behavior
#
# Ruby's GVL (Global VM Lock) does NOT protect against these issues because
# FFI calls release the GVL during execution.
#
# RESOURCE LIMITS:
# ===============
# The HEDL_MAX_OUTPUT_SIZE environment variable controls the maximum size of
# output from conversion operations (to_json, to_yaml, to_xml, etc.).
#
# Default: 100 MB (conservative, may be too restrictive for many use cases)
# Recommended for data processing: 500 MB - 1 GB
# For large datasets: 1 GB - 5 GB
#
# Set before loading hedl:
#
#   # In your shell (before running Ruby)
#   export HEDL_MAX_OUTPUT_SIZE=1073741824  # 1 GB
#
#   # Or in Ruby (must be set BEFORE require)
#   ENV['HEDL_MAX_OUTPUT_SIZE'] = '1073741824'  # 1 GB
#   require 'hedl'
#
# Use Cases:
# - Small configs: 10-50 MB (default may suffice)
# - Medium datasets: 100-500 MB (set to 524288000 for 500 MB)
# - Large datasets: 500 MB - 5 GB (set to 1073741824+ for 1 GB+)
# - No practical limit: set to a very high value like 10737418240 (10 GB)
#
# When the limit is exceeded, operations will raise Hedl::Error with code
# ErrorCode::ALLOC and a message suggesting to increase HEDL_MAX_OUTPUT_SIZE.

require 'ffi'

module Hedl
  # Resource limits
  # Default is 100MB, which may be too restrictive for many real-world scenarios.
  # Recommended: 500MB-1GB for data processing, higher for large datasets.
  # Set HEDL_MAX_OUTPUT_SIZE environment variable before requiring to customize.
  MAX_OUTPUT_SIZE = ENV.fetch('HEDL_MAX_OUTPUT_SIZE', '104857600').to_i  # 100MB default

  # Error codes
  module ErrorCode
    OK = 0
    NULL_PTR = -1
    INVALID_UTF8 = -2
    PARSE = -3
    CANONICALIZE = -4
    JSON = -5
    ALLOC = -6
    YAML = -7
    XML = -8
    CSV = -9
    PARQUET = -10
    LINT = -11
    NEO4J = -12
  end

  # Severity levels
  module Severity
    HINT = 0
    WARNING = 1
    ERROR = 2
  end

  # FFI bindings to the HEDL library
  module FFIBindings
    extend FFI::Library

    # Find the library
    def self.find_library
      lib_name = case FFI::Platform::OS
                 when 'darwin' then 'libhedl_ffi.dylib'
                 when 'windows' then 'hedl_ffi.dll'
                 else 'libhedl_ffi.so'
                 end

      # Check environment variable
      env_path = ENV['HEDL_LIB_PATH']
      return env_path if env_path && File.exist?(env_path)

      # Check common paths
      paths = [
        File.join(__dir__, '..', lib_name),
        File.join(__dir__, '..', '..', '..', 'target', 'release', lib_name),
        File.join('/usr/local/lib', lib_name),
        File.join('/usr/lib', lib_name)
      ]

      paths.each do |path|
        return path if File.exist?(path)
      end

      # Return name for system search
      lib_name
    end

    ffi_lib find_library

    # Error handling
    attach_function :hedl_get_last_error, [], :string

    # Memory management
    attach_function :hedl_free_string, [:pointer], :void
    attach_function :hedl_free_document, [:pointer], :void
    attach_function :hedl_free_diagnostics, [:pointer], :void
    attach_function :hedl_free_bytes, [:pointer, :size_t], :void

    # Parsing
    attach_function :hedl_parse, [:string, :int, :int, :pointer], :int
    attach_function :hedl_validate, [:string, :int, :int], :int

    # Document info
    attach_function :hedl_get_version, [:pointer, :pointer, :pointer], :int
    attach_function :hedl_schema_count, [:pointer], :int
    attach_function :hedl_alias_count, [:pointer], :int
    attach_function :hedl_root_item_count, [:pointer], :int

    # Canonicalization
    attach_function :hedl_canonicalize, [:pointer, :pointer], :int

    # JSON
    attach_function :hedl_to_json, [:pointer, :int, :pointer], :int
    attach_function :hedl_from_json, [:string, :int, :pointer], :int

    # YAML
    attach_function :hedl_to_yaml, [:pointer, :int, :pointer], :int
    attach_function :hedl_from_yaml, [:string, :int, :pointer], :int

    # XML
    attach_function :hedl_to_xml, [:pointer, :pointer], :int
    attach_function :hedl_from_xml, [:string, :int, :pointer], :int

    # CSV
    attach_function :hedl_to_csv, [:pointer, :pointer], :int

    # Parquet
    attach_function :hedl_to_parquet, [:pointer, :pointer, :pointer], :int
    attach_function :hedl_from_parquet, [:pointer, :size_t, :pointer], :int

    # Neo4j
    attach_function :hedl_to_neo4j_cypher, [:pointer, :int, :pointer], :int

    # Linting
    attach_function :hedl_lint, [:pointer, :pointer], :int
    attach_function :hedl_diagnostics_count, [:pointer], :int
    attach_function :hedl_diagnostics_get, [:pointer, :int, :pointer], :int
    attach_function :hedl_diagnostics_severity, [:pointer, :int], :int
  end

  # Exception raised by HEDL operations
  class Error < StandardError
    attr_reader :code

    def initialize(message, code = ErrorCode::PARSE)
      super(message)
      @code = code
    end

    def self.from_lib(code)
      msg = FFIBindings.hedl_get_last_error || "HEDL error code #{code}"
      new(msg, code)
    end
  end

  # Lint diagnostics container
  class Diagnostics
    include Enumerable

    def initialize(ptr)
      @ptr = ptr
      @closed = false
      ObjectSpace.define_finalizer(self, self.class.release(@ptr))
    end

    def self.release(ptr)
      proc { FFIBindings.hedl_free_diagnostics(ptr) unless ptr.null? }
    end

    def close
      return if @closed

      ObjectSpace.undefine_finalizer(self)
      FFIBindings.hedl_free_diagnostics(@ptr)
      @closed = true
    end

    def count
      return 0 if @closed

      c = FFIBindings.hedl_diagnostics_count(@ptr)
      c.negative? ? 0 : c
    end

    alias length count
    alias size count

    def get(index)
      raise Error.new('Diagnostics already closed', ErrorCode::NULL_PTR) if @closed
      raise IndexError, "index #{index} out of range" if index.negative? || index >= count

      msg_ptr = FFI::MemoryPointer.new(:pointer)
      result = FFIBindings.hedl_diagnostics_get(@ptr, index, msg_ptr)
      raise Error.from_lib(result) unless result == ErrorCode::OK

      str_ptr = msg_ptr.read_pointer
      message = str_ptr.read_string
      FFIBindings.hedl_free_string(str_ptr)

      severity = FFIBindings.hedl_diagnostics_severity(@ptr, index)
      { message: message, severity: severity }
    end

    def each
      return enum_for(:each) unless block_given?

      count.times { |i| yield get(i) }
    end

    def errors
      select { |d| d[:severity] == Severity::ERROR }.map { |d| d[:message] }
    end

    def warnings
      select { |d| d[:severity] == Severity::WARNING }.map { |d| d[:message] }
    end

    def hints
      select { |d| d[:severity] == Severity::HINT }.map { |d| d[:message] }
    end
  end

  # A parsed HEDL document
  class Document
    def initialize(ptr)
      @ptr = ptr
      @closed = false
      ObjectSpace.define_finalizer(self, self.class.release(@ptr))
    end

    def self.release(ptr)
      proc { FFIBindings.hedl_free_document(ptr) unless ptr.null? }
    end

    def close
      return if @closed

      ObjectSpace.undefine_finalizer(self)
      FFIBindings.hedl_free_document(@ptr)
      @closed = true
    end

    def closed?
      @closed
    end

    # Get the HEDL version as [major, minor]
    def version
      check_closed

      major_ptr = FFI::MemoryPointer.new(:int)
      minor_ptr = FFI::MemoryPointer.new(:int)
      result = FFIBindings.hedl_get_version(@ptr, major_ptr, minor_ptr)
      raise Error.from_lib(result) unless result == ErrorCode::OK

      [major_ptr.read_int, minor_ptr.read_int]
    end

    def schema_count
      check_closed
      c = FFIBindings.hedl_schema_count(@ptr)
      raise Error.from_lib(c) if c.negative?

      c
    end

    def alias_count
      check_closed
      c = FFIBindings.hedl_alias_count(@ptr)
      raise Error.from_lib(c) if c.negative?

      c
    end

    def root_item_count
      check_closed
      c = FFIBindings.hedl_root_item_count(@ptr)
      raise Error.from_lib(c) if c.negative?

      c
    end

    def canonicalize
      check_closed
      out_ptr = FFI::MemoryPointer.new(:pointer)
      result = FFIBindings.hedl_canonicalize(@ptr, out_ptr)
      raise Error.from_lib(result) unless result == ErrorCode::OK

      str_ptr = out_ptr.read_pointer
      output = str_ptr.read_string
      FFIBindings.hedl_free_string(str_ptr)

      # Check output size limit
      output_size = output.bytesize
      if output_size > MAX_OUTPUT_SIZE
        actual_mb = output_size / 1048576.0
        limit_mb = MAX_OUTPUT_SIZE / 1048576.0
        raise Error.new(
          format("Output size (%.2fMB) exceeds limit (%.2fMB). Set HEDL_MAX_OUTPUT_SIZE to increase.",
                 actual_mb, limit_mb),
          ErrorCode::ALLOC
        )
      end

      output
    end

    def to_json(include_metadata: false)
      check_closed
      out_ptr = FFI::MemoryPointer.new(:pointer)
      result = FFIBindings.hedl_to_json(@ptr, include_metadata ? 1 : 0, out_ptr)
      raise Error.from_lib(result) unless result == ErrorCode::OK

      str_ptr = out_ptr.read_pointer
      output = str_ptr.read_string
      FFIBindings.hedl_free_string(str_ptr)

      # Check output size limit
      output_size = output.bytesize
      if output_size > MAX_OUTPUT_SIZE
        actual_mb = output_size / 1048576.0
        limit_mb = MAX_OUTPUT_SIZE / 1048576.0
        raise Error.new(
          format("Output size (%.2fMB) exceeds limit (%.2fMB). Set HEDL_MAX_OUTPUT_SIZE to increase.",
                 actual_mb, limit_mb),
          ErrorCode::ALLOC
        )
      end

      output
    end

    def to_yaml(include_metadata: false)
      check_closed
      out_ptr = FFI::MemoryPointer.new(:pointer)
      result = FFIBindings.hedl_to_yaml(@ptr, include_metadata ? 1 : 0, out_ptr)
      raise Error.from_lib(result) unless result == ErrorCode::OK

      str_ptr = out_ptr.read_pointer
      output = str_ptr.read_string
      FFIBindings.hedl_free_string(str_ptr)

      # Check output size limit
      output_size = output.bytesize
      if output_size > MAX_OUTPUT_SIZE
        actual_mb = output_size / 1048576.0
        limit_mb = MAX_OUTPUT_SIZE / 1048576.0
        raise Error.new(
          format("Output size (%.2fMB) exceeds limit (%.2fMB). Set HEDL_MAX_OUTPUT_SIZE to increase.",
                 actual_mb, limit_mb),
          ErrorCode::ALLOC
        )
      end

      output
    end

    def to_xml
      check_closed
      out_ptr = FFI::MemoryPointer.new(:pointer)
      result = FFIBindings.hedl_to_xml(@ptr, out_ptr)
      raise Error.from_lib(result) unless result == ErrorCode::OK

      str_ptr = out_ptr.read_pointer
      output = str_ptr.read_string
      FFIBindings.hedl_free_string(str_ptr)

      # Check output size limit
      output_size = output.bytesize
      if output_size > MAX_OUTPUT_SIZE
        actual_mb = output_size / 1048576.0
        limit_mb = MAX_OUTPUT_SIZE / 1048576.0
        raise Error.new(
          format("Output size (%.2fMB) exceeds limit (%.2fMB). Set HEDL_MAX_OUTPUT_SIZE to increase.",
                 actual_mb, limit_mb),
          ErrorCode::ALLOC
        )
      end

      output
    end

    def to_csv
      check_closed
      out_ptr = FFI::MemoryPointer.new(:pointer)
      result = FFIBindings.hedl_to_csv(@ptr, out_ptr)
      raise Error.from_lib(result) unless result == ErrorCode::OK

      str_ptr = out_ptr.read_pointer
      output = str_ptr.read_string
      FFIBindings.hedl_free_string(str_ptr)

      # Check output size limit
      output_size = output.bytesize
      if output_size > MAX_OUTPUT_SIZE
        actual_mb = output_size / 1048576.0
        limit_mb = MAX_OUTPUT_SIZE / 1048576.0
        raise Error.new(
          format("Output size (%.2fMB) exceeds limit (%.2fMB). Set HEDL_MAX_OUTPUT_SIZE to increase.",
                 actual_mb, limit_mb),
          ErrorCode::ALLOC
        )
      end

      output
    end

    def to_parquet
      check_closed
      data_ptr = FFI::MemoryPointer.new(:pointer)
      len_ptr = FFI::MemoryPointer.new(:size_t)
      result = FFIBindings.hedl_to_parquet(@ptr, data_ptr, len_ptr)
      raise Error.from_lib(result) unless result == ErrorCode::OK

      ptr = data_ptr.read_pointer
      len = len_ptr.read(:size_t)

      # Check output size limit
      if len > MAX_OUTPUT_SIZE
        FFIBindings.hedl_free_bytes(ptr, len)
        actual_mb = len / 1048576.0
        limit_mb = MAX_OUTPUT_SIZE / 1048576.0
        raise Error.new(
          format("Output size (%.2fMB) exceeds limit (%.2fMB). Set HEDL_MAX_OUTPUT_SIZE to increase.",
                 actual_mb, limit_mb),
          ErrorCode::ALLOC
        )
      end

      data = ptr.read_bytes(len)
      FFIBindings.hedl_free_bytes(ptr, len)
      data
    end

    def to_cypher(use_merge: true)
      check_closed
      out_ptr = FFI::MemoryPointer.new(:pointer)
      result = FFIBindings.hedl_to_neo4j_cypher(@ptr, use_merge ? 1 : 0, out_ptr)
      raise Error.from_lib(result) unless result == ErrorCode::OK

      str_ptr = out_ptr.read_pointer
      output = str_ptr.read_string
      FFIBindings.hedl_free_string(str_ptr)

      # Check output size limit
      output_size = output.bytesize
      if output_size > MAX_OUTPUT_SIZE
        actual_mb = output_size / 1048576.0
        limit_mb = MAX_OUTPUT_SIZE / 1048576.0
        raise Error.new(
          format("Output size (%.2fMB) exceeds limit (%.2fMB). Set HEDL_MAX_OUTPUT_SIZE to increase.",
                 actual_mb, limit_mb),
          ErrorCode::ALLOC
        )
      end

      output
    end

    def lint
      check_closed
      diag_ptr = FFI::MemoryPointer.new(:pointer)
      result = FFIBindings.hedl_lint(@ptr, diag_ptr)
      raise Error.from_lib(result) unless result == ErrorCode::OK

      Diagnostics.new(diag_ptr.read_pointer)
    end

    private

    def check_closed
      raise Error.new('Document already closed', ErrorCode::NULL_PTR) if @closed
    end
  end

  # Module methods

  # Parse HEDL content into a Document
  def self.parse(content, strict: true)
    doc_ptr = FFI::MemoryPointer.new(:pointer)
    result = FFIBindings.hedl_parse(content, content.bytesize, strict ? 1 : 0, doc_ptr)
    raise Error.from_lib(result) unless result == ErrorCode::OK

    Document.new(doc_ptr.read_pointer)
  end

  # Validate HEDL content without creating a document
  def self.validate(content, strict: true)
    result = FFIBindings.hedl_validate(content, content.bytesize, strict ? 1 : 0)
    result == ErrorCode::OK
  end

  # Parse JSON content into a HEDL Document
  def self.from_json(content)
    doc_ptr = FFI::MemoryPointer.new(:pointer)
    result = FFIBindings.hedl_from_json(content, content.bytesize, doc_ptr)
    raise Error.from_lib(result) unless result == ErrorCode::OK

    Document.new(doc_ptr.read_pointer)
  end

  # Parse YAML content into a HEDL Document
  def self.from_yaml(content)
    doc_ptr = FFI::MemoryPointer.new(:pointer)
    result = FFIBindings.hedl_from_yaml(content, content.bytesize, doc_ptr)
    raise Error.from_lib(result) unless result == ErrorCode::OK

    Document.new(doc_ptr.read_pointer)
  end

  # Parse XML content into a HEDL Document
  def self.from_xml(content)
    doc_ptr = FFI::MemoryPointer.new(:pointer)
    result = FFIBindings.hedl_from_xml(content, content.bytesize, doc_ptr)
    raise Error.from_lib(result) unless result == ErrorCode::OK

    Document.new(doc_ptr.read_pointer)
  end

  # Parse Parquet content into a HEDL Document
  def self.from_parquet(data)
    data_ptr = FFI::MemoryPointer.from_string(data)
    doc_ptr = FFI::MemoryPointer.new(:pointer)
    result = FFIBindings.hedl_from_parquet(data_ptr, data.bytesize, doc_ptr)
    raise Error.from_lib(result) unless result == ErrorCode::OK

    Document.new(doc_ptr.read_pointer)
  end
end
