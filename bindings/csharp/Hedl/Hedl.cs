// HEDL (Hierarchical Entity Data Language) C# Bindings
// Token-efficient data format optimized for LLM context windows.
//
// THREAD SAFETY WARNING:
// =====================
// These bindings are NOT thread-safe. Document and Diagnostics objects must
// not be accessed concurrently from multiple threads. The underlying FFI
// library does not perform any internal locking. If you need concurrent
// access to HEDL documents, you must:
//
// 1. Use separate Document instances per thread, OR
// 2. Implement your own synchronization (lock, Monitor, SemaphoreSlim) around
//    all Document and Diagnostics method calls
//
// Concurrent access without proper synchronization may result in:
// - Memory corruption
// - Use-after-free errors
// - Access violations
// - Undefined behavior
//
// Example of safe concurrent usage:
//
//   var lockObj = new object();
//   using var doc = Hedl.Parse(content);
//
//   Parallel.For(0, 10, i => {
//       lock(lockObj) {
//           var json = doc.ToJson();
//           Console.WriteLine(json);
//       }
//   });
//
// RESOURCE LIMITS:
// ===============
// The HEDL_MAX_OUTPUT_SIZE environment variable controls the maximum size of
// output from conversion operations (ToJson, ToYaml, ToXml, etc.).
//
// Default: 100 MB (conservative, may be too restrictive for many use cases)
// Recommended for data processing: 500 MB - 1 GB
// For large datasets: 1 GB - 5 GB
//
// Set before loading the assembly:
//
//   // In your shell or system environment
//   set HEDL_MAX_OUTPUT_SIZE=1073741824  # 1 GB (Windows)
//   export HEDL_MAX_OUTPUT_SIZE=1073741824  # 1 GB (Linux/macOS)
//
//   // Or in C# (must be set BEFORE using Hedl)
//   Environment.SetEnvironmentVariable("HEDL_MAX_OUTPUT_SIZE", "1073741824");  // 1 GB
//   using Dweve.Hedl;
//
// Use Cases:
// - Small configs: 10-50 MB (default may suffice)
// - Medium datasets: 100-500 MB (set to 524288000 for 500 MB)
// - Large datasets: 500 MB - 5 GB (set to 1073741824+ for 1 GB+)
// - No practical limit: set to a very high value like 10737418240 (10 GB)
//
// When the limit is exceeded, operations will throw HedlException with code
// HedlErrorCode.Alloc and a message suggesting to increase HEDL_MAX_OUTPUT_SIZE.

using System;
using System.Runtime.InteropServices;
using System.Text;

namespace Dweve.Hedl
{
    /// <summary>
    /// Resource limits for HEDL operations.
    /// Default is 100MB, which may be too restrictive for many real-world scenarios.
    /// Recommended: 500MB-1GB for data processing, higher for large datasets.
    /// Set HEDL_MAX_OUTPUT_SIZE environment variable before using to customize.
    /// </summary>
    internal static class ResourceLimits
    {
        public static readonly long MaxOutputSize;

        static ResourceLimits()
        {
            // Default: 100MB
            MaxOutputSize = 104857600;
            var env = Environment.GetEnvironmentVariable("HEDL_MAX_OUTPUT_SIZE");
            if (!string.IsNullOrEmpty(env) && long.TryParse(env, out var size))
            {
                MaxOutputSize = size;
            }
        }

        public static void CheckOutputSize(byte[] data)
        {
            if (data.Length > MaxOutputSize)
            {
                var actualMB = data.Length / 1048576.0;
                var limitMB = MaxOutputSize / 1048576.0;
                throw new HedlException(
                    $"Output size ({actualMB:F2}MB) exceeds limit ({limitMB:F2}MB). Set HEDL_MAX_OUTPUT_SIZE to increase.",
                    HedlErrorCode.Alloc
                );
            }
        }

