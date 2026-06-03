using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using Microsoft.Win32;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.ComTypes;
using System.Text;

internal static class Program
{
    private const int S_OK = 0;
    private const int E_NOTIMPL = unchecked((int)0x80004001);
    internal const int E_NOTIMPL_FOR_COM = unchecked((int)0x80004001);
    private const int OLECLOSE_SAVEIFDIRTY = 0;
    private const int CF_METAFILEPICT = 3;
    private const int DVASPECT_CONTENT = 1;
    private const int MM_LOMETRIC = 2;
    private const int MM_HIMETRIC = 3;
    private const int MM_LOENGLISH = 4;
    private const int MM_HIENGLISH = 5;
    private const int MM_TWIPS = 6;
    private const int MM_ISOTROPIC = 7;
    private const int MM_ANISOTROPIC = 8;
    private const int TYMED_HGLOBAL = 1;
    private const int TYMED_MFPICT = 32;
    private const int STGM_CREATE = 0x00001000;
    private const int STGM_READWRITE = 0x00000002;
    private const int STGM_SHARE_EXCLUSIVE = 0x00000010;
    private const short MTINIT_LAUNCH_NOW = 1;
    private const short MTPRF_MODE_NEXT_EQN = 1;
    private const int MTDIM_WIDTH = 1;
    private const int MTDIM_HEIGHT = 2;
    private const int MTDIM_BASELINE = 3;
    private const int MTDIM_HORIZ_POS_TYPE = 4;
    private const int MTDIM_HORIZ_POS = 5;
    private const double MATH_TYPE_DIMENSION_UNITS_PER_POINT = 32.0;
    private const short MT_OK = 0;
    private const string MathTypeProgId = "Equation.DSMT4";

    private static readonly Guid IidIOleObject = new Guid("00000112-0000-0000-C000-000000000046");
    private static bool mathTypeApiDllResolved;
    private static string? mathTypeApiDllPath;

    [STAThread]
    public static int Main(string[] args)
    {
        try
        {
            var options = Options.Parse(args);
            OleCheck(OleInitialize(IntPtr.Zero), "OleInitialize");
            try
            {
                CreateOleBin(options);
            }
            finally
            {
                OleUninitialize();
            }
            return 0;
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine("[ole-helper] " + ex.Message);
            if (VerboseLoggingEnabled())
            {
                Console.Error.WriteLine(ex);
            }
            return 1;
        }
    }

    private static void CreateOleBin(Options options)
    {
        Log("resolve CLSID");
        var clsid = Guid.Empty;
        OleCheck(CLSIDFromProgID(MathTypeProgId, out clsid), $"CLSIDFromProgID({MathTypeProgId})");

        if (options.PrefsFilePath is not null)
        {
            Log($"ApplyMathTypePrefs({options.PrefsFilePath})");
            ApplyMathTypePrefs(options.PrefsFilePath);
        }

        Log("create storage");
        Directory.CreateDirectory(Path.GetDirectoryName(Path.GetFullPath(options.OutputPath))!);
        if (File.Exists(options.OutputPath))
        {
            File.Delete(options.OutputPath);
        }

        OleCheck(
            StgCreateDocfile(
                options.OutputPath,
                STGM_CREATE | STGM_READWRITE | STGM_SHARE_EXCLUSIVE,
                0,
                out var storage),
            "StgCreateDocfile");

        var iidIOleObject = IidIOleObject;

        var site = new MinimalOleClientSite();
        object created;
        if (options.Method.Equals("create-from-data", StringComparison.OrdinalIgnoreCase))
        {
            Log("OleCreateFromData");
            var payload = BuildPayload(options);
            var dataObject = new SingleFormatDataObject(payload.FormatId, payload.Bytes);
            OleCheck(
                OleCreateFromData(
                    dataObject,
                    ref iidIOleObject,
                    0,
                    IntPtr.Zero,
                    site,
                    storage,
                    out created),
                "OleCreateFromData");
        }
        else
        {
            Log("OleCreate");
            OleCheck(
                OleCreate(
                    ref clsid,
                    ref iidIOleObject,
                    0,
                    IntPtr.Zero,
                    site,
                    storage,
                    out created),
                "OleCreate(Equation.DSMT4)");
        }

        var oleObject = (IOleObject)created;
        try
        {
            Log("SetHostNames");
            oleObject.SetHostNames("Pandoc Manuscript Probe", "MathType equation");
            if (options.Method.Equals("set-data", StringComparison.OrdinalIgnoreCase))
            {
                if (options.PreVerb is not null)
                {
                    Log($"pre DoVerb({options.PreVerb.Value})");
                    var preRect = new RECT { left = 0, top = 0, right = 1600, bottom = 600 };
                    oleObject.DoVerb(options.PreVerb.Value, IntPtr.Zero, site, 0, IntPtr.Zero, ref preRect);
                }
                else
                {
                    Log("OleRun");
                    OleCheck(OleRun(created), "OleRun");
                }
                Log("SetEquationData");
                SetEquationData(created, options);
            }
            else if (options.Method.Equals("init-from-data", StringComparison.OrdinalIgnoreCase))
            {
                Log("InitFromData");
                var payload = BuildPayload(options);
                var dataObject = new SingleFormatDataObject(payload.FormatId, payload.Bytes);
                oleObject.InitFromData(dataObject, true, 0);
            }

            // MathType registers verb 2 as RunForConversion. It asks the OLE server
            // to consume the custom data format and materialize a normal equation.
            if (options.DoVerb && !options.Method.Equals("create-from-data", StringComparison.OrdinalIgnoreCase))
            {
                Log("DoVerb(2)");
                var rect = new RECT { left = 0, top = 0, right = 1600, bottom = 600 };
                oleObject.DoVerb(2, IntPtr.Zero, site, 0, IntPtr.Zero, ref rect);
            }

            Log("OleSave");
            var persist = (IPersistStorage)created;
            OleCheck(WriteClassStg(storage, ref clsid), "WriteClassStg");
            OleCheck(OleSave(persist, storage, true), "OleSave");
            OleCheck(storage.Commit(0), "IStorage.Commit");
            if (options.PreviewOutputPath is not null)
            {
                Log("Write WMF preview");
                var preview = WriteWmfPreview(created, options.PreviewOutputPath);
                if (options.MetadataOutputPath is not null)
                {
                    Log("Write metadata");
                    WriteMetadata(options.MetadataOutputPath, preview);
                }
            }
            oleObject.Close(OLECLOSE_SAVEIFDIRTY);
        }
        finally
        {
            try
            {
                oleObject.Close(OLECLOSE_SAVEIFDIRTY);
            }
            catch
            {
                // Best-effort cleanup for failed probes; MathType can keep an
                // OLE server process alive after conversion errors.
            }
            Marshal.ReleaseComObject(created);
            Marshal.ReleaseComObject(storage);
        }

        Log($"wrote {options.OutputPath}, bytes={new FileInfo(options.OutputPath).Length}");
    }

