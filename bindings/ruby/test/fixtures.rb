# frozen_string_literal: true

##
# Common test fixtures loader for HEDL Ruby bindings.
#
# This module provides access to shared test fixtures stored in the
# bindings/common/fixtures directory, eliminating test data duplication
# across language bindings.

require 'json'

module Hedl
  ##
  # Loads and provides access to common HEDL test fixtures.
  #
  # All fixtures are loaded from bindings/common/fixtures directory
  # to ensure consistency across language bindings.
  class Fixtures
    ##
    # Initialize the fixtures loader and load manifest.
    def initialize
      # Path to common fixtures directory
      # From bindings/ruby/test/fixtures.rb -> bindings/common/fixtures
      @fixtures_dir = File.expand_path('../../common/fixtures', __dir__)

      # Load manifest
      manifest_path = File.join(@fixtures_dir, 'manifest.json')
      @manifest = JSON.parse(File.read(manifest_path))
    end

    ##
    # Read a fixture file.
    #
    # @param filename [String] Name of the file to read
    # @param binary [Boolean] If true, read as binary
    # @return [String] File contents
    def read_file(filename, binary: false)
      filepath = File.join(@fixtures_dir, filename)
      mode = binary ? 'rb' : 'r'
      File.read(filepath, mode: mode, encoding: binary ? nil : 'UTF-8')
    end

    # Basic fixtures

    ##
    # Get basic HEDL sample document.
    # @return [String] HEDL content
    def basic_hedl
      read_file(@manifest['fixtures']['basic']['files']['hedl'])
    end

    ##
    # Get basic JSON sample document.
    # @return [String] JSON content
    def basic_json
      read_file(@manifest['fixtures']['basic']['files']['json'])
    end

    ##
    # Get basic YAML sample document.
    # @return [String] YAML content
    def basic_yaml
      read_file(@manifest['fixtures']['basic']['files']['yaml'])
    end

    ##
    # Get basic XML sample document.
    # @return [String] XML content
    def basic_xml
      read_file(@manifest['fixtures']['basic']['files']['xml'])
    end

    # Type-specific fixtures

    ##
    # Get HEDL document with various scalar types.
    # @return [String] HEDL content
    def scalars_hedl
      read_file(@manifest['fixtures']['scalars']['files']['hedl'])
    end

    ##
    # Get HEDL document with nested structures.
    # @return [String] HEDL content
    def nested_hedl
      read_file(@manifest['fixtures']['nested']['files']['hedl'])
    end

    ##
    # Get HEDL document with lists and arrays.
    # @return [String] HEDL content
    def lists_hedl
      read_file(@manifest['fixtures']['lists']['files']['hedl'])
    end

    # Performance fixtures

    ##
    # Get large HEDL document for performance testing.
    # @return [String] HEDL content
    def large_hedl
      read_file(@manifest['fixtures']['large']['files']['hedl'])
    end

    # Error fixtures

    ##
    # Get invalid HEDL syntax for error testing.
    # @return [String] Invalid HEDL content
    def error_invalid_syntax
      read_file(@manifest['errors']['invalid_syntax']['file'])
    end

    ##
    # Get malformed HEDL document for error testing.
    # @return [String] Malformed HEDL content
    def error_malformed
      read_file(@manifest['errors']['malformed']['file'])
    end

    # Utility methods

    ##
    # Get a specific fixture by category and name.
    #
    # @param category [String] Fixture category ("basic", "scalars", etc.)
    # @param name [String] Fixture name (same as category for most)
    # @param format [String] File format ("hedl", "json", "yaml", "xml")
    # @return [String] Fixture content
    #
    # @example
    #   fixtures = Fixtures.new
    #   hedl = fixtures.get_fixture("basic", "basic", "hedl")
    def get_fixture(category, name = category, format = 'hedl')
      if @manifest['fixtures'].key?(category)
        files = @manifest['fixtures'][category]['files']
        if files.key?(format)
          return read_file(files[format])
        end
      end

      raise ArgumentError, "Fixture not found: category=#{category}, format=#{format}"
    end

    ##
    # Get an error fixture by type.
    #
    # @param error_type [String] Type of error ("invalid_syntax", "malformed")
    # @return [String] Error fixture content
    def get_error_fixture(error_type)
      if @manifest['errors'].key?(error_type)
        return read_file(@manifest['errors'][error_type]['file'])
      end

      raise ArgumentError, "Error fixture not found: #{error_type}"
    end
  end
end

# Global fixtures instance for convenient access
$hedl_fixtures = Hedl::Fixtures.new

# Legacy constants for backward compatibility
SAMPLE_HEDL = $hedl_fixtures.basic_hedl
SAMPLE_JSON = $hedl_fixtures.basic_json
SAMPLE_YAML = $hedl_fixtures.basic_yaml
SAMPLE_XML = $hedl_fixtures.basic_xml
