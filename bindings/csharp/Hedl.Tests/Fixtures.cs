// Common test fixtures loader for HEDL C# bindings.
//
// This class provides access to shared test fixtures stored in the
// bindings/common/fixtures directory, eliminating test data duplication
// across language bindings.

using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;

namespace Dweve.Hedl.Tests
{
    /// <summary>
    /// Manifest structure for fixture definitions.
    /// </summary>
    internal class FixtureManifest
    {
        public Dictionary<string, FixtureEntry> Fixtures { get; set; } = new();
        public Dictionary<string, ErrorEntry> Errors { get; set; } = new();
    }

    /// <summary>
    /// Fixture entry in the manifest.
    /// </summary>
    internal class FixtureEntry
    {
        public string Description { get; set; } = string.Empty;
        public Dictionary<string, string> Files { get; set; } = new();
    }

    /// <summary>
    /// Error fixture entry in the manifest.
    /// </summary>
    internal class ErrorEntry
    {
        public string Description { get; set; } = string.Empty;
        public string File { get; set; } = string.Empty;
        public bool ExpectedError { get; set; }
    }

    /// <summary>
    /// Loads and provides access to common HEDL test fixtures.
    ///
    /// All fixtures are loaded from bindings/common/fixtures directory
    /// to ensure consistency across language bindings.
    /// </summary>
    public class Fixtures
    {
        private readonly string _fixturesDir;
        private readonly FixtureManifest _manifest;

        /// <summary>
        /// Initialize the fixtures loader and load manifest.
        /// </summary>
        public Fixtures()
        {
            // Path to common fixtures directory
            // Navigate from bin/Debug/net6.0 (or similar) to common/fixtures
            var baseDir = AppDomain.CurrentDomain.BaseDirectory;
            _fixturesDir = Path.GetFullPath(
                Path.Combine(baseDir, "..", "..", "..", "..", "..", "common", "fixtures")
            );

            // Load manifest
            var manifestPath = Path.Combine(_fixturesDir, "manifest.json");
            if (!File.Exists(manifestPath))
            {
                throw new FileNotFoundException($"Manifest not found at: {manifestPath}");
            }

            var manifestContent = File.ReadAllText(manifestPath);
            var options = new JsonSerializerOptions
            {
                PropertyNamingPolicy = JsonNamingPolicy.CamelCase
            };
            _manifest = JsonSerializer.Deserialize<FixtureManifest>(manifestContent, options)
                ?? throw new InvalidOperationException("Failed to parse manifest");
        }

        /// <summary>
        /// Read a fixture file.
        /// </summary>
        /// <param name="filename">Name of the file to read</param>
        /// <returns>File contents as string</returns>
        private string ReadFile(string filename)
        {
            var filepath = Path.Combine(_fixturesDir, filename);
            if (!File.Exists(filepath))
            {
                throw new FileNotFoundException($"Fixture file not found: {filepath}");
            }

            return File.ReadAllText(filepath);
        }

        // Basic fixtures

        /// <summary>
        /// Get basic HEDL sample document.
        /// </summary>
        public string BasicHedl => ReadFile(_manifest.Fixtures["basic"].Files["hedl"]);

        /// <summary>
        /// Get basic JSON sample document.
        /// </summary>
        public string BasicJson => ReadFile(_manifest.Fixtures["basic"].Files["json"]);

        /// <summary>
        /// Get basic YAML sample document.
        /// </summary>
        public string BasicYaml => ReadFile(_manifest.Fixtures["basic"].Files["yaml"]);

        /// <summary>
        /// Get basic XML sample document.
        /// </summary>
        public string BasicXml => ReadFile(_manifest.Fixtures["basic"].Files["xml"]);

        // Type-specific fixtures

        /// <summary>
        /// Get HEDL document with various scalar types.
        /// </summary>
        public string ScalarsHedl => ReadFile(_manifest.Fixtures["scalars"].Files["hedl"]);

        /// <summary>
        /// Get HEDL document with nested structures.
        /// </summary>
        public string NestedHedl => ReadFile(_manifest.Fixtures["nested"].Files["hedl"]);

        /// <summary>
        /// Get HEDL document with lists and arrays.
        /// </summary>
        public string ListsHedl => ReadFile(_manifest.Fixtures["lists"].Files["hedl"]);

        // Performance fixtures

        /// <summary>
        /// Get large HEDL document for performance testing.
        /// </summary>
        public string LargeHedl => ReadFile(_manifest.Fixtures["large"].Files["hedl"]);

        // Error fixtures

        /// <summary>
        /// Get invalid HEDL syntax for error testing.
        /// </summary>
        public string ErrorInvalidSyntax => ReadFile(_manifest.Errors["invalid_syntax"].File);

        /// <summary>
        /// Get malformed HEDL document for error testing.
        /// </summary>
        public string ErrorMalformed => ReadFile(_manifest.Errors["malformed"].File);

        // Utility methods

        /// <summary>
        /// Get a specific fixture by category and format.
        /// </summary>
        /// <param name="category">Fixture category ("basic", "scalars", etc.)</param>
        /// <param name="format">File format ("hedl", "json", "yaml", "xml")</param>
        /// <returns>Fixture content as string</returns>
        /// <example>
        /// <code>
        /// var fixtures = new Fixtures();
        /// var hedl = fixtures.GetFixture("basic", "hedl");
        /// </code>
        /// </example>
        public string GetFixture(string category, string format = "hedl")
        {
            if (_manifest.Fixtures.TryGetValue(category, out var entry))
            {
                if (entry.Files.TryGetValue(format, out var filename))
                {
                    return ReadFile(filename);
                }
            }

            throw new ArgumentException($"Fixture not found: category={category}, format={format}");
        }

        /// <summary>
        /// Get an error fixture by type.
        /// </summary>
        /// <param name="errorType">Type of error ("invalid_syntax", "malformed")</param>
        /// <returns>Error fixture content</returns>
        public string GetErrorFixture(string errorType)
        {
            if (_manifest.Errors.TryGetValue(errorType, out var entry))
            {
                return ReadFile(entry.File);
            }

            throw new ArgumentException($"Error fixture not found: {errorType}");
        }
    }

    /// <summary>
    /// Global fixtures instance for convenient access.
    /// </summary>
    public static class GlobalFixtures
    {
        private static readonly Lazy<Fixtures> _instance = new(() => new Fixtures());

        /// <summary>
        /// Get the global fixtures instance.
        /// </summary>
        public static Fixtures Instance => _instance.Value;

        // Legacy constants for backward compatibility
        public static string SampleHedl => Instance.BasicHedl;
        public static string SampleJson => Instance.BasicJson;
        public static string SampleYaml => Instance.BasicYaml;
        public static string SampleXml => Instance.BasicXml;
    }
}