        public static void CheckStringOutputSize(string str)
        {
            CheckOutputSize(Encoding.UTF8.GetBytes(str));
        }
    }

    /// <summary>
    /// Error codes returned by HEDL operations.
    /// </summary>
    public enum HedlErrorCode
    {
        Ok = 0,
        NullPtr = -1,
        InvalidUtf8 = -2,
        Parse = -3,
        Canonicalize = -4,
        Json = -5,
        Alloc = -6,
        Yaml = -7,
        Xml = -8,
        Csv = -9,
        Parquet = -10,
        Lint = -11,
        Neo4j = -12
    }

    /// <summary>
    /// Severity levels for diagnostic messages.
    /// </summary>
    public enum HedlSeverity
    {
        Hint = 0,
        Warning = 1,
        Error = 2
    }

    /// <summary>
    /// Exception thrown by HEDL operations.
    /// </summary>
    public class HedlException : Exception
    {
        public HedlErrorCode ErrorCode { get; }

        public HedlException(string message, HedlErrorCode code = HedlErrorCode.Parse)
            : base(message)
        {
            ErrorCode = code;
        }

        internal static HedlException FromLibrary(HedlErrorCode code)
        {
            var msg = NativeMethods.GetLastError() ?? $"HEDL error code {(int)code}";
            return new HedlException(msg, code);
        }
    }

    /// <summary>
    /// A diagnostic item from linting.
    /// </summary>
    public class DiagnosticItem
    {
        public string Message { get; }
        public HedlSeverity Severity { get; }

        public DiagnosticItem(string message, HedlSeverity severity)
        {
            Message = message;
            Severity = severity;
        }
    }

    /// <summary>
    /// Native P/Invoke methods for the HEDL library.
    /// </summary>
    internal static class NativeMethods
    {
        private const string LibraryName = "hedl_ffi";

        // Error handling
        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr hedl_get_last_error();

        // Memory management
        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern void hedl_free_string(IntPtr str);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern void hedl_free_document(IntPtr doc);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern void hedl_free_diagnostics(IntPtr diag);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern void hedl_free_bytes(IntPtr data, UIntPtr len);

        // Parsing
        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_parse(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string content,
            int len,
            int strict,
            out IntPtr doc);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_validate(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string content,
            int len,
            int strict);

        // Document info
        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_get_version(IntPtr doc, out int major, out int minor);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_schema_count(IntPtr doc);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_alias_count(IntPtr doc);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_root_item_count(IntPtr doc);

        // Canonicalization
        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_canonicalize(IntPtr doc, out IntPtr output);

        // JSON
        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_to_json(IntPtr doc, int includeMetadata, out IntPtr output);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_from_json(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string content,
            int len,
            out IntPtr doc);

        // YAML
        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_to_yaml(IntPtr doc, int includeMetadata, out IntPtr output);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_from_yaml(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string content,
            int len,
            out IntPtr doc);

        // XML
        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_to_xml(IntPtr doc, out IntPtr output);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_from_xml(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string content,
            int len,
            out IntPtr doc);

        // CSV
        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_to_csv(IntPtr doc, out IntPtr output);

        // Parquet
        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_to_parquet(IntPtr doc, out IntPtr data, out UIntPtr len);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_from_parquet(IntPtr data, UIntPtr len, out IntPtr doc);

        // Neo4j
        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_to_neo4j_cypher(IntPtr doc, int useMerge, out IntPtr output);

        // Linting
        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_lint(IntPtr doc, out IntPtr diag);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_diagnostics_count(IntPtr diag);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_diagnostics_get(IntPtr diag, int index, out IntPtr msg);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int hedl_diagnostics_severity(IntPtr diag, int index);

        internal static string? GetLastError()
        {
            var ptr = hedl_get_last_error();
            return ptr == IntPtr.Zero ? null : Marshal.PtrToStringUTF8(ptr);
        }

