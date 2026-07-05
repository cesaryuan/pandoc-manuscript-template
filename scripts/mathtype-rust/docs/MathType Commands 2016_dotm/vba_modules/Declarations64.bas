Attribute VB_Name = "Declarations64"

'Declarations64.bas
'64 bit declarations

'=====================================================================
' (c) Copyright 1992-2011 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OS/Win/64-bits/Declarations64.bas 6     1/24/14 10:10a Jimm $
'=====================================================================

Public Const kBits As Long = 64

'=====================================================================
'MATHPAGE declarations
'=====================================================================
#If Mac Then
Public Declare PtrSafe Function MTAPIVersion Lib "/Library/Frameworks/MT6-64Lib.framework/MT6-64Lib" (ByVal api As Integer) As Long
Public Declare PtrSafe Function MTInitAPI Lib "/Library/Frameworks/MT6-64Lib.framework/MT6-64Lib" Alias "MTAPIConnect" (ByVal options As Integer, ByVal timeout As Integer) As Long
Public Declare PtrSafe Function MTTermAPI Lib "/Library/Frameworks/MT6-64Lib.framework/MT6-64Lib" Alias "MTAPIDisconnect" () As Long
Public Declare PtrSafe Function MTIsFullFunctionality Lib "/Library/Frameworks/MT6-64Lib.framework/MT6-64Lib" () As Long
Public Declare PtrSafe Function MTGetUserString Lib "/Library/Frameworks/MT6-64Lib.framework/MT6-64Lib" ( _
    ByVal str As String, ByVal buffer As String, ByRef length As Long) As Long
Public Declare PtrSafe Function MTGetUserWString Lib "/Library/Frameworks/MT6-64Lib.framework/MT6-64Lib" (ByVal str As String, ByRef buffer As Byte, ByRef length As Long) As Long
Public Declare PtrSafe Function MTIncrementStatisticBy Lib "/Library/Frameworks/MT6-64Lib.framework/MT6-64Lib" ( _
    ByVal name As String, ByVal value As Long) As Long
Public Declare PtrSafe Function MTSetStatistic Lib "/Library/Frameworks/MT6-64Lib.framework/MT6-64Lib" ( _
    ByVal name As String, ByVal value As Long) As Long
Public Declare PtrSafe Function MTGetWPathToMathType Lib "/Library/Frameworks/MT6-64Lib.framework/MT6-64Lib" ( _
    ByRef buffer As Byte, ByRef pathLen As Long) As Long
Public Declare PtrSafe Function MTGetURL Lib "/Library/Frameworks/MT6-64Lib.framework/MT6-64Lib" ( _
    ByVal whichURL As Long, ByVal bGoToURL As Boolean, ByVal strURL As String, ByVal sizeURL As Long) As Long
Public Declare PtrSafe Function MTCrashTest Lib "/Library/Frameworks/MT6-64Lib.framework/MT6-64Lib" ( _
    ByVal where As Long, ByVal how As Long) As Long
Public Declare PtrSafe Function MTCopyButtonFace Lib "/Library/Frameworks/MT6-64Lib.framework/MT6-64Lib" (ByVal tag As String) As Long
Public Declare PtrSafe Function MTSetPreference Lib "/Library/Frameworks/MT6-64Lib.framework/MT6-64Lib" ( _
    ByVal name As String, ByVal value As String, ByVal section As String, ByVal filePath As String) As Long
Public Declare PtrSafe Function MTGetPreference Lib "/Library/Frameworks/MT6-64Lib.framework/MT6-64Lib" ( _
    ByVal name As String, ByVal value As String, ByRef valueLen As Integer, ByVal section As String, ByVal filePath As String) As Long
Public Declare PtrSafe Function MTCloseOleObject Lib "/Library/Frameworks/MT6-64Lib.framework/MT6-64Lib" (ByVal dwSaveOpts As Long, ByVal varOleObject As Object) As Long
Public Declare PtrSafe Function MTShowAboutBox Lib "/Library/Frameworks/MT6-64Lib.framework/MT6-64Lib" () As Long
Public Declare PtrSafe Function MTHelpLaunch Lib "/Library/Frameworks/MT6-64Lib.framework/MT6-64Lib" Alias "MTHelp" (ByVal topic As Long) As Long
#End If

