<?php
/**
 * Common test fixtures loader for HEDL PHP bindings.
 *
 * This class provides access to shared test fixtures stored in the
 * bindings/common/fixtures directory, eliminating test data duplication
 * across language bindings.
 */

declare(strict_types=1);

namespace Dweve\Hedl\Tests;

use RuntimeException;

/**
 * Loads and provides access to common HEDL test fixtures.
 *
 * All fixtures are loaded from bindings/common/fixtures directory
 * to ensure consistency across language bindings.
 */
class Fixtures
{
    /** @var string Path to fixtures directory */
    private string $fixturesDir;

    /** @var array<string, mixed> Parsed manifest data */
    private array $manifest;

    /**
     * Initialize the fixtures loader and load manifest.
     */
    public function __construct()
    {
        // Path to common fixtures directory
        // From bindings/php/tests/Fixtures.php -> bindings/common/fixtures
        $this->fixturesDir = realpath(__DIR__ . '/../..') . '/common/fixtures';

        // Load manifest
        $manifestPath = $this->fixturesDir . '/manifest.json';
        if (!file_exists($manifestPath)) {
            throw new RuntimeException("Manifest not found at: {$manifestPath}");
        }

        $manifestContent = file_get_contents($manifestPath);
        if ($manifestContent === false) {
            throw new RuntimeException("Failed to read manifest");
        }

        $this->manifest = json_decode($manifestContent, true, 512, JSON_THROW_ON_ERROR);
    }

    /**
     * Read a fixture file.
     *
     * @param string $filename Name of the file to read
     * @return string File contents
     * @throws RuntimeException If file cannot be read
     */
    private function readFile(string $filename): string
    {
        $filepath = $this->fixturesDir . '/' . $filename;
        if (!file_exists($filepath)) {
            throw new RuntimeException("Fixture file not found: {$filepath}");
        }

        $content = file_get_contents($filepath);
        if ($content === false) {
            throw new RuntimeException("Failed to read fixture: {$filepath}");
        }

        return $content;
    }

    // Basic fixtures

    /**
     * Get basic HEDL sample document.
     *
     * @return string HEDL content
     */
    public function basicHedl(): string
    {
        return $this->readFile($this->manifest['fixtures']['basic']['files']['hedl']);
    }

    /**
     * Get basic JSON sample document.
     *
     * @return string JSON content
     */
    public function basicJson(): string
    {
        return $this->readFile($this->manifest['fixtures']['basic']['files']['json']);
    }

    /**
     * Get basic YAML sample document.
     *
     * @return string YAML content
     */
    public function basicYaml(): string
    {
        return $this->readFile($this->manifest['fixtures']['basic']['files']['yaml']);
    }

    /**
     * Get basic XML sample document.
     *
     * @return string XML content
     */
    public function basicXml(): string
    {
        return $this->readFile($this->manifest['fixtures']['basic']['files']['xml']);
    }

    // Type-specific fixtures

    /**
     * Get HEDL document with various scalar types.
     *
     * @return string HEDL content
     */
    public function scalarsHedl(): string
    {
        return $this->readFile($this->manifest['fixtures']['scalars']['files']['hedl']);
    }

    /**
     * Get HEDL document with nested structures.
     *
     * @return string HEDL content
     */
    public function nestedHedl(): string
    {
        return $this->readFile($this->manifest['fixtures']['nested']['files']['hedl']);
    }

    /**
     * Get HEDL document with lists and arrays.
     *
     * @return string HEDL content
     */
    public function listsHedl(): string
    {
        return $this->readFile($this->manifest['fixtures']['lists']['files']['hedl']);
    }

    // Performance fixtures

    /**
     * Get large HEDL document for performance testing.
     *
     * @return string HEDL content
     */
    public function largeHedl(): string
    {
        return $this->readFile($this->manifest['fixtures']['large']['files']['hedl']);
    }

    // Error fixtures

    /**
     * Get invalid HEDL syntax for error testing.
     *
     * @return string Invalid HEDL content
     */
    public function errorInvalidSyntax(): string
    {
        return $this->readFile($this->manifest['errors']['invalid_syntax']['file']);
    }

    /**
     * Get malformed HEDL document for error testing.
     *
     * @return string Malformed HEDL content
     */
    public function errorMalformed(): string
    {
        return $this->readFile($this->manifest['errors']['malformed']['file']);
    }

    // Utility methods

    /**
     * Get a specific fixture by category and format.
     *
     * @param string $category Fixture category ("basic", "scalars", etc.)
     * @param string $name Fixture name (same as category for most)
     * @param string $format File format ("hedl", "json", "yaml", "xml")
     * @return string Fixture content
     * @throws RuntimeException If fixture not found
     *
     * @example
     * ```php
     * $fixtures = new Fixtures();
     * $hedl = $fixtures->getFixture("basic", "basic", "hedl");
     * ```
     */
    public function getFixture(string $category, string $name = '', string $format = 'hedl'): string
    {
        if ($name === '') {
            $name = $category;
        }

        if (isset($this->manifest['fixtures'][$category])) {
            $files = $this->manifest['fixtures'][$category]['files'];
            if (isset($files[$format])) {
                return $this->readFile($files[$format]);
            }
        }

        throw new RuntimeException("Fixture not found: category={$category}, format={$format}");
    }

    /**
     * Get an error fixture by type.
     *
     * @param string $errorType Type of error ("invalid_syntax", "malformed")
     * @return string Error fixture content
     * @throws RuntimeException If error fixture not found
     */
    public function getErrorFixture(string $errorType): string
    {
        if (isset($this->manifest['errors'][$errorType])) {
            return $this->readFile($this->manifest['errors'][$errorType]['file']);
        }

        throw new RuntimeException("Error fixture not found: {$errorType}");
    }
}

// Global fixtures instance for convenient access
$GLOBALS['hedl_fixtures'] = new Fixtures();

/**
 * Get the global fixtures instance.
 *
 * @return Fixtures
 */
function getFixtures(): Fixtures
{
    return $GLOBALS['hedl_fixtures'];
}

// Legacy constants for backward compatibility
define('SAMPLE_HEDL', getFixtures()->basicHedl());
define('SAMPLE_JSON', getFixtures()->basicJson());
define('SAMPLE_YAML', getFixtures()->basicYaml());
define('SAMPLE_XML', getFixtures()->basicXml());