        internal static string ReadAndFreeString(IntPtr ptr)
        {
            if (ptr == IntPtr.Zero)
                return string.Empty;

            var str = Marshal.PtrToStringUTF8(ptr) ?? string.Empty;
            hedl_free_string(ptr);
            return str;
        }
    }

    /// <summary>
    /// Lint diagnostics container.
    /// </summary>
    public class Diagnostics : IDisposable
    {
        private IntPtr _ptr;
        private bool _disposed;

        internal Diagnostics(IntPtr ptr)
        {
            _ptr = ptr;
        }

        public int Count
        {
            get
            {
                ThrowIfDisposed();
                var c = NativeMethods.hedl_diagnostics_count(_ptr);
                return c < 0 ? 0 : c;
            }
        }

        public DiagnosticItem this[int index]
        {
            get
            {
                ThrowIfDisposed();
                if (index < 0 || index >= Count)
                    throw new ArgumentOutOfRangeException(nameof(index));

                var result = NativeMethods.hedl_diagnostics_get(_ptr, index, out var msgPtr);
                if (result != (int)HedlErrorCode.Ok)
                    throw HedlException.FromLibrary((HedlErrorCode)result);

                var message = NativeMethods.ReadAndFreeString(msgPtr);
                var severity = (HedlSeverity)NativeMethods.hedl_diagnostics_severity(_ptr, index);

                return new DiagnosticItem(message, severity);
            }
        }

        public DiagnosticItem[] GetAll()
        {
            var items = new DiagnosticItem[Count];
            for (int i = 0; i < items.Length; i++)
                items[i] = this[i];
            return items;
        }

        public string[] GetErrors()
        {
            return GetBySeverity(HedlSeverity.Error);
        }

        public string[] GetWarnings()
        {
            return GetBySeverity(HedlSeverity.Warning);
        }

        public string[] GetHints()
        {
            return GetBySeverity(HedlSeverity.Hint);
        }

        private string[] GetBySeverity(HedlSeverity severity)
        {
            var result = new System.Collections.Generic.List<string>();
            for (int i = 0; i < Count; i++)
            {
                var item = this[i];
                if (item.Severity == severity)
                    result.Add(item.Message);
            }
            return result.ToArray();
        }

        private void ThrowIfDisposed()
        {
            if (_disposed)
                throw new ObjectDisposedException(nameof(Diagnostics));
        }

        public void Dispose()
        {
            if (!_disposed && _ptr != IntPtr.Zero)
            {
                NativeMethods.hedl_free_diagnostics(_ptr);
                _ptr = IntPtr.Zero;
                _disposed = true;
            }
            GC.SuppressFinalize(this);
        }

        ~Diagnostics() => Dispose();
    }

    /// <summary>
    /// A parsed HEDL document.
    /// </summary>
    public class Document : IDisposable
    {
        private IntPtr _ptr;
        private bool _disposed;

        internal Document(IntPtr ptr)
        {
            _ptr = ptr;
        }

        /// <summary>
        /// Gets the HEDL version as a tuple (major, minor).
        /// </summary>
        public (int Major, int Minor) Version
        {
            get
            {
                ThrowIfDisposed();
                var result = NativeMethods.hedl_get_version(_ptr, out var major, out var minor);
                if (result != (int)HedlErrorCode.Ok)
                    throw HedlException.FromLibrary((HedlErrorCode)result);
                return (major, minor);
            }
        }

        /// <summary>
        /// Gets the number of schemas defined in the document.
        /// </summary>
        public int SchemaCount
        {
            get
            {
                ThrowIfDisposed();
                var c = NativeMethods.hedl_schema_count(_ptr);
                if (c < 0) throw HedlException.FromLibrary((HedlErrorCode)c);
                return c;
            }
        }

        /// <summary>
        /// Gets the number of aliases defined in the document.
        /// </summary>
        public int AliasCount
        {
            get
            {
                ThrowIfDisposed();
                var c = NativeMethods.hedl_alias_count(_ptr);
                if (c < 0) throw HedlException.FromLibrary((HedlErrorCode)c);
                return c;
            }
        }