#If Win32 Then
Public Declare PtrSafe Function MPAnalyzeSymbol Lib "MathPage.WLL" (ByVal charCode As Integer, ByVal font As String, ByRef info As SymbolInfo) As Long
Public Declare PtrSafe Function MPAPIVersion Lib "MathPage.WLL" (ByVal api As Integer) As Long
Public Declare PtrSafe Function MPCopySupportFiles Lib "MathPage.WLL" () As Long
Public Declare PtrSafe Function MPDocInit Lib "MathPage.WLL" (ByVal docDir As String, ByVal support As String, ByVal wordVer As Integer, ByVal flags As Integer, ByVal target As String) As Long
Public Declare PtrSafe Function MPDocTerm Lib "MathPage.WLL" () As Long
Public Declare PtrSafe Function MPEnumMathMLTarget Lib "MathPage.WLL" (ByVal index As Integer, ByRef targetInfo As MPMathMLTarget) As Long
Public Declare PtrSafe Function MPEnumMathMLTarget2 Lib "MathPage.WLL" (ByVal index As Integer, ByRef targetInfo As MPMathMLTarget2) As Long
Public Declare PtrSafe Function MPFileCleanup Lib "MathPage.WLL" (ByVal fileName As String, ByVal supportDir As String) As Long
#If Word Then
'Range is undefined in PowerPoint
Public Declare PtrSafe Function MPFindSymbol Lib "MathPage.WLL" (ByVal Range As Range, ByVal font As String, ByRef info As SymbolInfo) As Long
#End If
Public Declare PtrSafe Function MPOpenFileInBrowser Lib "MathPage.WLL" (ByVal path As String) As Long
Public Declare PtrSafe Function MPProcessEquation Lib "MathPage.WLL" (ByVal gifFile As String, ByRef info As GIFInfo, ByVal tagAttrs As String) As Long
Public Declare PtrSafe Function MPProcessEquation2 Lib "MathPage.WLL" (ByVal gifFile As String, ByRef info As GIFInfo2, ByVal tagAttrs As String) As Long
Public Declare PtrSafe Function MPProcessHTML Lib "MathPage.WLL" (ByVal inPath As String, ByVal outPath As String) As Long
Public Declare PtrSafe Function MPProcessHTML2 Lib "MathPage.WLL" (ByVal inPath As String, ByVal outPath As String, ByVal isMasterDoc As Long) As Long
Public Declare PtrSafe Function MPProcessSymbol Lib "MathPage.WLL" (ByVal gifFile As String, ByVal font As String, ByRef info As SymbolInfo, ByVal symbolID As String, ByVal tagAttrs As String) As Long
Public Declare PtrSafe Function MTAPIGetNestingLevel Lib "MathPage.WLL" () As Long
Public Declare PtrSafe Function MTAPIVersion Lib "MathPage.WLL" (ByVal api As Integer) As Long
Public Declare PtrSafe Function MTClearClipboard Lib "MathPage.WLL" () As Long
Public Declare PtrSafe Function MTCloseOleObject Lib "MathPage.WLL" (ByVal dwSaveOpts As Long, ByVal varOleObject As Object) As Long
Public Declare PtrSafe Function MTConvertPrefsToUIForm Lib "MathPage.WLL" (ByVal inPrefs As String, ByVal outPrefs As String, ByVal outPrefsLen As Integer) As Long
Public Declare PtrSafe Function MTCopyButtonFace Lib "MathPage.WLL" (ByVal tag As String) As Long
Public Declare PtrSafe Function MTCrashTest Lib "MathPage.WLL" (ByVal where As Long, ByVal how As Long) As Long
Public Declare PtrSafe Function MTCreateDirectory Lib "MathPage.WLL" (ByVal directory As String) As Long
Public Declare PtrSafe Function MTEnumTranslators Lib "MathPage.WLL" (ByVal index As Integer, ByVal transName As String, ByVal transNameLen As Integer, ByVal transDesc As String, ByVal transDescLen As Integer, ByVal transFile As String, ByVal transFileLen As Integer) As Long
Public Declare PtrSafe Function MTEquationOnClipboard Lib "MathPage.WLL" () As Long
Public Declare PtrSafe Function MTGetAppFunctionality Lib "MathPage.WLL" () As Long
Public Declare PtrSafe Function MTGetClipboardText Lib "MathPage.WLL" (ByVal cbText As String, ByRef cbTextLen As Long) As Long
Public Declare PtrSafe Function MTGetLangBytesFromEqn Lib "MathPage.WLL" (ByVal myObj As Object, ByVal langType As Integer, ByRef langBuff As Byte, ByRef buffLen As Long) As Long
Public Declare PtrSafe Function MTGetLangStrFromEqn Lib "MathPage.WLL" (ByVal myObj As Object, ByVal langType As Integer, ByVal langStr As String, ByRef strLen As Long) As Long
Public Declare PtrSafe Function MTGetLastDimension Lib "MathPage.WLL" (ByVal dimIndex As Integer) As Long
Public Declare PtrSafe Function MTGetLocaleDLL Lib "MathPage.WLL" (ByVal buffer As String, ByRef length As Long) As Long
Public Declare PtrSafe Function MTGetLocaleString Lib "MathPage.WLL" (ByVal str As String, ByVal buffer As String, ByRef length As Long) As Long
Public Declare PtrSafe Function MTGetMathMLFromClipboard Lib "MathPage.WLL" (ByVal mathMlStr As String, ByRef mathMlDataLen As Long) As Long
Public Declare PtrSafe Function MTGetMathMLFromClipboardText Lib "MathPage.WLL" (ByVal mathMlStr As String, ByRef mathMlDataLen As Long) As Long
Public Declare PtrSafe Function MTGetOMMLFromClipboardHTML Lib "MathPage.WLL" (ByVal ommlData As String, ByRef ommlDataLen As Long) As Long
Public Declare PtrSafe Function MTGetOMMLFromClipboardPNGOA Lib "MathPage.WLL" (ByVal ommlData As String, ByRef ommlDataLen As Long) As Long
Public Declare PtrSafe Function MTGetOMMLFromClipboardRTF Lib "MathPage.WLL" (ByVal ommlData As String, ByRef ommlDataLen As Long) As Long
Public Declare PtrSafe Function MTGetPreference Lib "MathPage.WLL" (ByVal name As String, ByVal value As String, ByRef valueLen As Integer, ByVal section As String, ByVal filePath As String) As Long
Public Declare PtrSafe Function MTGetPrefsFromClipboard Lib "MathPage.WLL" (ByVal prefs As String, ByVal prefsLen As Integer) As Long
Public Declare PtrSafe Function MTGetPrefsFromFile Lib "MathPage.WLL" (ByVal prefFile As String, ByVal prefs As String, ByVal prefsLen As Integer) As Long
Public Declare PtrSafe Function MTGetPrefsMTDefault Lib "MathPage.WLL" (ByVal prefs As String, ByVal prefsLen As Integer) As Long
Public Declare PtrSafe Function MTGetTickCount Lib "MathPage.WLL" () As Long
Public Declare PtrSafe Function MTGetTranslatorsInfo Lib "MathPage.WLL" (ByVal infoIndex As Integer) As Long
Public Declare PtrSafe Function MTGetURL Lib "MathPage.WLL" (ByVal whichURL As Long, ByVal bGoToURL As Boolean, ByVal strURL As String, ByVal sizeURL As Long) As Long
Public Declare PtrSafe Function MTGetUserString Lib "MathPage.WLL" (ByVal str As String, ByVal buffer As String, ByRef length As Long) As Long
Public Declare PtrSafe Function MTGetUserWString Lib "MathPage.WLL" (ByVal str As String, ByRef buffer As Byte, ByRef length As Long) As Long
Public Declare PtrSafe Function MTIncrementStatisticBy Lib "MathPage.WLL" (ByVal name As String, ByVal value As Long) As Long
Public Declare PtrSafe Function MTInitAPI Lib "MathPage.WLL" (ByVal options As Integer, ByVal timeout As Integer) As Long
Public Declare PtrSafe Function MTInitLocaleStringDLL Lib "MathPage.WLL" (ByVal version As Long, ByVal languageID As Long) As Long
Public Declare PtrSafe Function MTInsertHandwrittenMath Lib "MathPage.WLL" () As Long
Public Declare PtrSafe Function MTIsClipboardFormatAvailable Lib "MathPage.WLL" (ByVal lpsFormat As String, ByVal iFormat As Integer, ByRef isAvailable As Boolean) As Long
Public Declare PtrSafe Function MTIsFullFunctionality Lib "MathPage.WLL" (Optional ByVal ShowDialog As Boolean = True) As Long
Public Declare PtrSafe Function MTIsMathInputPanelAvailable Lib "MathPage.WLL" (ByRef isAvailable As Boolean) As Long
Public Declare PtrSafe Function MTIsMathInputPanelTheClipboardOwner Lib "MathPage.WLL" (ByRef mipIsCBOwner As Boolean) As Long
Public Declare PtrSafe Function MTOpenFileDialog Lib "MathPage.WLL" (ByVal fileType As Integer, ByVal title As String, ByVal dir As String, ByVal file As String, ByVal fileLen As Integer) As Long
Public Declare PtrSafe Function MTPreviewDialog Lib "MathPage.WLL" (ByVal parent As Long, ByVal title As String, ByVal prefs As String, ByVal closeBtnText As String, ByVal helpBtnText As String, ByVal helpID As Long, ByVal helpFile As String) As Long
Public Declare PtrSafe Function MTSaveFileDialog Lib "MathPage.WLL" (ByVal title As String, ByVal dir As String, ByVal file As String, ByVal filterName As String, ByVal filterSpec As String, ByVal buffer As String, ByRef length As Long) As Long
Public Declare PtrSafe Function MTSetClipboardText Lib "MathPage.WLL" (ByVal cbText As String) As Long
Public Declare PtrSafe Function MTSetEqnFromLangBytes Lib "MathPage.WLL" (ByVal myObj As Object, ByVal langType As Integer, langBuff As Byte, ByVal buffLen As Long) As Long
Public Declare PtrSafe Function MTSetEqnFromLangStr Lib "MathPage.WLL" (ByVal myObj As Object, ByVal langType As Integer, langStr As Any, ByVal strLen As Long) As Long
Public Declare PtrSafe Function MTSetMTPrefs Lib "MathPage.WLL" (ByVal mode As Integer, ByVal prefs As String, ByVal timeout As Integer) As Long
Public Declare PtrSafe Function MTSetPreference Lib "MathPage.WLL" (ByVal name As String, ByVal value As String, ByVal section As String, ByVal filePath As String) As Long
Public Declare PtrSafe Function MTSetStatistic Lib "MathPage.WLL" (ByVal name As String, ByVal value As Long) As Long
Public Declare PtrSafe Function MTShowAboutBox Lib "MathPage.WLL" () As Long
Public Declare PtrSafe Function MTTermAPI Lib "MathPage.WLL" () As Long
Public Declare PtrSafe Function MTXFormAddVarSub Lib "MathPage.WLL" (ByVal options As Integer, ByVal findType As Integer, ByVal find As String, ByVal findLen As Long, ByVal replaceType As Integer, ByVal replace As String, ByVal replaceLen As Long, ByVal replaceStyle As Integer) As Long
Public Declare PtrSafe Function MTXFormEqn Lib "MathPage.WLL" (ByVal src As Integer, ByVal srcFmt As Integer, ByVal srcData As String, ByVal srcDataLen As Long, ByVal dst As Integer, ByVal dstFmt As Integer, ByVal dstData As String, ByVal dstDataLen As Long, ByVal dstPath As String, ByRef dims As MTAPI_DIMS) As Long
Public Declare PtrSafe Function MTXFormEqnBytes Lib "MathPage.WLL" Alias "MTXFormEqn" (ByVal src As Integer, ByVal srcFmt As Integer, srcData As Byte, ByVal srcDataLen As Long, ByVal dst As Integer, ByVal dstFmt As Integer, ByVal dstData As String, ByVal dstDataLen As Long, ByVal dstPath As String, ByRef dims As MTAPI_DIMS) As Long
Public Declare PtrSafe Function MTXFormGetStatus Lib "MathPage.WLL" (ByVal index As Integer) As Long
Public Declare PtrSafe Function MTXFormReset Lib "MathPage.WLL" () As Long
Public Declare PtrSafe Function MTXFormSetPrefs Lib "MathPage.WLL" (ByVal prefType As Integer, ByVal prefStr As String) As Long
Public Declare PtrSafe Function MTXFormSetTranslator Lib "MathPage.WLL" (ByVal options As Integer, ByVal transName As String) As Long

