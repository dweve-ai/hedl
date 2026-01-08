<?php
/**
 * Tests for HEDL PHP bindings.
 */

declare(strict_types=1);

require_once __DIR__ . '/../Hedl.php';
require_once __DIR__ . '/Fixtures.php';

use Dweve\Hedl\Hedl;
use Dweve\Hedl\Document;
use Dweve\Hedl\Diagnostics;
use Dweve\Hedl\HedlException;
use Dweve\Hedl\Tests\Fixtures;

// Set library path
putenv('HEDL_LIB_PATH=' . realpath(__DIR__ . '/../../..') . '/target/release/libhedl_ffi.so');

class HedlTest
{
    // Use shared fixtures from common/fixtures directory
    private static Fixtures $fixtures;
    private static string $sampleHEDL;
    private static string $sampleJSON;
    private static string $sampleYAML;
    private static string $sampleXML;

    public static function init(): void
    {
        self::$fixtures = new Fixtures();
        self::$sampleHEDL = self::$fixtures->basicHedl();
        self::$sampleJSON = self::$fixtures->basicJson();
        self::$sampleYAML = self::$fixtures->basicYaml();
        self::$sampleXML = self::$fixtures->basicXml();
    }

    private static int $passed = 0;
    private static int $failed = 0;

    public static function run(): void
    {
        $tests = [
            'testParse',
            'testParseInvalid',
            'testValidate',
            'testValidateInvalid',
            'testVersion',
            'testSchemaCount',
            'testCanonicalize',
            'testToJson',
            'testToYaml',
            'testToXml',
            'testToCsv',
            'testToCypher',
            // 'testToParquet', // toParquet not implemented in PHP bindings
            'testFromJson',
            'testFromYaml',
            'testFromXml',
            'testLint',
            'testDoubleClose',
        ];

        foreach ($tests as $test) {
            try {
                self::$test();
                echo "✓ {$test}\n";
                self::$passed++;
            } catch (Throwable $e) {
                echo "✗ {$test}: {$e->getMessage()}\n";
                self::$failed++;
            }
        }

        echo "\n";
        echo "Passed: " . self::$passed . "\n";
        echo "Failed: " . self::$failed . "\n";

        if (self::$failed > 0) {
            exit(1);
        }
    }

    private static function assert(bool $condition, string $message): void
    {
        if (!$condition) {
            throw new Exception($message);
        }
    }

    public static function testParse(): void
    {
        $doc = Hedl::parse(self::$sampleHEDL);
        self::assert($doc instanceof Document, "Expected Document instance");
        $doc->close();
    }

    public static function testParseInvalid(): void
    {
        $thrown = false;
        try {
            Hedl::parse(self::$fixtures->errorInvalidSyntax());
        } catch (Throwable $e) {
            // Accept any exception when parsing invalid content
            $thrown = true;
        }
        self::assert($thrown, "Expected exception for invalid content");
    }

    public static function testValidate(): void
    {
        self::assert(Hedl::validate(self::$sampleHEDL), "Expected valid content to pass");
    }

    public static function testValidateInvalid(): void
    {
        self::assert(!Hedl::validate(self::$fixtures->errorInvalidSyntax()), "Expected invalid content to fail");
    }

    public static function testVersion(): void
    {
        $doc = Hedl::parse(self::$sampleHEDL);
        [$major, $minor] = $doc->version();
        self::assert($major === 1 && $minor === 0, "Expected version 1.0, got {$major}.{$minor}");
        $doc->close();
    }

    public static function testSchemaCount(): void
    {
        $doc = Hedl::parse(self::$sampleHEDL);
        self::assert($doc->schemaCount() === 1, "Expected 1 schema");
        $doc->close();
    }

    public static function testCanonicalize(): void
    {
        $doc = Hedl::parse(self::$sampleHEDL);
        $canonical = $doc->canonicalize();
        self::assert(strlen($canonical) > 0, "Expected non-empty canonical output");
        self::assert(str_contains($canonical, '%VERSION'), "Expected %VERSION in output");
        $doc->close();
    }

    public static function testToJson(): void
    {
        $doc = Hedl::parse(self::$sampleHEDL);
        $json = $doc->toJson();
        self::assert(strlen($json) > 0, "Expected non-empty JSON output");
        json_decode($json);
        self::assert(json_last_error() === JSON_ERROR_NONE, "Expected valid JSON");
        $doc->close();
    }

    public static function testToYaml(): void
    {
        $doc = Hedl::parse(self::$sampleHEDL);
        $yaml = $doc->toYaml();
        self::assert(strlen($yaml) > 0, "Expected non-empty YAML output");
        $doc->close();
    }

    public static function testToXml(): void
    {
        $doc = Hedl::parse(self::$sampleHEDL);
        $xml = $doc->toXml();
        self::assert(strlen($xml) > 0, "Expected non-empty XML output");
        self::assert(str_contains($xml, '<'), "Expected XML tags");
        $doc->close();
    }

    public static function testToCsv(): void
    {
        $doc = Hedl::parse(self::$sampleHEDL);
        $csv = $doc->toCsv();
        self::assert(strlen($csv) > 0, "Expected non-empty CSV output");
        $doc->close();
    }

    public static function testToCypher(): void
    {
        $doc = Hedl::parse(self::$sampleHEDL);
        $cypher = $doc->toCypher();
        self::assert(strlen($cypher) > 0, "Expected non-empty Cypher output");
        $doc->close();
    }

    public static function testToParquet(): void
    {
        $doc = Hedl::parse(self::$sampleHEDL);
        $data = $doc->toParquet();
        self::assert(strlen($data) > 0, "Expected non-empty Parquet output");
        $doc->close();
    }

    public static function testFromJson(): void
    {
        $doc = Hedl::fromJson(self::$sampleJSON);
        self::assert($doc instanceof Document, "Expected Document instance");
        $canonical = $doc->canonicalize();
        self::assert(strlen($canonical) > 0, "Expected non-empty canonical output");
        $doc->close();
    }

    public static function testFromYaml(): void
    {
        $doc = Hedl::fromYaml(self::$sampleYAML);
        self::assert($doc instanceof Document, "Expected Document instance");
        $doc->close();
    }

    public static function testFromXml(): void
    {
        $doc = Hedl::fromXml(self::$sampleXML);
        self::assert($doc instanceof Document, "Expected Document instance");
        $doc->close();
    }

    public static function testLint(): void
    {
        $doc = Hedl::parse(self::$sampleHEDL);
        $diag = $doc->lint();
        self::assert($diag instanceof Diagnostics, "Expected Diagnostics instance");
        self::assert($diag->count() >= 0, "Expected count >= 0");
        $diag->close();
        $doc->close();
    }

    public static function testDoubleClose(): void
    {
        $doc = Hedl::parse(self::$sampleHEDL);
        $doc->close();
        $doc->close(); // Should not throw
    }
}

// Initialize and run tests
HedlTest::init();
HedlTest::run();