    private static void ApplyMathTypePrefs(string prefsFilePath)
    {
        // Applying per-equation size prefs requires MT6.dll. Resolve it lazily
        // so non-default MathType installs do not depend on one hard-coded path.
        TryConfigureMathTypeApiDll(required: true);

        var fullPrefsPath = Path.GetFullPath(prefsFilePath);
        if (!File.Exists(fullPrefsPath))
        {
            throw new FileNotFoundException($"MathType preferences file was not found: {fullPrefsPath}", fullPrefsPath);
        }

        MtCheck(MTAPIConnect(MTINIT_LAUNCH_NOW, 30), "MTAPIConnect(prefs)");
        try
        {
            var prefLength = MTGetPrefsFromFile(fullPrefsPath, null, 0);
            if (prefLength <= 0)
            {
                throw new InvalidOperationException($"MTGetPrefsFromFile returned invalid length {prefLength} for {fullPrefsPath}");
            }

            var prefs = new StringBuilder(prefLength);
            MtCheck(MTGetPrefsFromFile(fullPrefsPath, prefs, checked((short)prefLength)), "MTGetPrefsFromFile");
            MtCheck(MTSetMTPrefs(MTPRF_MODE_NEXT_EQN, prefs.ToString(), -1), "MTSetMTPrefs");
        }
        finally
        {
            MTAPIDisconnect();
        }
    }

    private static void SetEquationData(object created, Options options)
    {
        Log("cast IDataObject");
        var dataObject = (System.Runtime.InteropServices.ComTypes.IDataObject)created;
        var payload = BuildPayload(options);

        var format = new FORMATETC
        {
            cfFormat = unchecked((short)payload.FormatId),
            dwAspect = (DVASPECT)DVASPECT_CONTENT,
            lindex = -1,
            ptd = IntPtr.Zero,
            tymed = (TYMED)TYMED_HGLOBAL,
        };
        var medium = new STGMEDIUM
        {
            tymed = (TYMED)TYMED_HGLOBAL,
            unionmember = CopyToHGlobal(payload.Bytes),
            pUnkForRelease = null,
        };

        try
        {
            Log($"IDataObject.SetData({options.Format}, {payload.Bytes.Length} bytes)");
            // Keep HGLOBAL ownership in this helper. Some legacy OLE servers are
            // sensitive to fRelease ownership for custom clipboard formats.
            dataObject.SetData(ref format, ref medium, false);
        }
        finally
        {
            if (medium.unionmember != IntPtr.Zero)
            {
                GlobalFree(medium.unionmember);
            }
        }

        Log($"SetData({options.Format}) length={payload.Bytes.Length}");
    }

    private static Payload BuildPayload(Options options)
    {
        var formatId = RegisterClipboardFormat(options.Format);
        if (formatId == 0)
        {
            throw new InvalidOperationException($"RegisterClipboardFormat failed for {options.Format}");
        }

        var bytes = options.BinaryInput
            ? File.ReadAllBytes(options.InputPath)
            : EncodeText(File.ReadAllText(options.InputPath, Encoding.UTF8), options.EncodingName);
        return new Payload(formatId, bytes);
    }