        /// <summary>
        /// Gets the number of root items in the document.
        /// </summary>
        public int RootItemCount
        {
            get
            {
                ThrowIfDisposed();
                var c = NativeMethods.hedl_root_item_count(_ptr);
                if (c < 0) throw HedlException.FromLibrary((HedlErrorCode)c);
                return c;
            }
        }

        /// <summary>
        /// Converts the document to canonical HEDL format.
        /// </summary>
        public string Canonicalize()
        {
            ThrowIfDisposed();
            var result = NativeMethods.hedl_canonicalize(_ptr, out var output);
            if (result != (int)HedlErrorCode.Ok)
                throw HedlException.FromLibrary((HedlErrorCode)result);
            var str = NativeMethods.ReadAndFreeString(output);
            ResourceLimits.CheckStringOutputSize(str);
            return str;
        }

        /// <summary>
        /// Converts the document to JSON.
        /// </summary>
        public string ToJson(bool includeMetadata = false)
        {
            ThrowIfDisposed();
            var result = NativeMethods.hedl_to_json(_ptr, includeMetadata ? 1 : 0, out var output);
            if (result != (int)HedlErrorCode.Ok)
                throw HedlException.FromLibrary((HedlErrorCode)result);
            var str = NativeMethods.ReadAndFreeString(output);
            ResourceLimits.CheckStringOutputSize(str);
            return str;
        }

        /// <summary>
        /// Converts the document to YAML.
        /// </summary>
        public string ToYaml(bool includeMetadata = false)
        {
            ThrowIfDisposed();
            var result = NativeMethods.hedl_to_yaml(_ptr, includeMetadata ? 1 : 0, out var output);
            if (result != (int)HedlErrorCode.Ok)
                throw HedlException.FromLibrary((HedlErrorCode)result);
            var str = NativeMethods.ReadAndFreeString(output);
            ResourceLimits.CheckStringOutputSize(str);
            return str;
        }

        /// <summary>
        /// Converts the document to XML.
        /// </summary>
        public string ToXml()
        {
            ThrowIfDisposed();
            var result = NativeMethods.hedl_to_xml(_ptr, out var output);
            if (result != (int)HedlErrorCode.Ok)
                throw HedlException.FromLibrary((HedlErrorCode)result);
            var str = NativeMethods.ReadAndFreeString(output);
            ResourceLimits.CheckStringOutputSize(str);
            return str;
        }

        /// <summary>
        /// Converts the document to CSV.
        /// </summary>
        public string ToCsv()
        {
            ThrowIfDisposed();
            var result = NativeMethods.hedl_to_csv(_ptr, out var output);
            if (result != (int)HedlErrorCode.Ok)
                throw HedlException.FromLibrary((HedlErrorCode)result);
            var str = NativeMethods.ReadAndFreeString(output);
            ResourceLimits.CheckStringOutputSize(str);
            return str;
        }

        /// <summary>
        /// Converts the document to Parquet bytes.
        /// </summary>
        public byte[] ToParquet()
        {
            ThrowIfDisposed();
            var result = NativeMethods.hedl_to_parquet(_ptr, out var dataPtr, out var len);
            if (result != (int)HedlErrorCode.Ok)
                throw HedlException.FromLibrary((HedlErrorCode)result);

            var data = new byte[(int)len];
            Marshal.Copy(dataPtr, data, 0, (int)len);
            NativeMethods.hedl_free_bytes(dataPtr, len);
            ResourceLimits.CheckOutputSize(data);
            return data;
        }

        /// <summary>
        /// Converts the document to Neo4j Cypher statements.
        /// </summary>
        public string ToCypher(bool useMerge = true)
        {
            ThrowIfDisposed();
            var result = NativeMethods.hedl_to_neo4j_cypher(_ptr, useMerge ? 1 : 0, out var output);
            if (result != (int)HedlErrorCode.Ok)
                throw HedlException.FromLibrary((HedlErrorCode)result);
            var str = NativeMethods.ReadAndFreeString(output);
            ResourceLimits.CheckStringOutputSize(str);
            return str;
        }