'=====================================================================
'WINDOWS OS declarations
'=====================================================================

'For IDataObject manipulations
Public Declare PtrSafe Function GlobalAlloc Lib "kernel32" (ByVal wFlags As Long, ByVal dwBytes As Long) As Long
Public Declare PtrSafe Function GlobalFree Lib "kernel32" (ByVal hMem As Long) As Long
Public Declare PtrSafe Function GlobalLock Lib "kernel32" (ByVal hMem As Long) As Long
Public Declare PtrSafe Function GlobalSize Lib "kernel32" (ByVal hMem As Long) As Long
Public Declare PtrSafe Function GlobalUnlock Lib "kernel32" (ByVal hMem As Long) As Long
Public Declare PtrSafe Function RegisterClipboardFormat Lib "user32" Alias "RegisterClipboardFormatA" (ByVal lpString As String) As Long
Public Declare PtrSafe Sub CopyMemory Lib "kernel32" Alias "RtlMoveMemory" (lpvDest As Any, lpvSource As Any, ByVal cbCopy As Long)

' Clipboard operations
Public Declare PtrSafe Function CloseClipboard Lib "user32" () As Integer
Public Declare PtrSafe Function CountClipboardFormats Lib "user32" () As Long
Public Declare PtrSafe Function EmptyClipboard Lib "user32" () As Long
Public Declare PtrSafe Function GetClipboardData Lib "user32" (ByVal wFormat As Integer) As Long
Public Declare PtrSafe Function lstrcpy Lib "kernel32" (ByVal lpString1 As Any, ByVal lpString2 As Any) As Long
Public Declare PtrSafe Function OpenClipboard Lib "user32" (ByVal hwnd As Integer) As Integer
Public Declare PtrSafe Function SetClipboardData Lib "user32" (ByVal wFormat As Long, ByVal hMem As Long) As Long