    private static PreviewMetadata WriteWmfPreview(object created, string outputPath)
    {
        var dataObject = (System.Runtime.InteropServices.ComTypes.IDataObject)created;
        var format = new FORMATETC
        {
            cfFormat = CF_METAFILEPICT,
            dwAspect = (DVASPECT)DVASPECT_CONTENT,
            lindex = -1,
            ptd = IntPtr.Zero,
            tymed = (TYMED)TYMED_MFPICT,
        };
        dataObject.GetData(ref format, out var medium);
        try
        {
            if (medium.unionmember == IntPtr.Zero)
            {
                throw new InvalidOperationException("CF_METAFILEPICT returned an empty handle");
            }

            var pictPtr = GlobalLock(medium.unionmember);
            if (pictPtr == IntPtr.Zero)
            {
                throw new InvalidOperationException("GlobalLock failed for METAFILEPICT");
            }

            int mapMode;
            int xExt;
            int yExt;
            IntPtr hMetaFile;
            try
            {
                mapMode = Marshal.ReadInt32(pictPtr, 0);
                xExt = Marshal.ReadInt32(pictPtr, 4);
                yExt = Marshal.ReadInt32(pictPtr, 8);
                hMetaFile = Marshal.ReadIntPtr(pictPtr, IntPtr.Size == 8 ? 16 : 12);
            }
            finally
            {
                GlobalUnlock(medium.unionmember);
            }

            var bytes = ReadMetaFileBits(hMetaFile);
            var unitsPerInch = UnitsPerInchForMapMode(mapMode);
            if (!HasPlaceableHeader(bytes))
            {
                bytes = AddPlaceableHeader(bytes, mapMode, xExt, yExt);
            }

            Directory.CreateDirectory(Path.GetDirectoryName(Path.GetFullPath(outputPath))!);
            File.WriteAllBytes(outputPath, bytes);
            Log($"wrote preview {outputPath}, bytes={bytes.Length}");
            return new PreviewMetadata(
                mapMode,
                xExt,
                yExt,
                unitsPerInch,
                Math.Abs(xExt) * 72.0 / unitsPerInch,
                Math.Abs(yExt) * 72.0 / unitsPerInch,
                TryReadMathTypeLastDimensions());
        }
        finally
        {
            ReleaseStgMedium(ref medium);
        }
    }

    private static void WriteMetadata(string outputPath, PreviewMetadata preview)
    {
        // The metadata is intentionally tiny JSON so Python can drive Word XML
        // placement without depending on MathType COM at DOCX injection time.
        Directory.CreateDirectory(Path.GetDirectoryName(Path.GetFullPath(outputPath))!);
        File.WriteAllText(outputPath, preview.ToJson(), new UTF8Encoding(encoderShouldEmitUTF8Identifier: false));
        Log($"wrote metadata {outputPath}");
    }

    private static MathTypeLastDimensions? TryReadMathTypeLastDimensions()
    {
        try
        {
            // MTGetLastDimension reports 1/32-point units for the most recent
            // MathType-rendered equation. If MathType has no fresh value, keep
            // this optional so the DOCX layer can fall back to preview metrics.
            if (!TryConfigureMathTypeApiDll(required: false))
            {
                return null;
            }
            MtCheck(MTAPIConnect(MTINIT_LAUNCH_NOW, 30), "MTAPIConnect(dimensions)");
            try
            {
                return new MathTypeLastDimensions(
                    MTGetLastDimension(MTDIM_WIDTH),
                    MTGetLastDimension(MTDIM_HEIGHT),
                    MTGetLastDimension(MTDIM_BASELINE),
                    MTGetLastDimension(MTDIM_HORIZ_POS_TYPE),
                    MTGetLastDimension(MTDIM_HORIZ_POS));
            }
            finally
            {
                MTAPIDisconnect();
            }
        }
        catch (Exception ex)
        {
            Log($"dimension read failed: {ex.Message}");
            return null;
        }
    }

    private static bool TryConfigureMathTypeApiDll(bool required)
    {
        // MT6.dll is an auxiliary MathType API DLL, not the OLE server itself.
        // Detect it from the registered OLE server first to support custom installs.
        if (mathTypeApiDllResolved)
        {
            if (mathTypeApiDllPath is not null)
            {
                return true;
            }

            if (required)
            {
                throw new FileNotFoundException("Could not find MathType MT6.dll from the registered OLE server or common install folders.");
            }
            return false;
        }

        mathTypeApiDllResolved = true;
        mathTypeApiDllPath = ResolveMathTypeApiDllPath();
        if (mathTypeApiDllPath is null)
        {
            Log("MT6.dll not found; MathType dimension metadata will use preview metrics only");
            if (required)
            {
                throw new FileNotFoundException("Could not find MathType MT6.dll from the registered OLE server or common install folders.");
            }
            return false;
        }

        var directory = Path.GetDirectoryName(mathTypeApiDllPath)!;
        if (!SetDllDirectory(directory))
        {
            throw new InvalidOperationException(
                $"SetDllDirectory failed for {directory}: 0x{Marshal.GetLastWin32Error():X8}");
        }
        Log($"MT6.dll resolved: {mathTypeApiDllPath}");
        return true;
    }

    private static string? ResolveMathTypeApiDllPath()
    {
        // Return the first MT6.dll path found from MathType registry and common roots.
        foreach (var root in CandidateMathTypeRoots().Where(root => !string.IsNullOrWhiteSpace(root)).Distinct(StringComparer.OrdinalIgnoreCase))
        {
            foreach (var relativePath in new[] { Path.Combine("System", "64", "MT6.dll"), Path.Combine("System", "32", "MT6.dll"), "MT6.dll" })
            {
                var candidate = Path.Combine(root, relativePath);
                if (File.Exists(candidate))
                {
                    return candidate;
                }
            }
        }
        return null;
    }

