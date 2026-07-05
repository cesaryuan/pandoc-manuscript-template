---
title: "Getting started to MathType's API"
source: "https://docs.wiris.com/en_US/mathtype-sdk-technical-documentation/mathtype-api-documentation"
author:
published:
created: 2026-07-05
description: "The use of the MathType SDK requires MathType 7 to be activated with an SDK license. Please, contact our sales support team if you need further information"
tags:
  - "clippings"
---
- ## MathType
	- ## Getting started
		- ## Using MathType
		- ## Technical documentation
		- ## MathType 7
			- - ## Getting started to MathType's API
								- ## MathType SDK documentation
						- ## Network administrator's manual
						- ## MathType's command line options
				- ## MathType for Microsoft 365
				- ## MathType for HTML editors
				- ## MathType for LMS
		- ## Troubleshooting & FAQs
		- ## Reference and legal
- ## WirisQuizzes
- ## Nubric
- ## CalcMe
- ## MathPlayer
- ## Store FAQ
- ## MathFlow
- ## BF FAQ
- ## Miscellaneous
- ## Wiris Integrations

The use of the MathType SDK requires MathType 7 to be activated with an SDK license. Please, [contact our sales support team](https://www.wiris.com/en/contact-us/) if you need further information.

Please note this document applies only to MathType 6.9 and later for Windows, and MathType 6.7 and later for Mac.

## Introduction

The MathType API consists of a WLL, two Microsoft Word document templates containing VBA macros, and a separate DLL (on Windows only).

On Windows, the DLL is named `MT6.DLL` and is located in the System folder inside the MathType folder. This DLL manages all communications with MathType itself, launching MathType when necessary. Most of the functions in the DLL provide support for the MathType commands added to Word. All API functions in `MT6.DLL` begin with the prefix `MT`.

The WLL (basically a DLL with some Word-specific entry points) is named `MathPage.WLL`. On Windows, it is installed in the Office Startup folder. For the MathType 6 Word Commands on Windows, all entry points to the DLL are accessed via the WLL. This is partly due to the way VBA code calls functions in a DLL, and partly because a single entry-point makes for a cleaner architecture. We recommend that you follow this model. The WLL contains the MathPage functions (prefixed by MP), which help in converting a Word document to HTML. This is a fairly complicated process and you're unlikely to use any of the individual MathPage functions. The WLL also contains other functions used by the MathType Commands for Word; these functions begin with the `MT` prefix.

There are two Word templates located in the Office Support folder inside the MathType folder: the *stub*, named **MathType 6 Commands for Word.dot**; and the *commands*, named **WordCmds.dot**. The *stub* template is also copied to the Microsoft Word Startup location during installation, and gets loaded when Word starts. The first time a MathType command is used in Word, the *commands* template is explicitly loaded from the Office Support folder inside the MathType folder. These two templates are used in this manner in order to avoid long Word startup times. The *stub* template has been kept as small as possible so that a minimal delay will occur when Word starts. The *commands* template takes a few seconds to load; users only pay this price the first time they use any MathType command. The *stub* contains handlers for each toolbar button and menu command; each of them loads the *commands* template if it isn't loaded yet, and then calls a function in the *commands* template that actually does the work. If you want to call your own function in the WordCmds.dot template, be sure to use the same kind of handler as the existing commands. The handler functions not only verify the correct versions of the WLLs and load the WordCmds.dot template, they also call some initialization routines that are necessary for the API functions to work.

The best way to understand how a particular command works is to follow through the VBA code. The SDK contains unlocked copies of these templates so that you can see the code. It contains comments which should make it easier to understand. The file MT6SDK.dot (in the SDK's templates folder) contains some additional functions that may be useful if you want to suppress the dialogs that normally appear during Format, Convert, Export and Toggle Equations. See the [MathType Commands For Word API](https://docs.wiris.com/mathtype/en/) for details.

Two sample templates are provided: SDKTest.dot and MTVarSub.dot. The first template,SDKTest.dot, contains examples of how to call Format, Convert, Export and Toggle Equations non-interactively, as well as a simple search-and-replace example. MTVarSub.dot shows the many variations possible for search-and-replace, a.k.a variable substitution. To avoid problems with macro security in Word 2000 and above on Windows, set your security level to Medium or Low, since these macros are unsigned.You must create a reference to MT6SDK.dot before running any of the test macros in these two templates. To do this, switch to the Visual Basic Editor via ALT-F11 on Windows and select Tools|References, then select MT6SDK.dot from the list. If it is not in the list, use the Browse button to locate it in the templates folder inside the SDK folder.

## Office locations

The MathType installer places the stub template into the Word Startup folder which, on Windows, varies depending on the version of Office installed. This is a per-machine location as opposed to a per-user location, and Word will use this folder no matter which user is logged on. On Windows, the WLL is also copied to this location. On a Mac, it's a per-user location, and like Windows, varies depending on the version of Office installed.

## MathType commands for Word API

These are supplemental functions provided in MT6SDK.dot. They provide an easy way to call Format, Convert, Export and Toggle Equations without displaying a dialog. If you wish to perform any of these functions in a batch mode, you may not need to modify the VBA code itself but can simply call these functions. The values in the various *info* structures correspond to the commands' dialog settings.

### FormatEquations

`Public Sub FormatEquations(info As FormatEquationsInfo)`

Formats equations in the active document based on the passed settings.

#### Parameters

*info* FormatEquationsInfo structure defined below

```
Public Const mt_FMT_PREFS_DOCUMENT As Long = 1 'preferences stored in document
Public Const mt_FMT_PREFS_MATHTYPE As Long = 2 'MathType's equation preferences
Public Const mt_FMT_PREFS_CLIPBOARD As Long = 3 'equation on clipboard
Public Const mt_FMT_PREFS_FILE As Long = 4 'preference file
Public Type FormatEquationsInfo
showStats As Boolean 'show #equations formatted at end of processing document
prefsSource As Long 'source of preferences, one of the mt_FMT_PREFS constants above
prefsFileName As String 'prefs filename if prefsSource=mt_FMT_PREFS_FILE
selectionOnly As Boolean 'True=process selection only, False=whole document
savePrefs As Boolean 'True=save preferences in document
count As Long 'number of equations formatted
End Type
```

#### Return value

None

### ConvertEquations

`Public Sub ConvertEquations(info As ConvertEquationsInfo)`

Converts equations in the active document based on the passed settings.

#### Parameters

*info* ConvertEquationsInfo structure defined below

```
'Types and constants used for calling ConvertEquations
Public Const mt_CVT_FIND_MTEE As Long = 1 'find MathType and EE equations
Public Const mt_CVT_FIND_WORDEQ As Long = 2 'find Word EQ fields
Public Const mt_CVT_FIND_TRANSLATOR As Long = 4 'find translated (text) equations
Public Const mt_CVT_TRAN_NAME As Long = 1 'include translator name in translation
Public Const mt_CVT_TRAN_MTEF As Long = 2 'include MathType data in translation
Public Type ConvertEquationsInfo
showStats As Boolean 'show #equations formatted at end of processing document
equationTypes As Long 'types of equations to convert, one or more of the mt_CVT_FIND constants above
selectionOnly As Boolean 'True=process selection only, False=whole document
promptUser As Boolean 'True=prompt user whether to convert each equation
translatorFileName As String 'file name of the translator to use or "" to convert to MT equations
translatorOptions As Long 'options to the translator, one or more of the mt_CVT_TRAN options above
count As Long 'number of equations converted
End Type
```

#### Return value

None

### ExportEquations

`Public Sub ExportEquations(info As ExportEquationsInfo)`

Exports equations in the active document based on the passed settings.

#### Parameters

*info* ExportEquationsInfo structure defined below

```
'Types and constants used for calling ExportEquations
Public Const mt_EXP_TYPE_EPSWMF As Long = 0 'EPS/WMF
Public Const mt_EXP_TYPE_EPSTIFF As Long = 1 'EPS/TIFF
Public Const mt_EXP_TYPE_EPSNONE As Long = 2 'EPS/none
Public Const mt_EXP_TYPE_GIF As Long = 3 'GIF
Public Const mt_EXP_TYPE_WMF As Long = 4 'WMF
Public Type ExportEquationsInfo
showStats As Boolean 'show #equations formatted at end of processing document
folderPath As String 'path to folder to store exported equation files
deleteAll As Boolean 'True=delete all files in the output folder with the same extension
fileType As Long 'type of file to create for each equation, one of the mt_EXP_TYPE constants above
fileNamePattern As String 'file naming pattern, with # wildcards e.g. "EQN###"
firstNumber As Long 'starting equation number
replace As Boolean 'True=replace each equation with its filename
selectionOnly As Boolean 'True=process selection only, False=whole document
count As Long 'number of equations exported
End Type
```

#### Return value

None

### ToggleTexEquations

`Public Sub ToggleTexEquations(info As ToggleTexEquationsInfo)`

Toggles Tex equations in the range based on the passed settings.

#### Parameters

*info* ToggleTexEquationsInfo structure defined below

```
'Types and constants used for calling ToggleTexEquations
Public Const mt_RNG_TYPE_CURSOR As Long = 0 'nearest equation to the cursor location
Public Const mt_RNG_TYPE_SELECTION As Long = 1 'current selection
Public Const mt_RNG_TYPE_DOCUMENT As Long = 2 'whole document
Public Type ToggleTexEquationsInfo
rngType As Long 'ran
```

## MathPage API

These functions are stored in `MathPage.WLL` on Windows. Functions with an MP prefix perform parts of the MathPage processing. Those with an MT prefix perform other functions for the MathType Commands For Word but do not call into MathType itself.

### Return codes

The following return codes apply to all MathPage API functions, and are defined in the module `MPApi`:

```
'MathPage API return codes
mpOK = 0
mpEQN_NO_BASELINE = 1000
mpBAD_VERSION = -1000
mpMTDLL_NOT_FOUND = -1001
mpFILE_INVALID = -1002
mpFILE_NO_ACCESS = -1003
mpNOT_AN_EQUATION = -1004
mpERROR = -9999
```

### MathPage API types

```
'kind values used by MPProcessSymbol; also returned by MPAnalyzeSymbol
Public Const kSIDefault As Long = 0 'default value
Public Const kSIGIF As Long = 1 'needs a GIF
Public Const kSIEntity As Long = 2 'needs entity
Public Const kSIInsertSymbolPlaceholder As Long = 3 'maybe an inserted symbol
Public Const kSISubstituted As Long = 4 'char is being changed, 0 = delete char
Public Const kSIMissingFont As Long = 5 'font not installed
'style values used by MPProcessSymbol
Public Const kPlainText As Integer = 0
Public Const kBoldText As Integer = 1
Public Const kItalicText As Integer = 2
Type SymbolInfo
 kind As Integer 'out from MPFindSymbol, input to MPProcessSymbol (see above)
 charCode As Integer 'out from MPFindSymbol, input to MPProcessSymbol
 isUniCode As Boolean 'private to WLL
 size As Single
 style As Integer 'see style constants above
 color As Long 'RGB value
End Type
Type BorderInfo
 width As Long 'one of wdLineWidth
 style As Long 'one of wdLineStyle
 color As Long 'RGB value
End Type
Type GIFInfo
 smooth As Boolean 'True to use anti-aliasing
 bkgndColor As Long 'only needed if (smooth OR not transparent)
 size As Single 'size of surrounding text
 fillType As Long 'one of msoFillType or 0
 topBorder As BorderInfo
 leftBorder As BorderInfo
 bottomBorder As BorderInfo
 rightBorder As BorderInfo
End Type
'used for calls to MPProcessEquation2
Type GIFInfo2
 version As Integer 'MTW5.1 and above expect 1
 smooth As Boolean 'True to use anti-aliasing
 bkgndColor As Long 'only needed if (smooth OR not transparent)
 size As Single 'size of surrounding text
 fillType As Long 'one of msoFillType or 0
 topBorder As BorderInfo
 leftBorder As BorderInfo
 bottomBorder As BorderInfo
 rightBorder As BorderInfo
 display As Long 'one of mpdtXXX constants (see MPProcessEquation2)
End Type
'struct for MPEnumMathMLTarget
Type MPMathMLTarget
 browser As Integer '0-based compatibility choice, -1 = all
 minWordVer As Integer 'min version of Word that supports target (8,9,10...)
 nameMax As Integer 'max len of name
 targetName As String 'MathML target name
 descMax As Integer 'max len of description
 targetDesc As String 'MathML target description
End Type
'struct for MPEnumMathMLTarget2
Type MPMathMLTarget2
 version As Integer 'MTW5.1 and above expect 1
 browser As Integer '0-based compatibility choice, -1 = all
 minWordVer As Integer 'min version of Word that supports target (8,9,10...)
 nameMax As Integer 'max len of name
 targetName As String 'MathML target name
 descMax As Integer 'max len of description
 targetDesc As String 'MathML target description
 extMax As Integer 'max len of extensions
 extensions As String 'valid extensions (default is first)
End Type
```

### MPAPIVersion

```
Public Function MPAPIVersion(
 api As Integer
) As Long
```

Gets and checks the version of the MathPage API.

#### Parameters

*api* api version required (just major version), currently 5

#### Return value

Returns hi-byte (of lo-word) = major version, lo-byte (of lo-word) = minor version, or mpBAD\_VERSION. MathType 5.0 returns 0x0500, MathType 5.1 returns 0x0501.

### MTInitAPI

```
Public Function MTInitAPI (
 options As Integer,
 timeout As Integer
) As Long
```

Initializes the API (prepares for use). This function should always be called before any other API function (except MPAPIVersion). Other API functions will return an error if this function has not been called.

Calls to MTInitAPI should always be paired with calls to MTTermAPI, and they may be nested (i.e. multiple calls to MTInitAPI may be made without calling MTTermAPI, but, of course, must eventually be followed by the corresponding number of calls to MTTermAPI).

This function calls the MT6.DLL function MTAPIConnect().

#### Parameters

*options*

```
mtinitLAUNCH_NOW => launch MathType server immediately
mtinitLAUNCH_AS_NEEDED => launch MathType when first needed
```

*timeout*

```
# of seconds to wait before timing out when attempting to launch MathType. If timeOut = -1 then will never timeout. This value is eventually passed to RPCConnectToServer where it is not currently used.
```

#### Return value

If successful (>0) the value is the nesting level of MTInitAPI calls (i.e. the number of times MTInitAPI has been called minus the number of times MTTermAPI has been called).

Otherwise, error status: mtMT\_NOT\_FOUND, mtMT\_CANT\_RUN, mtMT\_BAD\_VERSION, mtMT\_IN\_USE, mtMT\_RUN\_TIME\_OUT, or mtERROR.

### MTTermAPI

```
Public Function MTTermAPI () As Long
```

Must always call after MTInitAPI. Closes connection with MathType and terminates the API (no further usage). See MTInitAPI for more details.

This function calls the MT6.DLL function MTAPIDisconnect().

#### Parameters

None.

#### Return value

If successful (>0) the value is the nesting level of MTInitAPI calls (i.e. the number of times MTInitAPI has been called minus the number of times MTTermAPI has been called).

Otherwise, error status: mtERROR - if MTInitAPI was not called successfully

### MPDocInit

```
Public Function MPDocInit (
 docDir As String,
 support As String,
 wordVer As Integer,
 flags As Integer,
 target As String
) As Long
```

Initializes processing for a document.

#### Parameters

| *docDir* | path of document's directory |
| --- | --- |
| *support* | support files directory name |
| *wordVer* | version of Word (14 = Word 2010, 15 = 2013, 16 = 2016) |
| *flags* | options for the conversion process: mpdFullCompatibility = 1 mpdGenMathML = 2 mpdMathZoom = 4 |
| *target* | name of MathML target (ignored unless mpdGenMathML set) |

#### Return value

Returns mpOK or mpERROR.

### MPDocTerm

```
Public Function MPDocTerm () As Long
```

Terminates the processing of the document.

#### Parameters

None

#### Return value

Returns mpOK.

### MPProcessEquation

```
Public Function MPProcessEquation (
 gifFile As String,
 info As GIFInfo,
 tagAttrs As String
) As Long
```

Converts equation on clipboard to GIFs or MathML based on conversion settings.

#### Parameters

| *gifFile* | unique identifier for GIF file. Include in name for resolution tag; function replaces it in actual filenames. |
| --- | --- |
| *info* | Info about the GIFs to be created. See type definitions above. |
| *tagAttrs* | result: attributes to be placed in the DSTAG for this eqn |

#### Return value

See Return Codes above.

### MPProcessEquation2

```
Public Function MPProcessEquation2 (
 gifFile As String,
 info As GIFInfo2,
 tagAttrs As String
) As Long
```

Converts equation on clipboard to GIFs or MathML based on conversion settings.

#### Parameters

| *gifFile* | unique identifier for GIF file. Include in name for resolution tag, function replaces it in actual filenames. |
| --- | --- |
| *info* | Info about the GIFs to be created. See type definitions above. |
| ::: | GIFInfo2 added in MTW5.1, requires a version value of 1, and adds a display value: |
| ::: | '' mpdtInternal (0) = use internal inline/display setting as set in MathType `\| \|::: \|` mpdtInline (1) = make inline equation `\| \|::: \|` mpdtDisplay (2) = make display equation'' |
| *tagAttrs* | result: attributes to be placed in the DSTAG for this eqn |

#### Return value

See Return Codes above.

### MPFindSymbol

```
Public Function MPFindSymbol (
 range As Range,
 font As String,
 info As SymbolInfo
) As Long
```

Finds the first symbol in a string (if any) that must be handled specially.

#### Parameters

| *range* | range of text to search |
| --- | --- |
| *font* | name of font |
| *info* | symbol description (char, size, style). See definition of SymbolInfo type above. |

#### Return value

Returns the index of the character (0-based), -1 if none.

### MPAnalyzeSymbol

```
Public Function MPAnalyzeSymbol (
 charCode As Integer,
 font As String,
 info As SymbolInfo
) As Long
```

Find the first symbol in a string (if any) that must be handled specially.

#### Parameters

| *charCode* | character code of symbol to analyze |
| --- | --- |
| *font* | name of font |
| *info* | Symbol description (char, size, style). See definition of SymbolInfo type above. |

#### Return value

Returns a copy of info.kind (kSIxxxx)

### MPProcessSymbol

```
Public Function MPProcessSymbol (
 gifFile As String,
 font As String,
 info As SymbolInfo,
 symbolID As String,
 tagAttrs As String
) As Long
```

Process a symbol, creating a GIF or MathML fragment.

#### Parameters

| *gifFile* | filename pattern containing and to be replaced |
| --- | --- |
| *font* | font name |
| *info* | symbol description (char, size, style) in SymbolInfo structure as defined above. |
| *symbolID* | result: symbol id |
| *tagAttrs* | result: attributes to be placed in the DSTAG for this symbol |

#### Return value

mpOK or mpERROR

### MPProcessHTML

```
Public Function MPProcessHTML (
 inPath As String,
 outPath As String
) As Long
```

Reads the document into memory, filters out Word's HTML, inserts MathPage header snippets, replaces equation and symbol DSTags by MathPage HTML snippets, converts to XHTML if indicated for MathML target, and writes the modified document out.

#### Parameters

| *inPath* | input filename, i.e. Word's SaveAs HTML |
| --- | --- |
| *outPath* | output filename, i.e. MathPage HTML |

#### Return value

mpOK or mpERROR.

### MPProcessHTML2

```
Public Function MPProcessHTML2 (
 inPath As String,
 outPath As String,
 isMasterDoc As Boolean
) As Long
```

Reads the document into memory, filters out Word's HTML, inserts MathPage header snippets, replaces equation and symbol DSTags by MathPage HTML snippets, converts to XHTML if indicated for MathML target, and writes the modified document out. MTW5.1 added isMasterDoc parameter (and hence this new call), needed for proper handling of master documents.

#### Parameters

| *inPath* | input filename, i.e. Word's SaveAs HTML |
| --- | --- |
| *outPath* | output filename, i.e. MathPage HTML |
| *isMasterDoc* | True if a master document |

#### Return value

mpOK or mpERROR.

### MPFileCleanup

```
Public Function MPFileCleanup (
 fileName As String,
 supportDir As String
) As Long
```

Cleans up in preparation for creating MathPage document. Deletes HTML file and/or supporting files folder, if found. Creates supporting files folder as necessary.

#### Parameters

| *fileName* | name of HTML file we'll be saving |
| --- | --- |
| *supportDir* | name of supporting files folder |

#### Return value

mpOK or mpERROR.

### MPEnumMathMLTarget

```
Public Function MPEnumMathMLTarget (
 index As Integer,
 targetInfo As MPMathMLTarget
) As Long
```

Enumerates the MathML targets in the MathPage dialog.

#### Parameters

| *index* | 0-based index |
| --- | --- |
| *targetInfo* | result: mathML target. See definition of MPMathMLTarget type above. |

#### Return value

Returns 1 if an entry is returned, 0 if end of entries.

### MPEnumMathMLTarget2

```
Public Function MPEnumMathMLTarget2 (
 index As Integer,
 targetInfo As MPMathMLTarget2
) As Long
```

Enumerates the MathML targets in the MathPage dialog. Added in MathType for Windows 5.1, returns allowable file extensions for target in targetInfo.

#### Parameters

| *index* | 0-based index |
| --- | --- |
| *targetInfo* | result: mathML target. See definition of MPMathMLTarget type above. |

#### Return value

Returns 1 if an entry is returned, 0 if end of entries, mpERROR if wrong version in struct.

### MPCopySupportFiles

```
Public Function MPCopySupportFiles () As Long
```

Copies the supporting files (depending on the target) into their appropriate locations. For GIF output, this copies MathPage.js and empty.gif, while the MathML targets can specify their own supporting files.

#### Parameters

None

#### Return value

See Return Codes above.

### MTInitLocaleStringDLL

```
Public Function MTInitLocaleStringDLL (
 version As Long,
 languageID As Long
) As Long
```

Locates locale-dependent string DLL; OK to call more than once.

#### Parameters

| *wordVersion* | version of Word |
| --- | --- |
| *languageID* | language ID |

#### Return value

Returns 0 if successful, else -1.

### MTGetLocaleDLL

```
Public Function MTGetLocaleDLL (
 buffer As String,
 length As Long
) As Long
```

Gets name of primary DLL

#### Parameters

| *buffer* | result: name of primary DLL |
| --- | --- |
| *length* | length of buffer |

#### Return value

Returns 0 if successful, else -1.

### MTGetLocaleString

```
Public Function MTGetLocaleString (
 str As String,
 buffer As String,
 length As Long
) As Long
```

Returns user string in buffer.

#### Parameters

| *str* | in: string of form "!nnnnThis is a string" |
| --- | --- |
| *buffer* | out: buffer for string |
| *length* | in/out; in: length of buffer, out: #chars in buffer |

#### Return value

Returns 0 if successful, else -1.

### MTGetUserString

```
Public Function MTGetUserString (
 str As String,
 buffer As String,
 length As Long
) As Long
```

Returns user string in buffer.

#### Parameters

| *str* | in: string of form "!nnnnThis is a string" |
| --- | --- |
| *buffer* | out: buffer for string |
| *length* | in/out; in: length of buffer, out: #chars in buffer |

#### Return value

Returns 0 if successful, else -1.

### MTSaveFileDialog

```
Public Function MTSaveFileDialog (
 title As String,
 dir As String,
 file As String,
 filterName As String,
 filterSpec As String,
 buffer As String,
 length As Long
) As Long
```

Displays Save File dialog.

#### Parameters

| *title* | title to display in dialog |
| --- | --- |
| *dir* | initial directory |
| *file* | initial filename |
| *filterName* |  |
| *filterSpec* |  |
| *buffer* | out: returns full path |
| *length* | length of buffer |

#### Return value

Returns directory name in buffer, 0 if successful, else -1 (user cancelled).

### MTCreateDirectory

```
Public Function MTCreateDirectory (
 directory As String
) As Long
```

Creates a directory.

#### Parameters

| *directory* | full path of directory to create |
| --- | --- |

#### Return value

Returns 0 if OK, mtBAD\_PATH or mtERROR.

### MTIsFullFunctionality

```
Public Function MTIsFullFunctionality () As Long
```

Tests if MathType full functionality should be available (i.e. not an expired demo).

#### Parameters

None

#### Return value

Returns 1 if full functionality is available, else 0

## MathType API

The following return codes apply to all MathType API functions:

```
' can't find MathType application
Public Const mtMT_NOT_FOUND As Long = -1
' can't run the MathType application
Public Const mtMT_CANT_RUN As Long = -2
' the MathType application is the wrong version
Public Const mtMT_BAD_VERSION As Long = -3
' the MathType application is already in use
Public Const mtMT_IN_USE As Long = -4
' the MathType application is not running (i.e. unexpectedly aborted)
Public Const mtMT_NOT_RUNNING As Long = -5
' time ran out waiting for the MathType application to start up
Public Const mtRUN_TIMEOUT As Long = -6
' not equation on clipboard
Public Const mtNOT_EQUATION As Long = -7
' file does not exist or bad pathname
Public Const mtFILE_NOT_FOUND As Long = -8
' insufficient memory
Public Const mtMEMORY As Long = -9
' bad file
Public Const mtBAD_FILE As Long = -10
' requested data does not exist
Public Const mtDATA_NOT_FOUND As Long = -11
' too many server session open
Public Const mtTOO_MANY_SESSIONS As Long = -12
' could not perform one or more subs
Public Const mtSUBSTITUTION_ERROR As Long = -13
' could not perform translation
Public Const mtTRANSLATOR_ERROR As Long = -14
' could not set preferences, or invalid preference string
Public Const mtPREFERENCE_ERROR As Long = -15
' bad path (e.g. directory doesn't exist)
Public Const mtBAD_PATH As Long = -16
' file access error (e.g. user doesn't have privileges to write file)
Public Const mtFILE_ACCESS As Long = -17
' file write error (e.g. disk full)
Public Const mtFILE_WRITE_ERROR As Long = -18
' other error
Public Const mtERROR As Long = -9999
```

Note: For additional documentation on MathType SDK constants and #defines, see MathTypeSDK.cs in the sample app MTSDKDN.

### Types

```
'Picture specifier
Public Type MTAPI_PICT
 mm As Long
 xExt As Long
 yExt As Long
 hMF As Long
End Type
Public Type RECT
 left As Long
 top As Long
 right As Long
 bottom As Long
End Type
' Picture dimensions
Public Type MTAPI_DIMS
 baseline As Integer ' distance of baseline from bottom (points)
 bounds As RECT ' bounding rectangle (points)
End Type
```

### MTAPIVersion

```
Public Function MTAPIVersion (
 api As Integer
) As Long
```

Gets version info for a particular API set.

#### Parameters

| *api* | the value of the API that's required (only api = 5 is allowed currently and its version is 5.2) |
| --- | --- |

#### Return value

Returns version or mpMTDLL\_NOT\_FOUND. Version is 0 if API set unknown or hi-byte (of lo-word) = major version, lo-byte (of lo-word) = minor version.

### MTAPIConnect

```
Public Function MTAPIConnect (
 options As Integer,
 timeout As Integer
) As Long
```

This function initializes the API (prepares for use). This function should always be called before any other API function (except MTAPIVersion). Other API functions will return an error if this function has not been called. Calls to MTAPIConnect should always be paired with calls to MTAPIDisconnect, and they may not be nested. Prior to calling MTAPIDisconnect subsequent calls to MTAPIConnect will do nothing and just return mtOK. This function may not be called from MathPage.WLL, use the MathPage.WLL function [MTInitAPI](https://docs.wiris.com/mathtype/en/) instead. Note: This function was formerly named MTInitAPI, and calling it was optional (i.e. if a given call to another API function detected that MTInitAPI had not been called it would call it for you). This is not the case with MTAPIConnect. You must call MTAPIConnect prior to any other API calls (except MTAPIVersion).

#### Parameters

| *options* | mtinitLAUNCH\_NOW => launch MathType server immediately mtinitLAUNCH\_AS\_NEEDED => launch MathType when first needed |
| --- | --- |
| *timeout* | \# of seconds to wait before timing out when attempting to launch MathType. If timeOut = -1 then will never timeout. This value is eventually passed to RPCConnectToServer where it is not currently used |

#### Return value

Returns mtOK, mtMT\_NOT\_FOUND, mtMT\_CANT\_RUN, mtMT\_BAD\_VERSION, mtMT\_IN\_USE, mtMT\_RUN\_TIME\_OUT, OR mtERROR.

### MTAPIDisconnect

```
Public Function MTAPIDisconnect () As Long
```

This function must always be called after MTAPIConnect. It closes connection with MathType and terminates the API (no further usage). This function may not be called from MathPage.WLL, use the MathPage.WLL function [MTTermAPI](https://docs.wiris.com/mathtype/en/) instead.

#### Parameters

None.

#### Return value

Returns mtOK.

### MTEquationOnClipboard

Check for the type of equation on the clipboard, if any.

```
Public Function MTEquationOnClipboard () As Long
```

#### Parameters

None.

#### Return value

If equation on the clipboard, returns type of eqn data: \<code> mtOLE\_EQUATION, mtWMF\_EQUATION, mtMAC\_PICT\_EQUATION,\</code> Otherwise status value: \<code> mtNOT\_EQUATION - no eqn on clipboard mtMEMORY - insufficient memory for prog ID mtERROR - any other error\</code>

### MTClearClipboard

```
Public Function MTClearClipboard () As Long
```

Clears the clipboard contents.

#### Parameters

None.

#### Return value

Returns mtOK.

### MTGetLastDimension

```
Public Function MTGetLastDimension (
 dimIndex As Integer
) As Long
```

Gets a dimension from the last equation copied to the clipboard or written to a file.

#### Parameters

| *dimIndex* | desired dimension. One of the following: mtdimWIDTH, mtdimHEIGHT, mtdimBASELINE, mtdimHORIZ\_POS\_TYPE, mtdimHORIZ\_POS |
| --- | --- |

#### Return value

If successful (>0), value of desired dimension in 32nds of a point.

Otherwise, error status: \<code> mtNOT\_EQUATION - no equation to take dimension from mtERROR - bad value for dimIndex\</code>

### MTOpenFileDialog

```
Public Function MTOpenFileDialog (
 fileType As Integer,
 title As String,
 dir As String,
 file As String,
 fileLen As Integer
) As Long
```

Puts up an open file dialog (Win32 only). Calls GetForegroundWindow for parent, upon which it gets centered.

#### Parameters

| *fileType* | 1 for MT preference files |
| --- | --- |
| *title* | dialog window title |
| *dir* | default directory (may be empty or NULL) |
| *file* | result: new filename |
| *fileLen* | maximum number of characters in filename |

#### Return value

Returns 1 for OK, 0 for Cancel

### MTGetPrefsFromClipboard

```
Public Function MTGetPrefsFromClipboard (
 prefs As String,
 prefsLen As Integer
) As Long
```

Gets equation preferences from the MathType equation currently on the clipboard.

#### Parameters

| *prefs* | \[out\] Preference string (if sizeStr > 0) |
| --- | --- |
| *prefsLen* | \[in\] Size of prefStr (inc. null) or 0 |

#### Return value

If prefsLenr = 0 then this is the size required for prefs. Otherwise it's a status: mtOK Success mtMEMORY Not enough memory for to store preferences mtNOT\_EQUATION Not equation on clipboard mtBAD\_VERSION No preference data found in equation mtERROR Other error

### MTGetPrefsFromFile

```
Public Function MTGetPrefsFromFile (
 prefFile As String,
 prefs As String,
 prefsLen As Integer
) As Long
```

Get equation preferences from the specified preferences file.

#### Parameters

| *prefFile* | \[in\] Pathname for the preference file |
| --- | --- |
| *prefs* | \[out\] Preference string (if sizeStr > 0) |
| *prefsLen* | \[in\] Size of prefStr or 0 |

#### Return value

If sizeStr = 0 then this is the size required for prefStr. Otherwise it's a status: \<code> mtOK Success mtMEMORY Not enough memory for to store preferences mtFile\_NOT\_FOUND File does not exist or bad pathname mtERROR Other error\</code>

### MTConvertPrefsToUIForm

```
Public Function MTConvertPrefsToUIForm (
 inPrefs As String,
 outPrefs As String,
 outPrefsLen As Integer
) As Long
```

Convert internal preferences string to a form to be presented to the user.

#### Parameters

| *inPrefs* | \[in\] internal preferences string |
| --- | --- |
| *outPrefs* | \[out\] Preference string (if sizeStr > 0) |
| *outPrefsLen* | \[in\] Size of outPrefStr (inc. null) or 0 to get length |

#### Return value

If outPrefsLen = 0 then this is the size required for outPrefStr, else it's a status: \<code> mtOK Success mtMEMORY Not enough memory for to store preferences mtERROR Other error\</code>

### MTGetPrefsMTDefault

```
Public Function MTGetPrefsMTDefault (
 prefs As String,
 prefsLen As Integer
) As Long
```

Get MathType's current default equation preferences

#### Parameters

| *prefs* | \[out\] Preference string (if sizeStr > 0) |
| --- | --- |
| *prefsLen* | \[in\] Size of prefStr or 0 |

#### Return value

If prefsLen = 0 then this is the size required for prefs, otherwise it's a status. \<code> mtOK Success mtMEMORY Not enough memory for to store preferences mtERROR Other error\</code>

### MTSetMTPrefs

```
Public Function MTSetMTPrefs (
 mode As Integer,
 prefs As String,
 timeout As Integer
) As Long
```

Set MathType's default preferences for new equations.

#### Parameters

| *mode* | \[in\] Specifies the way the preferences will be applied mtprfMODE\_NEXT\_EQN => Apply to next new equation (see timeOut) mtprfMODE\_MTDEFAULT => Set MathType's defaults for new equations mtprfMODE\_INLINE => makes next eqn inline |
| --- | --- |
| *prefs* | \[in\] Null terminated preference string |
| *timeout* | \[in\] Number of seconds to wait for new equation (used only when mode = 1), Note: -1 means wait forever |

#### Return value

Returns status: \<code> mtOK Success mtBAD\_DATA Bad prefs string, mtERROR Any other error\</code>

### MTGetTranslatorsInfo

```
Public Function MTGetTranslatorsInfo (
 infoIndex As Integer
) As Long
```

Get information about the current set of translators.

#### Parameters

| *infoindex* | \[in\] A flag indicating what info to return: mttrnCOUNT => Total number of translators mttrnMAX\_NAME => Maximum size of any translator name mttrnMAX\_DESC => Maximum size of any translator description string mttrnMAX\_FILE => Maximum size of any translator file name mttrnOPTIONS => Translator options |
| --- | --- |

#### Return value

If >= 0 then this value is the information specified by infoIndex, otherwise its a status: \<code> mtERROR Bad value for infoIndex\</code>

### MTEnumTranslators

```
Public Function MTEnumTranslators (
 index As Integer,
 transName As String,
 transNameLen As Integer,
 transDesc As String,
 transDescLen As Integer,
 transFile As String,
 transFileLen As Integer
) As Long
```

Enumerate the available equation (TeX, etc.) translators.

#### Parameters

| *index* | \[in\] Index of the translator to enumerate (must be initialized to 1 by the caller) |
| --- | --- |
| *transName* | \[out\] Translator name |
| *transNameLen* | \[in\] Size of tShort. (May be set to zero) |
| *transDesc* | \[out\] Translator descriptor string |
| *transDescLen* | \[in\] Size of transDesc. (May be set to zero) |
| *transFile* | \[out\] Translator file name |
| *transFileLen* | \[in\] Size of transFile. (May be set to zero) |

#### Return value

If >0 then this value is the index of next translator to enumerate, (i.e. the caller should pass this value in for index). Otherwise, a status: \<code> mtOK Success (no more translators in the list) mtMEMORY Not enough room in transName, transDesc, or transFile mtERROR Any other failure\</code>

### MTXFormReset

```
Public Function MTXFormReset () As Long
```

Resets to default options for MTXFormEqn (i.e. no substitutions, no translation, and use existing preferences).

#### Parameters

None.

#### Return value

Return mtOK.

### MTXFormAddVarSub

```
Public Function MTXFormAddVarSub (
 options As Integer,
 findType As Integer,
 find As String,
 findLen As Long,
 replaceType As Integer,
 replace As String,
 replaceLen As Long,
 replaceStyle As Integer
) As Long
```

Adds a variable substitution to be performed with next MTXFormEqn (may be called 0 or more times).

#### Parameters

| *options* | mtxfmSUBST\_ALL or mtxfmSUBST\_ONE |
| --- | --- |
| *findType* | type of data in find arg (must be mtxfmVAR\_SUB\_PLAIN\_TEXT for now) |
| *find* | equation text to be found and replaced (null-terminated text string for now) |
| *findLen* | length of find arg data (ignored for now) |
| *replaceType* | type of data in replace arg: mtxfmVAR\_SUB\_PLAIN\_TEXT mtxfmVAR\_SUB\_MTEF\_TEXT mtxfmVAR\_SUB\_MTEF\_BINARY mtxfmVAR\_SUB\_DELETE - delete the "find" text |
| *replace* | equation text to replace "find" arg with |
| *replaceLen* | if replaceType = mtxfmVAR\_SUB\_MTEF\_BINARY, length of replace arg data |
| //replaceStyle | if replaceType = mtxfmVAR\_SUB\_PLAIN\_TEXT, style of replacement text: mtxfmSTYLE\_TEXT mtxfmSTYLE\_FUNCTION mtxfmSTYLE\_VARIABLE mtxfmSTYLE\_LCGREEK mtxfmSTYLE\_UCGREEK mtxfmSTYLE\_SYMBOL mtxfmSTYLE\_VECTOR mtxfmSTYLE\_NUMBER |

#### Return value

Returns status: \<code> mtOK - success mtERROR - some other error\</code>

### MTXFormSetTranslator

```
Public Function MTXFormSetTranslator (
 options As Integer,
 transName As String
) As Long
```

Specify translation to be performed with the next MTXFormEqn.

#### Parameters

| *options\|\[in\] One or more (OR'd together) of: mtxfmTRANSL\_INC\_NONE - no options mtxfmTRANSL\_INC\_NAME - include the translator's name in translator output mtxfmTRANSL\_INC\_DATA - include MathType equation data in translator output mtxfmTRANSL\_INC\_MTDEFAULT use MathType's defaults mtxfmTRANSL\_INC\_CLIPBOARD\_EXTRA\| \|* transName// | \[in\] File name of translator to be used, NULL for no translation |
| --- | --- |

#### Return value

Returns status: \<code> mtOK - success mtFILE\_NOT\_FOUND - could not find translator mtTRANSLATOR\_ERROR - errors compiling translator mtERROR - some other error\</code>

### MTXFormSetPrefs

```
Public Function MTXFormSetPrefs (
 prefType As Integer,
 prefStr As String
) As Long
```

Specify a new set of preferences to be used with the next MTXFormEqn.

#### Parameters

| *prefType* | \[in\] One of the following: mtxfmPREF\_EXISTING - use existing preferences mtxfmPREF\_MTDEFAULT - use MathType's default preferences mtxfmPREF\_USER - use specified preferences |
| --- | --- |
| *prefStr* | \[in\] Preferences to apply (mtxfmPREF\_USER) |

#### Return value

Returns mtOK or mtERROR.

### MTXFormEqn

```
Public Function MTXFormEqn (
 src As Integer,
 srcFmt As Integer,
 srcData As String,
 srcDataLen As Long,
 dst As Integer,
 dstFmt As Integer,
 dstData As String,
 dstDataLen As Long,
 dstPath As String,
 dims As MTAPI_DIMS
) As Long
```

Transforms an equation (uses options specified via MTXFormAddVarSub, MTXFormSetTranslator, and MTXFormSetPrefs)

Note: Variations involving mtxfm\_PICT or dstFmt=mtxfm\_HMTEF are not callable from VBA.

#### Parameters

| *src* | \[in\] Equation data source, one of: mtxfmPREVIOUS => data from previous result mtxfmCLIPBOARD => data on clipboard mtxfmLOCAL => data passed (i.e. in srcData) |
| --- | --- |
| *srcFmt* | \[in\] Equation source data format, one of: mtxfmMTEF mtxfmPICT mtxfmTEXT Note: srcFmt, srcData, and srcDataLen are used only if src is mtxfmLOCAL |
| *srcData* | \[in\] Depends on data source (srcFmt) if srcFmt is mtxfmMTEF, then srcData must point to MTEF-binary (BYTE \*) data if srcFmt is mtxfmPICT, then srcData must point to PICT (MTAPI\_PICT \*) data if srcFmt is mtxfmTEXT, then srcData must point to either MTEF-text or plain text (CHAR \*) data |
| *srcDataLen* | \[in\] # of bytes in srcData |
| *dst* | \[in\] Equation data destination, one of: mtxfmCLIPBOARD => transformed data placed on clipboard mtxfmFILE => transformed data in the file specified by dstPath mtxfmLOCAL => transformed data in dstData |
| *dstFmt* | \[in\] Equation data format, one of: mtxfmMTEF mtxfmHMTEF mtxfmPICT mtxfmGIF mtxfmTEXT mtxfm HTEXT Note: dstFmt, dstData, and dstDataLen are used only if dst is mtxfmLOCAL (The data is placed on the clipboard in either an OLE object or translator text) or mtxfmFILE (dstFmt specifies the file format, dstPath specifies the name of the file to create). |
| *dstData* | \[out\] Depends on data destination (dstFmt) if dstFmt is mtxfmMTEF, then dstData points to MTEF-binary data if dstFmt is mtxfmHMTEF, then dstData is a handle to MTEF-binary data if dstFmt is mtxfmPICT, then dstData points to PICT data if dstFmt is mtxfmGIF, then dstData points to GIF data if dstFmt is mtxfmTEXT, then dstData points to translated text or, if no translator, MTEF-text data if dstFmt is mtxfmHTEXT, then dstData is a handle to translated text or, if no translator, MTEF-text data Note: If a translator was specified, dstFmt must be either mtxfmTEXT or mtxfmHTEXT for the translation to be performed. |
| *dstDataLen* | \[in\] # of bytes in dstData (used for mtxfmLOCAL only) |
| *dstPath* | \[in\] destination pathname (used if dst == mtxfmFILE only, may be NULL if not used) |
| *dims* | \[out\] pict dimensions, may be NULL (valid only for dst = mtxfmPICT). See MTAPI\_DIMS definition above in Types section. |

#### Notes:

1. if src is mtfxmLOCAL, then srcFmt, srcData, and srcDataLen must be supplied
2. if dst is mtfxmLOCAL, then dstFmt, dstData, and dstDataLen must be supplied
3. one can convert from MTEF text to various file types by passing these parameters:
	- src: mtxfmCLIPBOARD
		- srcFmt: mtxfmTEXT
		- dst: mtxfmFILE
		- dstFmt: one of: mtxfmPICT (wmf), mtxfmGIF (gif), mtxfmEPS\_NONE (eps), mtxfmEPS\_WMF (eps), mtxfmEPS\_TIFF (eps)
		- dstPath: (path and file name for output file)
		- all other parameters are ignored
4. The SDK sample application, ConvertEquations, demonstrates how to call MTXFormEqn for each of the supported combinations of source format and type, and destination format and type. The table in note #5 below also shows all of the supported combinations.
5. The following table shows the various supported conversion options in terms of where the data is to be converted from/to and what formats are supported:

| ::: | ::: | Destination |  |  |  |  |  |  |  |  |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ::: | ::: | clipboard | file | local |  |  |  |  |  |  |
| Source | format | text | MTEF, EPS, or GIF | WMF | Emb. Obj. | text or MTEF | EPS, GIF, or WMF | Emb. Obj. | text or MTEF | EPS, GIF, WMF, or Emb. Obj. |
| clipboard | text or MTEF | Yes |  | Yes |  |  | Yes |  | Yes |  |
| ::: | EPS, GIF, or WMF |  |  |  |  |  |  |  |  |  |
| ::: | Emb. Obj. | Yes |  | Yes |  |  | Yes |  | Yes |  |
| file | text or MTEF |  |  |  |  |  |  |  |  |  |
| ::: | EPS, GIF, or WMF | Yes |  | Yes |  |  | Yes |  | Yes |  |
| ::: | Emb. Obj. |  |  |  |  |  |  |  |  |  |
| local | text or MTEF | Yes |  |  |  |  | Yes |  | Yes |  |
| ::: | EPS, GIF, WMF, or Emb. Obj. |  |  |  |  |  |  |  |  |  |

#### Return value

Returns status: \<code> mtOK - success mtNOT\_EQUATION - source data does not contain MTEF mtSUBSTITUTION\_ERROR - could not perform one or more subs mtTRANSLATOR\_ERROR - errors occured during translation (translation not done) mtPREFERENCE\_ERROR - could not set perferences mtMEMORY - not enough space in dstData mtERROR - some other error\</code>

### MTXFormGetStatus

```
Public Function MTXFormGetStatus (
 index As Integer
) As Long
```

Check error/status after MTXFormEqn.

#### Parameters

| *index* | \[in\] which status to get; described below: mtxfmSTAT\_PREF, status for set preferences mtxfmSTAT\_TRANSL, status for translation mtxfmSTAT\_ACTUAL\_LEN ≥1, status of the i=th (i=index) variable substitution |
| --- | --- |

#### Return value

Depends on the value of 'index', as follows:

```
If index = mtxfmSTAT_PREF, status for set preferences:
 mtOK - success setting preferences
 mtBAD_DATA - bad preference data
If index = mtxfmSTAT_TRANSL, status for translation:
 mtOK - successful translation
 mtFILE_NOT_FOUND - could not find translator
 mtBAD_FILE - file found was not a translator
If index = mtxfmSTAT_ACTUAL_LEN, number of bytes of data actually returned in dstData (if MTXformEqn succeeded) or the number of bytes required (if MTXformEqn returned
 mtMEMORY - not enough memory was specified for dstData), otherwise 0L.
If index ≥1, status of the i-th (i = index) variable substitution, either # of times the substitution was performed, or, if < 0, an error status.
```

NOTE: returns mtERROR for bad values of index

### MTPreviewDialog

```
Public Function MTPreviewDialog (
 parent As Long,
 title As String,
 prefs As String,
 closeBtnText As String,
 helpBtnText As String,
 helpID As Long,
 helpFile As String
) As Long
```

Puts up a preview dialog for displaying preferences

#### Parameters

| *parent* | parent window |
| --- | --- |
| *title* | dialog title |
| *prefs* | text to preview |
| *closeBtnText* | text for Close button (can be NULL for English) |
| *helpBtnText* | text for Help button (can be NULL for English) |
| *helpID* | help topic ID |
| *helpFile* | help file |

#### Return value

Returns 0 if successful, non-zero if error.

### MTGetPathToMathType

```
Public Function MTGetPathToMathType (
 path As String,
 pathMax As Long
) As Long
```

Get the path to the MathType folder.

#### Parameters

| *path* | \[out\] Fully qualified path to the MathType folder |
| --- | --- |
| *pahMax* | \[in/out\] Max size of path on input; actual length on output |

#### Return value

Returns mtOK on success, otherwise mtERROR.