'System API's
Public Declare PtrSafe Function FindWindowByClass Lib "user32" Alias "FindWindowA" (ByVal lpClassName As String, ByVal lpWindowName As Long) As Long
Public Declare PtrSafe Function GetEnvironmentVariable Lib "kernel32" Alias "GetEnvironmentVariableA" (ByVal lpName As String, ByVal lpBuffer As String, ByVal nSize As Long) As Long
Public Declare PtrSafe Function GetFileVersionInfo Lib "Version.dll" Alias "GetFileVersionInfoA" (ByVal lptstrFilename As String, ByVal dwhandle As Long, ByVal dwlen As Long, lpData As Any) As Long
Public Declare PtrSafe Function GetFileVersionInfoSize Lib "Version.dll" Alias "GetFileVersionInfoSizeA" (ByVal lptstrFilename As String, lpdwHandle As Long) As Long
Public Declare PtrSafe Function GetForegroundWindow Lib "user32" () As Long
Public Declare PtrSafe Function GetKeyState Lib "user32" (ByVal vKey As Long) As Integer
Public Declare PtrSafe Function GetLastError Lib "kernel32" () As Long
Public Declare PtrSafe Function GetLocaleInfo Lib "kernel32" Alias "GetLocaleInfoA" (ByVal Locale As Long, ByVal LCType As Long, ByVal lpLCData As String, ByVal cchData As Long) As Long
Public Declare PtrSafe Function GetTickCount Lib "kernel32" () As Long
Public Declare PtrSafe Function GetWindowRect Lib "user32" (ByVal hwnd As Long, ByRef hMem As RECT) As Boolean
Public Declare PtrSafe Function SetEnvironmentVariable Lib "kernel32" Alias "SetEnvironmentVariableA" (ByVal lpName As String, ByVal lpValue As String) As Long
Public Declare PtrSafe Function SetForegroundWindow Lib "user32" (ByVal hwnd As Long) As Long
Public Declare PtrSafe Function SetWindowPos Lib "user32" (ByVal hwnd As Long, ByVal hWndInsertAfter As Long, ByVal topx As Integer, ByVal topy As Integer, ByVal width As Integer, ByVal height As Integer, ByVal flags As Integer) As Boolean
Public Declare PtrSafe Function ShellExecute Lib "SHELL32.DLL" Alias "ShellExecuteA" (ByVal hwnd As Long, ByVal lpOperation As String, ByVal lpFile As String, Optional ByVal lpParameters As String, Optional ByVal lpDirectory As String, Optional ByVal nShowCmd As Long) As Long
Public Declare PtrSafe Function SHGetSpecialFolderPath Lib "SHELL32.DLL" Alias "SHGetSpecialFolderPathA" (ByVal hwnd As Long, ByVal lpszPath As String, ByVal nFolder As Integer, ByVal fCreate As Boolean) As Boolean
Public Declare PtrSafe Function StringFromGUID2 Lib "OLE32.dll" (ByRef rGUID As Any, ByVal lpSz As String, ByVal cchMax As Long) As Long
Public Declare PtrSafe Function VerQueryValue Lib "Version.dll" Alias "VerQueryValueA" (pBlock As Any, ByVal lpSubBlock As String, lplpBuffer As Any, puLen As Long) As Long
Public Declare PtrSafe Sub MoveMemory Lib "kernel32" Alias "RtlMoveMemory" (dest As Any, ByVal Source As Long, ByVal length As Long)