    private static IEnumerable<string> CandidateMathTypeRoots()
    {
        // Yield likely MathType install roots, with registry-derived paths first.
        var oleServerPath = ResolveMathTypeOleServerPath();
        if (oleServerPath is not null)
        {
            var serverDirectory = Path.GetDirectoryName(oleServerPath);
            if (!string.IsNullOrEmpty(serverDirectory))
            {
                yield return serverDirectory;
                var directoryName = Path.GetFileName(serverDirectory);
                var parent = Directory.GetParent(serverDirectory);
                if (parent is not null && string.Equals(directoryName, "System", StringComparison.OrdinalIgnoreCase))
                {
                    yield return parent.FullName;
                }
                if (parent is not null && (string.Equals(directoryName, "64", StringComparison.OrdinalIgnoreCase) || string.Equals(directoryName, "32", StringComparison.OrdinalIgnoreCase)))
                {
                    var grandparent = parent.Parent;
                    if (grandparent is not null && string.Equals(parent.Name, "System", StringComparison.OrdinalIgnoreCase))
                    {
                        yield return grandparent.FullName;
                    }
                }
            }
        }

        foreach (var folder in new[]
        {
            Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86),
            Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles),
            Environment.GetEnvironmentVariable("ProgramW6432"),
        })
        {
            if (!string.IsNullOrWhiteSpace(folder))
            {
                yield return Path.Combine(folder, "MathType");
            }
        }
    }

    private static string? ResolveMathTypeOleServerPath()
    {
        // Return the registered MathType OLE server executable path when available.
        var clsid = ReadClassesRootDefault($@"{MathTypeProgId}\CLSID");
        if (string.IsNullOrWhiteSpace(clsid))
        {
            return null;
        }

        var command = ReadClassesRootDefault($@"CLSID\{clsid}\LocalServer32")
            ?? ReadClassesRootDefault($@"CLSID\{clsid}\LocalServer");
        return command is null ? null : ParseRegistryExecutablePath(command);
    }

    private static string? ReadClassesRootDefault(string subkey)
    {
        // Read a default HKCR value, trying both registry views for custom installs.
        foreach (var view in new[] { RegistryView.Registry64, RegistryView.Registry32 })
        {
            try
            {
                using var root = RegistryKey.OpenBaseKey(RegistryHive.ClassesRoot, view);
                using var key = root.OpenSubKey(subkey);
                if (key?.GetValue(null) is string value && !string.IsNullOrWhiteSpace(value))
                {
                    return value.Trim();
                }
            }
            catch (IOException)
            {
            }
            catch (UnauthorizedAccessException)
            {
            }
        }
        return null;
    }

    private static string? ParseRegistryExecutablePath(string command)
    {
        // Extract the executable path from a quoted or unquoted registry command.
        var text = command.Trim();
        if (text.StartsWith("\"", StringComparison.Ordinal))
        {
            var endQuote = text.IndexOf('"', 1);
            if (endQuote > 1)
            {
                return text.Substring(1, endQuote - 1);
            }
        }

        var exeIndex = text.IndexOf(".exe", StringComparison.OrdinalIgnoreCase);
        return exeIndex < 0 ? null : text.Substring(0, exeIndex + 4).Trim();
    }

    private static byte[] ReadMetaFileBits(IntPtr hMetaFile)
    {
        var size = GetMetaFileBitsEx(hMetaFile, 0, IntPtr.Zero);
        if (size == 0)
        {
            throw new InvalidOperationException("GetMetaFileBitsEx returned zero bytes");
        }
        var buffer = Marshal.AllocHGlobal((int)size);
        try
        {
            var written = GetMetaFileBitsEx(hMetaFile, size, buffer);
            if (written != size)
            {
                throw new InvalidOperationException($"GetMetaFileBitsEx wrote {written} of {size} bytes");
            }
            var bytes = new byte[size];
            Marshal.Copy(buffer, bytes, 0, (int)size);
            return bytes;
        }
        finally
        {
            Marshal.FreeHGlobal(buffer);
        }
    }

    private static bool HasPlaceableHeader(byte[] bytes)
    {
        return bytes.Length >= 4 && bytes[0] == 0xD7 && bytes[1] == 0xCD && bytes[2] == 0xC6 && bytes[3] == 0x9A;
    }

    private static ushort UnitsPerInchForMapMode(int mapMode)
    {
        // METAFILEPICT extents carry their own map-mode units. Treat isotropic
        // and anisotropic OLE previews as HIMETRIC; using twips here made
        // MathType previews about 1.76x too large in Word.
        return mapMode switch
        {
            MM_LOMETRIC => 254,
            MM_HIMETRIC => 2540,
            MM_LOENGLISH => 100,
            MM_HIENGLISH => 1000,
            MM_TWIPS => 1440,
            MM_ISOTROPIC => 2540,
            MM_ANISOTROPIC => 2540,
            _ => 2540,
        };
    }

    private static byte[] AddPlaceableHeader(byte[] wmfBytes, int mapMode, int xExt, int yExt)
    {
        var right = ClampPositiveInt16(xExt);
        var bottom = ClampPositiveInt16(yExt);
        var unitsPerInch = UnitsPerInchForMapMode(mapMode);
        Log($"METAFILEPICT mapMode={mapMode}, xExt={xExt}, yExt={yExt}, unitsPerInch={unitsPerInch}");
        var header = new byte[22];
        using var stream = new MemoryStream(header);
        using var writer = new BinaryWriter(stream);
        writer.Write(0x9AC6CDD7u);
        writer.Write((ushort)0);
        writer.Write((short)0);
        writer.Write((short)0);
        writer.Write(right);
        writer.Write(bottom);
        writer.Write(unitsPerInch);
        writer.Write(0u);
        writer.Write((ushort)0);
        var checksum = ComputePlaceableChecksum(header);
        BitConverter.GetBytes(checksum).CopyTo(header, 20);
        return header.Concat(wmfBytes).ToArray();
    }

    private static short ClampPositiveInt16(int value)
    {
        // .NET Framework does not provide Math.Clamp. Keep the placeable WMF
        // bounds valid while preserving the previous clamping behavior.
        var magnitude = Math.Abs((long)value);
        if (magnitude < 1)
        {
            return 1;
        }
        if (magnitude > short.MaxValue)
        {
            return short.MaxValue;
        }
        return (short)magnitude;
    }

    private static ushort ComputePlaceableChecksum(byte[] header)
    {
        ushort checksum = 0;
        for (var offset = 0; offset < 20; offset += 2)
        {
            checksum ^= BitConverter.ToUInt16(header, offset);
        }
        return checksum;
    }

    private static byte[] EncodeText(string text, string encodingName)
    {
        var encoding = encodingName.Equals("utf16le", StringComparison.OrdinalIgnoreCase)
            ? Encoding.Unicode
            : Encoding.UTF8;
        var payload = encoding.GetBytes(text);
        var terminator = encodingName.Equals("utf16le", StringComparison.OrdinalIgnoreCase)
            ? new byte[] { 0, 0 }
            : new byte[] { 0 };
        return payload.Concat(terminator).ToArray();
    }

    private static IntPtr CopyToHGlobal(byte[] bytes)
    {
        var handle = GlobalAlloc(0x0042, (UIntPtr)bytes.Length);
        if (handle == IntPtr.Zero)
        {
            throw new OutOfMemoryException("GlobalAlloc failed");
        }

        var locked = GlobalLock(handle);
        if (locked == IntPtr.Zero)
        {
            GlobalFree(handle);
            throw new InvalidOperationException("GlobalLock failed");
        }
        try
        {
            Marshal.Copy(bytes, 0, locked, bytes.Length);
        }
        finally
        {
            GlobalUnlock(handle);
        }

        return handle;
    }

    private static void OleCheck(int hr, string call)
    {
        if (hr < 0)
        {
            Marshal.ThrowExceptionForHR(hr);
        }
        if (hr != S_OK)
        {
            Log($"{call} returned 0x{hr:X8}");
        }
    }

    private static void Log(string message)
    {
        // Keep normal conversion output quiet; enable this only when debugging
        // noisy OLE/COM transitions on a local MathType installation.
        if (!VerboseLoggingEnabled())
        {
            return;
        }
        Console.Error.WriteLine("[ole-helper] " + message);
        Console.Error.Flush();
    }

    private static bool VerboseLoggingEnabled()
    {
        // Any non-empty, non-zero value restores the old diagnostic stream.
        var value = Environment.GetEnvironmentVariable("MATHTYPE_OLE_HELPER_VERBOSE");
        return !string.IsNullOrWhiteSpace(value) && value != "0";
    }

    private static void MtCheck(int status, string call)
    {
        if (status != MT_OK)
        {
            throw new InvalidOperationException($"{call} returned MathType status {status}");
        }
    }

    [DllImport("ole32.dll")]
    private static extern int OleInitialize(IntPtr pvReserved);

    [DllImport("ole32.dll")]
    private static extern void OleUninitialize();

    [DllImport("ole32.dll", CharSet = CharSet.Unicode)]
    private static extern int CLSIDFromProgID(string lpszProgID, out Guid pclsid);

    [DllImport("ole32.dll", CharSet = CharSet.Unicode)]
    private static extern int StgCreateDocfile(
        string pwcsName,
        int grfMode,
        int reserved,
        out IStorage ppstgOpen);

    [DllImport("ole32.dll")]
    private static extern int OleCreate(
        ref Guid rclsid,
        ref Guid riid,
        uint renderopt,
        IntPtr pFormatEtc,
        IOleClientSite? pClientSite,
        IStorage pStg,
        [MarshalAs(UnmanagedType.Interface)] out object ppvObj);

    [DllImport("ole32.dll")]
    private static extern int OleCreateFromData(
        [MarshalAs(UnmanagedType.Interface)] System.Runtime.InteropServices.ComTypes.IDataObject pSrcDataObj,
        ref Guid riid,
        uint renderopt,
        IntPtr pFormatEtc,
        IOleClientSite? pClientSite,
        IStorage pStg,
        [MarshalAs(UnmanagedType.Interface)] out object ppvObj);

    [DllImport("ole32.dll")]
    private static extern int OleSave(IPersistStorage pPS, IStorage pStg, bool fSameAsLoad);

    [DllImport("ole32.dll")]
    private static extern int OleRun([MarshalAs(UnmanagedType.IUnknown)] object pUnknown);

    [DllImport("ole32.dll")]
    private static extern void ReleaseStgMedium(ref STGMEDIUM pmedium);

    [DllImport("gdi32.dll")]
    private static extern uint GetMetaFileBitsEx(IntPtr hmf, uint cbBuffer, IntPtr lpData);

    [DllImport("ole32.dll")]
    private static extern int WriteClassStg(IStorage pStg, ref Guid rclsid);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern ushort RegisterClipboardFormat(string lpszFormat);

    [DllImport("MT6.dll", CharSet = CharSet.Ansi)]
    private static extern int MTAPIConnect(short options, short timeout);

    [DllImport("MT6.dll", CharSet = CharSet.Ansi)]
    private static extern int MTAPIDisconnect();

    [DllImport("MT6.dll", CharSet = CharSet.Ansi)]
    private static extern int MTGetPrefsFromFile(
        [MarshalAs(UnmanagedType.LPStr)] string prefFile,
        [MarshalAs(UnmanagedType.LPStr)] StringBuilder? prefs,
        short prefsLen);

    [DllImport("MT6.dll", CharSet = CharSet.Ansi)]
    private static extern int MTSetMTPrefs(
        short mode,
        [MarshalAs(UnmanagedType.LPStr)] string prefs,
        short timeout);

    [DllImport("MT6.dll", CharSet = CharSet.Ansi)]
    private static extern int MTGetLastDimension(int dimIndex);

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern bool SetDllDirectory(string lpPathName);

    [DllImport("kernel32.dll")]
    private static extern IntPtr GlobalAlloc(uint uFlags, UIntPtr dwBytes);

    [DllImport("kernel32.dll")]
    private static extern IntPtr GlobalLock(IntPtr hMem);

    [DllImport("kernel32.dll")]
    private static extern bool GlobalUnlock(IntPtr hMem);

    [DllImport("kernel32.dll")]
    private static extern IntPtr GlobalFree(IntPtr hMem);

    private sealed class Options
    {
        public Options(
            string format,
            string inputPath,
            string outputPath,
            string encodingName,
            bool binaryInput,
            bool doVerb,
            string method,
            int? preVerb,
            string? prefsFilePath,
            string? previewOutputPath,
            string? metadataOutputPath)
        {
            Format = format;
            InputPath = inputPath;
            OutputPath = outputPath;
            EncodingName = encodingName;
            BinaryInput = binaryInput;
            DoVerb = doVerb;
            Method = method;
            PreVerb = preVerb;
            PrefsFilePath = prefsFilePath;
            PreviewOutputPath = previewOutputPath;
            MetadataOutputPath = metadataOutputPath;
        }

        public string Format { get; }
        public string InputPath { get; }
        public string OutputPath { get; }
        public string EncodingName { get; }
        public bool BinaryInput { get; }
        public bool DoVerb { get; }
        public string Method { get; }
        public int? PreVerb { get; }
        public string? PrefsFilePath { get; }
        public string? PreviewOutputPath { get; }
        public string? MetadataOutputPath { get; }

        public static Options Parse(string[] args)
        {
            string? format = null;
            string? input = null;
            string? output = null;
            var encoding = "utf8";
            var binary = false;
            var doVerb = true;
            var method = "create-from-data";
            int? preVerb = null;
            string? prefsFilePath = null;
            string? previewOutput = null;
            string? metadataOutput = null;

            for (var i = 0; i < args.Length; i++)
            {
                switch (args[i])
                {
                    case "--format":
                        format = args[++i];
                        break;
                    case "--input":
                        input = args[++i];
                        break;
                    case "--output":
                        output = args[++i];
                        break;
                    case "--encoding":
                        encoding = args[++i];
                        break;
                    case "--binary":
                        binary = true;
                        break;
                    case "--no-verb":
                        doVerb = false;
                        break;
                    case "--method":
                        method = args[++i];
                        break;
                    case "--pre-verb":
                        preVerb = int.Parse(args[++i]);
                        break;
                    case "--prefs-file":
                        prefsFilePath = args[++i];
                        break;
                    case "--preview-output":
                        previewOutput = args[++i];
                        break;
                    case "--metadata-output":
                        metadataOutput = args[++i];
                        break;
                    default:
                        throw new ArgumentException($"Unknown argument: {args[i]}");
                }
            }

            if (format is null || input is null || output is null)
            {
                throw new ArgumentException("Usage: MathTypeOleHelper --format <clipboard format> --input <file> --output <ole.bin> [--encoding utf8|utf16le] [--binary] [--no-verb] [--method create-from-data|init-from-data|set-data] [--pre-verb N] [--prefs-file <eqp>] [--preview-output <wmf>] [--metadata-output <json>]");
            }

            return new Options(
                format ?? "",
                input ?? "",
                output ?? "",
                encoding,
                binary,
                doVerb,
                method,
                preVerb,
                prefsFilePath,
                previewOutput,
                metadataOutput);
        }
    }

    private sealed class Payload
    {
        public Payload(ushort formatId, byte[] bytes)
        {
            FormatId = formatId;
            Bytes = bytes;
        }

        public ushort FormatId { get; }
        public byte[] Bytes { get; }
    }

    private sealed class MathTypeLastDimensions
    {
        public MathTypeLastDimensions(
            int widthRaw,
            int heightRaw,
            int baselineRaw,
            int horizPosType,
            int horizPos)
        {
            WidthRaw = widthRaw;
            HeightRaw = heightRaw;
            BaselineRaw = baselineRaw;
            HorizPosType = horizPosType;
            HorizPos = horizPos;
        }

        public int WidthRaw { get; }
        public int HeightRaw { get; }
        public int BaselineRaw { get; }
        public int HorizPosType { get; }
        public int HorizPos { get; }

        private static double ToPoints(int value) => value / MATH_TYPE_DIMENSION_UNITS_PER_POINT;

        public string ToJson() => FormattableString.Invariant(
            $"{{\"width_raw\":{WidthRaw},\"height_raw\":{HeightRaw},\"baseline_from_bottom_raw\":{BaselineRaw},\"width_pt\":{ToPoints(WidthRaw):F4},\"height_pt\":{ToPoints(HeightRaw):F4},\"baseline_from_bottom_pt\":{ToPoints(BaselineRaw):F4},\"horiz_pos_type\":{HorizPosType},\"horiz_pos\":{HorizPos}}}");
    }

    private sealed class PreviewMetadata
    {
        public PreviewMetadata(
            int mapMode,
            int xExt,
            int yExt,
            ushort unitsPerInch,
            double widthPt,
            double heightPt,
            MathTypeLastDimensions? mathType)
        {
            MapMode = mapMode;
            XExt = xExt;
            YExt = yExt;
            UnitsPerInch = unitsPerInch;
            WidthPt = widthPt;
            HeightPt = heightPt;
            MathType = mathType;
        }

        public int MapMode { get; }
        public int XExt { get; }
        public int YExt { get; }
        public ushort UnitsPerInch { get; }
        public double WidthPt { get; }
        public double HeightPt { get; }
        public MathTypeLastDimensions? MathType { get; }

        public string ToJson()
        {
            var mathTypeJson = MathType is null ? "null" : MathType.ToJson();
            return FormattableString.Invariant(
                $"{{\"map_mode\":{MapMode},\"x_ext\":{XExt},\"y_ext\":{YExt},\"units_per_inch\":{UnitsPerInch},\"width_pt\":{WidthPt:F4},\"height_pt\":{HeightPt:F4},\"mathtype\":{mathTypeJson}}}");
        }
    }
}

