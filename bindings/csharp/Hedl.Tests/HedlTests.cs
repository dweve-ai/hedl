// Tests for HEDL C# bindings

using System;
using Dweve.Hedl;
using Xunit;

namespace Dweve.Hedl.Tests
{
    public class HedlTests
    {
        // Use shared fixtures from common/fixtures directory
        private static readonly Fixtures _fixtures = new Fixtures();
        private static readonly string SampleHEDL = _fixtures.BasicHedl;
        private static readonly string SampleJSON = _fixtures.BasicJson;
        private static readonly string SampleYAML = _fixtures.BasicYaml;
        private static readonly string SampleXML = _fixtures.BasicXml;

        [Fact]
        public void Parse_ValidContent_ReturnsDocument()
        {
            using var doc = Hedl.Parse(SampleHEDL);
            Assert.NotNull(doc);
        }

        [Fact]
        public void Parse_InvalidContent_ThrowsException()
        {
            Assert.Throws<HedlException>(() => Hedl.Parse(_fixtures.ErrorInvalidSyntax));
        }

        [Fact]
        public void Validate_ValidContent_ReturnsTrue()
        {
            Assert.True(Hedl.Validate(SampleHEDL));
        }

        [Fact]
        public void Validate_InvalidContent_ReturnsFalse()
        {
            Assert.False(Hedl.Validate(_fixtures.ErrorInvalidSyntax));
        }

        [Fact]
        public void Document_Version_ReturnsCorrectVersion()
        {
            using var doc = Hedl.Parse(SampleHEDL);
            Assert.Equal((1, 0), doc.Version);
        }

        [Fact]
        public void Document_SchemaCount_ReturnsOne()
        {
            using var doc = Hedl.Parse(SampleHEDL);
            Assert.Equal(1, doc.SchemaCount);
        }

        [Fact]
        public void Document_Canonicalize_ReturnsNonEmpty()
        {
            using var doc = Hedl.Parse(SampleHEDL);
            var canonical = doc.Canonicalize();
            Assert.NotEmpty(canonical);
            Assert.Contains("%VERSION", canonical);
        }

        [Fact]
        public void Document_ToJson_ReturnsValidJson()
        {
            using var doc = Hedl.Parse(SampleHEDL);
            var json = doc.ToJson();
            Assert.NotEmpty(json);
            Assert.Contains("users", json);
        }

        [Fact]
        public void Document_ToYaml_ReturnsNonEmpty()
        {
            using var doc = Hedl.Parse(SampleHEDL);
            var yaml = doc.ToYaml();
            Assert.NotEmpty(yaml);
        }

        [Fact]
        public void Document_ToXml_ReturnsValidXml()
        {
            using var doc = Hedl.Parse(SampleHEDL);
            var xml = doc.ToXml();
            Assert.NotEmpty(xml);
            Assert.Contains("<", xml);
        }

        [Fact]
        public void Document_ToCsv_ReturnsNonEmpty()
        {
            using var doc = Hedl.Parse(SampleHEDL);
            var csv = doc.ToCsv();
            Assert.NotEmpty(csv);
        }

        [Fact]
        public void Document_ToCypher_ReturnsNonEmpty()
        {
            using var doc = Hedl.Parse(SampleHEDL);
            var cypher = doc.ToCypher();
            Assert.NotEmpty(cypher);
        }

        [Fact]
        public void Document_ToParquet_ReturnsBytes()
        {
            using var doc = Hedl.Parse(SampleHEDL);
            var data = doc.ToParquet();
            Assert.NotEmpty(data);
        }

        [Fact]
        public void FromJson_ValidJson_ReturnsDocument()
        {
            using var doc = Hedl.FromJson(SampleJSON);
            Assert.NotNull(doc);
            var canonical = doc.Canonicalize();
            Assert.NotEmpty(canonical);
        }

        [Fact]
        public void FromYaml_ValidYaml_ReturnsDocument()
        {
            using var doc = Hedl.FromYaml(SampleYAML);
            Assert.NotNull(doc);
        }

        [Fact]
        public void FromXml_ValidXml_ReturnsDocument()
        {
            using var doc = Hedl.FromXml(SampleXML);
            Assert.NotNull(doc);
        }

        [Fact]
        public void Document_Lint_ReturnsDiagnostics()
        {
            using var doc = Hedl.Parse(SampleHEDL);
            using var diag = doc.Lint();
            Assert.NotNull(diag);
            Assert.True(diag.Count >= 0);
        }

        [Fact]
        public void Document_DoubleDispose_DoesNotThrow()
        {
            var doc = Hedl.Parse(SampleHEDL);
            doc.Dispose();
            doc.Dispose(); // Should not throw
        }

        [Fact]
        public void Document_UseAfterDispose_ThrowsException()
        {
            var doc = Hedl.Parse(SampleHEDL);
            doc.Dispose();
            Assert.Throws<ObjectDisposedException>(() => doc.ToJson());
        }
    }
}