'Help
Public Declare PtrSafe Function HtmlHelp Lib "hhctrl.ocx" Alias "HtmlHelpA" (ByVal hwnd As Long, ByVal lpHelpFile As String, ByVal wCommand As Long, ByVal dwData As Long) As Long
Public Declare PtrSafe Function WinHelp Lib "user32" Alias "WinHelpA" (ByVal hwnd As Long, ByVal lpHelpFile As String, ByVal wCommand As Long, ByVal dwData As Long) As Long

'Registry API's
Public Declare PtrSafe Function RegQueryInfoKey Lib "advapi32.dll" Alias "RegQueryInfoKeyA" (ByVal pRegKey As Long, sClass As String, sClassLen As Long, ByVal Reserved As Long, cSubKeys As Long, cMaxSubKeyLen As Long, cMaxClassLen As Long, cValues As Long, cMaxValueNameLen As Long, cMaxValueLen As Long, cbSecurityDescriptor As Long, ftLastWriteTime As Long) As Long

Public Declare PtrSafe Function RegCreateKeyEx Lib "advapi32.dll" _
    Alias "RegCreateKeyExA" ( _
    ByVal lngKey As LongPtr, _
    ByVal lpSubKey As String, _
    ByVal Reserved As Long, _
    ByVal lpClass As String, _
    ByVal dwOptions As Long, _
    ByVal samDesired As Long, _
    ByVal lpSecurityAttributes As Long, _
    phkResult As LongPtr, _
    ByVal lpdwDisposition As Long) As Long