[ComVisible(true)]
internal sealed class SingleFormatDataObject : System.Runtime.InteropServices.ComTypes.IDataObject
{
    private readonly short _formatId;
    private readonly byte[] _bytes;

    public SingleFormatDataObject(ushort formatId, byte[] bytes)
    {
        _formatId = unchecked((short)formatId);
        _bytes = bytes;
    }

    public void GetData(ref FORMATETC format, out STGMEDIUM medium)
    {
        if (!CanServe(format))
        {
            Marshal.ThrowExceptionForHR(unchecked((int)0x80040064)); // DV_E_FORMATETC
        }

        medium = new STGMEDIUM
        {
            tymed = TYMED.TYMED_HGLOBAL,
            unionmember = CopyToHGlobal(_bytes),
            pUnkForRelease = null,
        };
    }

    public void GetDataHere(ref FORMATETC format, ref STGMEDIUM medium)
    {
        Marshal.ThrowExceptionForHR(unchecked((int)0x80040064)); // DV_E_FORMATETC
    }

    public int QueryGetData(ref FORMATETC format)
    {
        return CanServe(format) ? 0 : unchecked((int)0x80040064);
    }

    public int GetCanonicalFormatEtc(ref FORMATETC formatIn, out FORMATETC formatOut)
    {
        formatOut = formatIn;
        return unchecked((int)0x80040064); // DATA_S_SAMEFORMATETC would also be acceptable, but this is simpler.
    }

