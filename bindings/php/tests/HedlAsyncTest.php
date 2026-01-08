<?php
/**
 * Async tests for HEDL PHP bindings using AMPHP.
 *
 * Requires: amphp/amp >= 2.6.0
 *
 * Install with: composer require amphp/amp
 */

declare(strict_types=1);

require_once __DIR__ . '/../Hedl.php';
require_once __DIR__ . '/Fixtures.php';

use Dweve\Hedl\Hedl;
use Dweve\Hedl\Document;
use Dweve\Hedl\AsyncDocument;
use Dweve\Hedl\HedlException;
use Dweve\Hedl\Tests\Fixtures;
use Amp\Promise;
use Amp\Loop;

// Check if AMPHP is installed
if (!class_exists('Amp\Promise')) {
    echo "AMPHP is not installed. Install with: composer require amphp/amp\n";
    exit(1);
}

// Set library path
putenv('HEDL_LIB_PATH=' . realpath(__DIR__ . '/../../..') . '/target/release/libhedl_ffi.so');

class HedlAsyncTest
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
            'testParseAsync',
            'testParseAsyncInvalid',
            'testValidateAsync',
            'testValidateAsyncInvalid',
            'testFromJsonAsync',
            'testFromYamlAsync',
            'testFromXmlAsync',
            'testAsyncDocumentToJson',
            'testAsyncDocumentToYaml',
            'testAsyncDocumentToXml',
            'testAsyncDocumentToCsv',
            'testAsyncDocumentCanonical',
            'testAsyncDocumentLint',
            'testWrapAsync',
            'testPromiseComposition',
            'testAsyncErrorHandling',
        ];

        Loop::run(function() use ($tests) {
            foreach ($tests as $test) {
                try {
                    yield self::$test();
                    echo "✓ {$test}\n";
                    self::$passed++;
                } catch (Throwable $e) {
                    echo "✗ {$test}: {$e->getMessage()}\n";
                    if (getenv('DEBUG')) {
                        echo "Stack trace:\n{$e->getTraceAsString()}\n";
                    }
                    self::$failed++;
                }
            }

            echo "\n";
            echo "Passed: " . self::$passed . "\n";
            echo "Failed: " . self::$failed . "\n";

            if (self::$failed > 0) {
                exit(1);
            }
        });
    }

    private static function assert(bool $condition, string $message): void
    {
        if (!$condition) {
            throw new Exception($message);
        }
    }

    public static function testParseAsync(): Promise
    {
        return Amp\call(function() {
            $doc = yield Hedl::parseAsync(self::$sampleHEDL);
            self::assert($doc instanceof Document, "Expected Document instance");
            $doc->close();
        });
    }

    public static function testParseAsyncInvalid(): Promise
    {
        return Amp\call(function() {
            $thrown = false;
            try {
                yield Hedl::parseAsync(self::$fixtures->errorInvalidSyntax());
            } catch (HedlException $e) {
                $thrown = true;
            }
            self::assert($thrown, "Expected exception for invalid content");
        });
    }

    public static function testValidateAsync(): Promise
    {
        return Amp\call(function() {
            $result = yield Hedl::validateAsync(self::$sampleHEDL);
            self::assert($result === true, "Expected validation to pass");
        });
    }

    public static function testValidateAsyncInvalid(): Promise
    {
        return Amp\call(function() {
            $result = yield Hedl::validateAsync(self::$fixtures->errorInvalidSyntax());
            self::assert($result === false, "Expected validation to fail");
        });
    }

    public static function testFromJsonAsync(): Promise
    {
        return Amp\call(function() {
            $doc = yield Hedl::fromJsonAsync(self::$sampleJSON);
            self::assert($doc instanceof Document, "Expected Document instance");
            $canonical = $doc->canonicalize();
            self::assert(strlen($canonical) > 0, "Expected non-empty canonical output");
            $doc->close();
        });
    }

    public static function testFromYamlAsync(): Promise
    {
        return Amp\call(function() {
            $doc = yield Hedl::fromYamlAsync(self::$sampleYAML);
            self::assert($doc instanceof Document, "Expected Document instance");
            $doc->close();
        });
    }

    public static function testFromXmlAsync(): Promise
    {
        return Amp\call(function() {
            $doc = yield Hedl::fromXmlAsync(self::$sampleXML);
            self::assert($doc instanceof Document, "Expected Document instance");
            $doc->close();
        });
    }

    public static function testAsyncDocumentToJson(): Promise
    {
        return Amp\call(function() {
            $doc = Hedl::parse(self::$sampleHEDL);
            $asyncDoc = Hedl::wrapAsync($doc);

            $json = yield $asyncDoc->toJsonAsync();
            self::assert(strlen($json) > 0, "Expected non-empty JSON");
            json_decode($json);
            self::assert(json_last_error() === JSON_ERROR_NONE, "Expected valid JSON");

            $doc->close();
        });
    }

    public static function testAsyncDocumentToYaml(): Promise
    {
        return Amp\call(function() {
            $doc = Hedl::parse(self::$sampleHEDL);
            $asyncDoc = Hedl::wrapAsync($doc);

            $yaml = yield $asyncDoc->toYamlAsync();
            self::assert(strlen($yaml) > 0, "Expected non-empty YAML");

            $doc->close();
        });
    }

    public static function testAsyncDocumentToXml(): Promise
    {
        return Amp\call(function() {
            $doc = Hedl::parse(self::$sampleHEDL);
            $asyncDoc = Hedl::wrapAsync($doc);

            $xml = yield $asyncDoc->toXmlAsync();
            self::assert(strlen($xml) > 0, "Expected non-empty XML");
            self::assert(str_contains($xml, '<'), "Expected XML tags");

            $doc->close();
        });
    }

    public static function testAsyncDocumentToCsv(): Promise
    {
        return Amp\call(function() {
            $doc = Hedl::parse(self::$sampleHEDL);
            $asyncDoc = Hedl::wrapAsync($doc);

            $csv = yield $asyncDoc->toCsvAsync();
            self::assert(strlen($csv) > 0, "Expected non-empty CSV");

            $doc->close();
        });
    }

    public static function testAsyncDocumentCanonical(): Promise
    {
        return Amp\call(function() {
            $doc = Hedl::parse(self::$sampleHEDL);
            $asyncDoc = Hedl::wrapAsync($doc);

            $canonical = yield $asyncDoc->canonicalizeAsync();
            self::assert(strlen($canonical) > 0, "Expected non-empty canonical");
            self::assert(str_contains($canonical, '%VERSION'), "Expected %VERSION in canonical");

            $doc->close();
        });
    }

    public static function testAsyncDocumentLint(): Promise
    {
        return Amp\call(function() {
            $doc = Hedl::parse(self::$sampleHEDL);
            $asyncDoc = Hedl::wrapAsync($doc);

            $diag = yield $asyncDoc->lintAsync();
            self::assert($diag->count() >= 0, "Expected valid diagnostic count");
            $diag->close();
            $doc->close();
        });
    }

    public static function testWrapAsync(): Promise
    {
        return Amp\call(function() {
            $doc = Hedl::parse(self::$sampleHEDL);
            $asyncDoc = Hedl::wrapAsync($doc);

            self::assert($asyncDoc instanceof AsyncDocument, "Expected AsyncDocument instance");
            self::assert($asyncDoc->document() === $doc, "Expected wrapped document to match");

            $doc->close();
        });
    }

    public static function testPromiseComposition(): Promise
    {
        return Amp\call(function() {
            // Test parallel promise execution
            $promises = [
                Hedl::parseAsync(self::$sampleHEDL),
                Hedl::fromJsonAsync(self::$sampleJSON),
                Hedl::fromYamlAsync(self::$sampleYAML),
            ];

            $docs = yield Amp\Promise\all($promises);
            self::assert(count($docs) === 3, "Expected 3 documents");

            foreach ($docs as $doc) {
                self::assert($doc instanceof Document, "Expected Document in array");
                $doc->close();
            }
        });
    }

    public static function testAsyncErrorHandling(): Promise
    {
        return Amp\call(function() {
            $caught = false;
            try {
                yield Hedl::parseAsync(self::$fixtures->errorInvalidSyntax());
            } catch (HedlException $e) {
                $caught = true;
                self::assert($e->getHedlCode() !== 0, "Expected non-zero error code");
            }
            self::assert($caught, "Expected exception to be caught");
        });
    }
}

// Initialize and run tests
HedlAsyncTest::init();
HedlAsyncTest::run();