Public Declare PtrSafe Function RegQueryValueExString Lib "advapi32.dll" _
    Alias "RegQueryValueExA" ( _
    ByVal lngKey As LongPtr, _
    ByVal lpValueName As String, _
    ByVal lpReserved As Long, _
    lpType As Long, _
    lpData As Any, _
    lpcbData As Long) As Long

Public Declare PtrSafe Function RegQueryValueExNULL Lib "advapi32.dll" _
    Alias "RegQueryValueExA" ( _
    ByVal lngKey As LongPtr, _
    ByVal lpValueName As String, _
    ByVal lpReserved As LongPtr, _
    lpType As Long, _
    ByVal lpData As Long, _
    lpcbData As Long) As Long

Public Declare PtrSafe Function RegOpenKeyEx Lib "advapi32.dll" _
    Alias "RegOpenKeyExA" ( _
    ByVal lngKey As LongPtr, _
    ByVal lpSubKey As String, _
    ByVal ulOptions As Long, _
    ByVal samDesired As Long, _
    phkResult As LongPtr) As Long

Public Declare PtrSafe Function RegCloseKey Lib "advapi32.dll" _
    (ByVal lngKey As LongPtr) As Long

Public Declare PtrSafe Function RegSetValueExString Lib "advapi32.dll" _
    Alias "RegSetValueExA" ( _
    ByVal hKey As LongPtr, _
    ByVal lpValueName As String, _
    ByVal Reserved As Long, _
    ByVal dwType As Long, _
    ByVal lpValue As String, _
    ByVal cbData As Long) As Long

#End If
'=====================================================================