    public void SetData(ref FORMATETC formatIn, ref STGMEDIUM medium, bool release)
    {
        Marshal.ThrowExceptionForHR(unchecked((int)0x80040064)); // DV_E_FORMATETC
    }

    public IEnumFORMATETC EnumFormatEtc(DATADIR direction)
    {
        throw new NotImplementedException();
    }

    public int DAdvise(ref FORMATETC pFormatetc, ADVF advf, IAdviseSink adviseSink, out int connection)
    {
        connection = 0;
        return Program.E_NOTIMPL_FOR_COM;
    }

    public void DUnadvise(int connection)
    {
        throw new NotImplementedException();
    }

    public int EnumDAdvise(out IEnumSTATDATA enumAdvise)
    {
        enumAdvise = null!;
        return Program.E_NOTIMPL_FOR_COM;
    }

    private bool CanServe(FORMATETC format)
    {
        return format.cfFormat == _formatId && (format.tymed & TYMED.TYMED_HGLOBAL) != 0;
    }

    private static IntPtr CopyToHGlobal(byte[] bytes)
    {
        var handle = GlobalAlloc(0x0042, (UIntPtr)bytes.Length);
        if (handle == IntPtr.Zero)
        {
            throw new OutOfMemoryException("GlobalAlloc failed");
        }

        var locked = GlobalLock(handle);
        if (locked == IntPtr.Zero)
        {
            GlobalFree(handle);
            throw new InvalidOperationException("GlobalLock failed");
        }
        try
        {
            Marshal.Copy(bytes, 0, locked, bytes.Length);
        }
        finally
        {
            GlobalUnlock(handle);
        }

        return handle;
    }