        /// <summary>
        /// Runs linting on the document.
        /// </summary>
        public Diagnostics Lint()
        {
            ThrowIfDisposed();
            var result = NativeMethods.hedl_lint(_ptr, out var diagPtr);
            if (result != (int)HedlErrorCode.Ok)
                throw HedlException.FromLibrary((HedlErrorCode)result);
            return new Diagnostics(diagPtr);
        }

        private void ThrowIfDisposed()
        {
            if (_disposed)
                throw new ObjectDisposedException(nameof(Document));
        }

        public void Dispose()
        {
            if (!_disposed && _ptr != IntPtr.Zero)
            {
                NativeMethods.hedl_free_document(_ptr);
                _ptr = IntPtr.Zero;
                _disposed = true;
            }
            GC.SuppressFinalize(this);
        }

        ~Document() => Dispose();
    }

    /// <summary>
    /// Main HEDL class with static parsing and conversion methods.
    /// </summary>
    public static class Hedl
    {
        /// <summary>
        /// Parses HEDL content into a Document.
        /// </summary>
        public static Document Parse(string content, bool strict = true)
        {
            var result = NativeMethods.hedl_parse(content, Encoding.UTF8.GetByteCount(content), strict ? 1 : 0, out var docPtr);
            if (result != (int)HedlErrorCode.Ok)
                throw HedlException.FromLibrary((HedlErrorCode)result);
            return new Document(docPtr);
        }

        /// <summary>
        /// Validates HEDL content without creating a document.
        /// </summary>
        public static bool Validate(string content, bool strict = true)
        {
            var result = NativeMethods.hedl_validate(content, Encoding.UTF8.GetByteCount(content), strict ? 1 : 0);
            return result == (int)HedlErrorCode.Ok;
        }

        /// <summary>
        /// Parses JSON content into a HEDL Document.
        /// </summary>
        public static Document FromJson(string content)
        {
            var result = NativeMethods.hedl_from_json(content, Encoding.UTF8.GetByteCount(content), out var docPtr);
            if (result != (int)HedlErrorCode.Ok)
                throw HedlException.FromLibrary((HedlErrorCode)result);
            return new Document(docPtr);
        }

        /// <summary>
        /// Parses YAML content into a HEDL Document.
        /// </summary>
        public static Document FromYaml(string content)
        {
            var result = NativeMethods.hedl_from_yaml(content, Encoding.UTF8.GetByteCount(content), out var docPtr);
            if (result != (int)HedlErrorCode.Ok)
                throw HedlException.FromLibrary((HedlErrorCode)result);
            return new Document(docPtr);
        }

        /// <summary>
        /// Parses XML content into a HEDL Document.
        /// </summary>
        public static Document FromXml(string content)
        {
            var result = NativeMethods.hedl_from_xml(content, Encoding.UTF8.GetByteCount(content), out var docPtr);
            if (result != (int)HedlErrorCode.Ok)
                throw HedlException.FromLibrary((HedlErrorCode)result);
            return new Document(docPtr);
        }

        /// <summary>
        /// Parses Parquet data into a HEDL Document.
        /// </summary>
        public static Document FromParquet(byte[] data)
        {
            var handle = GCHandle.Alloc(data, GCHandleType.Pinned);
            try
            {
                var result = NativeMethods.hedl_from_parquet(handle.AddrOfPinnedObject(), (UIntPtr)data.Length, out var docPtr);
                if (result != (int)HedlErrorCode.Ok)
                    throw HedlException.FromLibrary((HedlErrorCode)result);
                return new Document(docPtr);
            }
            finally
            {
                handle.Free();
            }
        }
    }
}