    [DllImport("kernel32.dll")]
    private static extern IntPtr GlobalAlloc(uint uFlags, UIntPtr dwBytes);

    [DllImport("kernel32.dll")]
    private static extern IntPtr GlobalLock(IntPtr hMem);

    [DllImport("kernel32.dll")]
    private static extern bool GlobalUnlock(IntPtr hMem);

    [DllImport("kernel32.dll")]
    private static extern IntPtr GlobalFree(IntPtr hMem);
}

[ComImport]
[Guid("00000112-0000-0000-C000-000000000046")]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
internal interface IOleObject
{
    void SetClientSite(IOleClientSite? pClientSite);
    void GetClientSite(out IOleClientSite? ppClientSite);
    void SetHostNames([MarshalAs(UnmanagedType.LPWStr)] string szContainerApp, [MarshalAs(UnmanagedType.LPWStr)] string szContainerObj);
    void Close(uint dwSaveOption);
    void SetMoniker(uint dwWhichMoniker, object? pmk);
    void GetMoniker(uint dwAssign, uint dwWhichMoniker, out object? ppmk);
    void InitFromData(System.Runtime.InteropServices.ComTypes.IDataObject pDataObject, bool fCreation, uint dwReserved);
    void GetClipboardData(uint dwReserved, out System.Runtime.InteropServices.ComTypes.IDataObject ppDataObject);
    void DoVerb(int iVerb, IntPtr lpmsg, IOleClientSite? pActiveSite, int lindex, IntPtr hwndParent, ref RECT lprcPosRect);
    void EnumVerbs(out object ppEnumOleVerb);
    void Update();
    void IsUpToDate();
    void GetUserClassID(out Guid pClsid);
    void GetUserType(uint dwFormOfType, out IntPtr pszUserType);
    void SetExtent(uint dwDrawAspect, ref SIZEL psizel);
    void GetExtent(uint dwDrawAspect, out SIZEL psizel);
    void Advise(IAdviseSink pAdvSink, out int pdwConnection);
    void Unadvise(int dwConnection);
    void EnumAdvise(out object ppenumAdvise);
    void GetMiscStatus(uint dwAspect, out uint pdwStatus);
    void SetColorScheme(IntPtr pLogpal);
}

[ComImport]
[Guid("00000118-0000-0000-C000-000000000046")]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
internal interface IOleClientSite
{
    [PreserveSig]
    int SaveObject();
    [PreserveSig]
    int GetMoniker(uint dwAssign, uint dwWhichMoniker, out object? ppmk);
    [PreserveSig]
    int GetContainer(out object? ppContainer);
    [PreserveSig]
    int ShowObject();
    [PreserveSig]
    int OnShowWindow(bool fShow);
    [PreserveSig]
    int RequestNewObjectLayout();
}

[ComVisible(true)]
internal sealed class MinimalOleClientSite : IOleClientSite
{
    public int SaveObject() => S_OK;
    public int GetMoniker(uint dwAssign, uint dwWhichMoniker, out object? ppmk)
    {
        ppmk = null;
        return E_NOTIMPL;
    }
    public int GetContainer(out object? ppContainer)
    {
        ppContainer = null;
        return E_NOTIMPL;
    }
    public int ShowObject() => S_OK;
    public int OnShowWindow(bool fShow) => S_OK;
    public int RequestNewObjectLayout() => E_NOTIMPL;

    private const int S_OK = 0;
    private const int E_NOTIMPL = unchecked((int)0x80004001);
}

[StructLayout(LayoutKind.Sequential)]
internal struct RECT
{
    public int left;
    public int top;
    public int right;
    public int bottom;
}

[StructLayout(LayoutKind.Sequential)]
internal struct SIZEL
{
    public int cx;
    public int cy;
}

[ComImport]
[Guid("0000000B-0000-0000-C000-000000000046")]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
internal interface IStorage
{
    [PreserveSig]
    int CreateStream([MarshalAs(UnmanagedType.LPWStr)] string pwcsName, int grfMode, int reserved1, int reserved2, out IStream ppstm);
    [PreserveSig]
    int OpenStream([MarshalAs(UnmanagedType.LPWStr)] string pwcsName, IntPtr reserved1, int grfMode, int reserved2, out IStream ppstm);
    [PreserveSig]
    int CreateStorage([MarshalAs(UnmanagedType.LPWStr)] string pwcsName, int grfMode, int reserved1, int reserved2, out IStorage ppstg);
    [PreserveSig]
    int OpenStorage([MarshalAs(UnmanagedType.LPWStr)] string pwcsName, IStorage? pstgPriority, int grfMode, IntPtr snbExclude, int reserved, out IStorage ppstg);
    [PreserveSig]
    int CopyTo(int ciidExclude, IntPtr rgiidExclude, IntPtr snbExclude, IStorage pstgDest);
    [PreserveSig]
    int MoveElementTo([MarshalAs(UnmanagedType.LPWStr)] string pwcsName, IStorage pstgDest, [MarshalAs(UnmanagedType.LPWStr)] string pwcsNewName, int grfFlags);
    [PreserveSig]
    int Commit(int grfCommitFlags);
    [PreserveSig]
    int Revert();
    [PreserveSig]
    int EnumElements(int reserved1, IntPtr reserved2, int reserved3, out object ppenum);
    [PreserveSig]
    int DestroyElement([MarshalAs(UnmanagedType.LPWStr)] string pwcsName);
    [PreserveSig]
    int RenameElement([MarshalAs(UnmanagedType.LPWStr)] string pwcsOldName, [MarshalAs(UnmanagedType.LPWStr)] string pwcsNewName);
    [PreserveSig]
    int SetElementTimes([MarshalAs(UnmanagedType.LPWStr)] string pwcsName, IntPtr pctime, IntPtr patime, IntPtr pmtime);
    [PreserveSig]
    int SetClass(ref Guid clsid);
    [PreserveSig]
    int SetStateBits(int grfStateBits, int grfMask);
    [PreserveSig]
    int Stat(out System.Runtime.InteropServices.ComTypes.STATSTG pstatstg, int grfStatFlag);
}

[ComImport]
[Guid("0000010A-0000-0000-C000-000000000046")]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
internal interface IPersistStorage
{
    void GetClassID(out Guid pClassID);
    [PreserveSig]
    int IsDirty();
    void InitNew(IStorage pStg);
    void Load(IStorage pStg);
    void Save(IStorage pStgSave, bool fSameAsLoad);
    void SaveCompleted(IStorage pStgNew);
    void HandsOffStorage();
}
