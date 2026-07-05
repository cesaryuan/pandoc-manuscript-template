Attribute VB_Name = "Declarations"

'Declarations.bas
'bit independent declarations

'====================================================================
' (c) Copyright 1992-2015 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/Declarations.bas 9     12/02/15 4:39p Johns $
'====================================================================

' Note that this module is also shared between Word and PowerPoint

Option Explicit

'name of commands project
Public Const kMTCommands As String = "MTCommandsMain"

Public Const kAppMSW As String = "Microsoft Word"
Public Const kAppMSPP As String = "Microsoft PowerPoint"

'major version IDs of Word - Windows
Public Const kWord97 As Long = 8
Public Const kWord2000 As Long = 9
Public Const kWordX As Long = 10
Public Const kWord2003 As Long = 11
Public Const kWord2007 As Long = 12
Public Const kWord2010 As Long = 14
Public Const kWord2013 As Long = 15
Public Const kWord2016 As Long = 16
Public Const kWord2003SP3MinorVersion As Long = 8169
Public Const kWord2007SP2MinorVersion As Long = 6400

'major version IDs of Word - Mac
Public Const kWord2004 As Long = 11
Public Const kWord2008 As Long = 12
Public Const kWord2011 As Long = 14

'major version IDs of PPT - Windows
Public Const kPP2003 As Long = 11
Public Const kPP2007 As Long = 12
Public Const kPP2010 As Long = 14
Public Const kPP2013 As Long = 15
Public Const kPP2016 As Long = 16

'major version IDs of PPT - Mac
Public Const kPP2004 As Long = 11
Public Const kPP2008 As Long = 12
Public Const kPP2011 As Long = 14

'version # of the MathType API
Public Const MTAPI_VERSION As Integer = 5

' used with the return value from MTGetAppFunctionality
#If Win32 Then
Enum DemoMode
    dmNone = 0  ' no mode at all (completely fresh install)
    dmDemo      'full functionality, but with expiration date
    dmFull      ' full functionality, registered, no expiration
    dmExpired   ' reduced functionality, demo has expired
    dmCancelled ' reduced functionality, reg num has been cancelled by reg check
End Enum
#End If

'----------- Numbers we compare against with MTAPIvers ----------
Public Const mtversMajVerHi As Integer = 1535   '0x05ff
Public Const mtversMajVerLo As Integer = 1280   '0x0500
Public Const mtversMinVer As Integer = 1280     '0x0500

'Custom Properties
'Boolean, indicates type of equations in doc
Public Const mtprop_HAS_MAC_EQNS As String = "MTMacEqns"
Public Const mtprop_HAS_WIN_EQNS As String = "MTWinEqns"
Public Const mtprop_EQN_NUMS_ON_RIGHT As String = "MTEqnNumsOnRight"

'MT registry name
#If Win32 Then
Public Const mt_REGISTRY As String = ""             'use Windows registry
#Else
Public Const mt_REGISTRY As String = "MathType6"    'use Mac file
#End If

'Registry Locations
Public Const HKEY_CLASSES_ROOT = &H80000000
Public Const HKEY_CURRENT_USER = &H80000001
Public Const HKEY_LOCAL_MACHINE = &H80000002
Public Const HKEY_USERS = &H80000003
Public Const HKEY_PERFORMANCE_DATA = &H80000004
Public Const HKEY_CURRENT_CONFIG = &H80000005
Public Const HKEY_DYN_DATA = &H80000006

Public Const mtreg_HKLM = "HKEY_LOCAL_MACHINE"
Public Const mtreg_HKCU = "HKEY_CURRENT_USER"

Public Const mtreg_MT_HKLM_HOME = "SOFTWARE\Design Science\DSMT7"
Public Const mtreg_MT_HKCU_HOME = "Software\Design Science\DSMT7"
Public Const mtreg_MT_HKLM_DIRECTORIES = mtreg_MT_HKLM_HOME & "\Directories"
Public Const mtreg_MT_DIRECTORIES_LOCATION As String = "Software\Design Science\DSMT7\Directories"

Public Const mtreg_MT_ASK_ON_PASTE As String = "AskOnPaste"
Public Const mtreg_MT_EQN_NUMS_ON_RIGHT_KEY As String = "EqnNumsOnRight"
Public Const mtreg_MT_HELPDIR_KEY As String = "HelpDir"
Public Const mtreg_MT_HELPFILE_KEY As String = "HelpFile"
Public Const mtreg_MT_HELPFILE_LOCATION As String = "Software\Design Science\DSMT7\Config"
Public Const mtreg_MT_LANGUAGEDIR_KEY As String = "LangDir"             'language support files directory
Public Const mtreg_MT_MATHPAGE_KEY As String = mtreg_MT_HKCU_HOME & "\MathPage"
Public Const mtreg_MT_PASTECANCELDEF As String = "PasteCancelDefault"   'Continue default behavior for pasting
Public Const mtreg_MT_PREFDIR_KEY As String = "PrefsDir"                'system directory
Public Const mtreg_MT_PROGDIR_KEY As String = "ProgDir"                 'app directory
Public Const mtreg_MT_VERBOSE_LOGGING_KEY As String = "VerboseLogging"
Public Const mtreg_MT_WORDCMDS_LOCATION As String = mtreg_MT_HKCU_HOME & "\WordCommands"
Public Const mtreg_MT_WORD_2003_OMML_IN_RTF As String = "OMMLinRTF"     'if non-zero look in RTF instead of PNG in Word 2003
Public Const mtreg_MT_WORD_CONVFROM As String = "ConvertFrom"           'ConvertFrom key
Public Const mtreg_MT_WORD_CONVMISC As String = "ConvertMisc"           'ConvertMisc key
Public Const mtreg_MT_WORD_CONVTO As String = "ConvertTo"               'ConvertTo key
Public Const mtreg_MT_WORD_CONVTRANS As String = "ConvertTranslator"    'ConvertTranslator key
Public Const mtreg_MT_WORD_DEFAULT_EQNNUM_CUSTOM As String = "DefaultEqnNumCustom"      'Default eqn num. format
Public Const mtreg_MT_WORD_DEFAULT_EQNNUM_FORMAT As String = "DefaultEqnNumFormat"      'Default eqn num. format
Public Const mtreg_MT_WORD_DONTSHOW_EQNNUM_WARNING As String = "NoEqnNumWarningDlg"     'Don't Show EqnNum's 'Insert Break?' dialog key
Public Const mtreg_MT_WORD_DONTSHOW_EQNREFDLG As String = "NoInsertEqnRefDlg"           'Don't Show Insert Eqn Ref dialog key
Public Const mtreg_MT_WORD_DONTSHOW_LANGDLLERROR As String = "NoLanguageDLLError"       'Don't Show Missing Lang DLL key
Public Const mtreg_MT_WORD_DONTSHOW_SLOWEQNUPDATE As String = "NoSlowUpdateEqnDlg"      'Don't Show 'Slow EqnNum updates' key
Public Const mtreg_MT_WORD_EXPORT_DIRECTORY As String = "ExportDir"     'Default export dir
Public Const mtreg_MT_WORD_EXPORT_FILETYPE As String = "ExportFileType" 'Default export filetype
Public Const mtreg_MT_WORD_EXPORT_PATTERN As String = "ExportPattern"   'Default pattern
Public Const mtreg_MT_WORD_EXPORT_REPLACE As String = "ExportReplace"   'Replace?
Public Const mtreg_MT_WORD_MATHML_PASTEAS As String = "MMLPasteAs"
Public Const mtreg_MT_WORD_NOAPPLY_CONVERTED_EQN_STYLE As String = "ConvertEqnNoStyle" 'If set does not apply the MTConvertedEquation Style to converted equations
Public Const mtreg_MT_WORD_NO_SPACE_AFTER_INLINE As String = "NoSpaceAfterInline"
Public Const mtreg_MT_WORD_NO_CHECK_PROG_ID As String = "NoCheckProgID" 'MT-3373
Public Const mtreg_MT_WORD_OMML2MML_XSL_DIRNAME As String = "OMML2MMLXSL_DIR"
Public Const mtreg_MT_WORD_OMML2MML_XSL_FILENAME As String = "OMML2MMLXSL_FILE"
Public Const mtreg_MT_WORD_SEEN_MATHMLHELP As String = "SeenMathMLHelp" '"1" if MathMLHelp tip has been shown
Public Const mtreg_MT_CONVERTEQNS_DELAY As String = "ConvertEquationDelay"  ' amount of time to delay in milliseconds after copy to clipboard in convert equations

Public Const msoreg_WORD2007DIR_SECTION As String = "Software\Microsoft\Office\12.0\Word\Options\"
Public Const msoreg_WORD2010DIR_SECTION As String = "Software\Microsoft\Office\14.0\Word\Options\"
Public Const msoreg_WORD2013DIR_SECTION As String = "Software\Microsoft\Office\15.0\Word\Options\"
Public Const msoreg_WORD2016DIR_SECTION As String = "Software\Microsoft\Office\16.0\Word\Options\"
Public Const msoreg_WORDDIR_KEY As String = "PROGRAMDIR"

Public Const msoreg_WORD2007DIR_SECTION2 As String = "Software\Microsoft\Office\12.0\Word\InstallRoot\"
Public Const msoreg_WORD2010DIR_SECTION2 As String = "Software\Microsoft\Office\14.0\Word\InstallRoot\"
Public Const msoreg_WORD2013DIR_SECTION2 As String = "Software\Microsoft\Office\15.0\Word\InstallRoot\"
Public Const msoreg_WORD2016DIR_SECTION2 As String = "Software\Microsoft\Office\16.0\Word\InstallRoot\"
Public Const msoreg_WORDDIR_KEY2 As String = "Path"

' the following declarations are used for identifying buttons in the UI
Public Const mtbIDInsDispMTEqn As String = "MathType_B_InsDispMTEqn"
Public Const mtbIDInsDispMTEqn3 As String = "MathType_B_InsDispMTEqn3"
Public Const mtbIDInsDispMTEqnMac As String = "MathType_B_InsDispMTEqnMac"
Public Const mtbIDInsDispMTEqnLeftNum As String = "MathType_B_InsDispMTEqnLeftNum"
Public Const mtbIDInsDispMTEqnLeftNum2 As String = "MathType_B_InsDispMTEqnLeftNum2"
Public Const mtbIDInsDispMTEqnLeftNum3 As String = "MathType_B_InsDispMTEqnLeftNum3"
Public Const mtbIDInsDispMTEqnLeftNumMac As String = "MathType_B_InsDispMTEqnLeftNumMac"
Public Const mtbIDInsDispMTEqnRightNum As String = "MathType_B_InsDispMTEqnRightNum"
Public Const mtbIDInsDispMTEqnRightNum2 As String = "MathType_B_InsDispMTEqnRightNum2"
Public Const mtbIDInsDispMTEqnRightNum3 As String = "MathType_B_InsDispMTEqnRightNum3"
Public Const mtbIDInsDispMTEqnRightNumMac As String = "MathType_B_InsDispMTEqnRightNumMac"
Public Const mtbIDInsEBEqn2 As String = "MathType_B_InsEBEqn2"
Public Const mtbIDInsHandEqn As String = "MathType_B_InsHandEqn"
Public Const mtbIDInsInlineMTEqn As String = "MathType_B_InsInlineMTEqn"
Public Const mtbIDInsInlineMTEqn3 As String = "MathType_B_InsInlineMTEqn3"
Public Const mtbIDInsInlineMTEqnMac As String = "MathType_B_InsInlineMTEqnMac"
Public Const mtbIDSpeak As String = "MathType_B_Speak"
Public Const mtbIDSpeak3 As String = "MathType_B_Speak3"

'Public Const mtbIDMathType_B_InsEBEqn As String = "MathType_B_InsEBEqn" ' replaced by O2007 implementation
Public Const mtbIDInsEBEqnLeftNum As String = "MathType_B_InsEBEqnLeftNum"
Public Const mtbIDInsEBEqnRightNum As String = "MathType_B_InsEBEqnRightNum"

Public Const mtbIDInsertNumber As String = "MathType_B_InsertNumber"

Public Const mtbIDFormatEqnNums As String = "MathType_B_FormatEqnNums"
Public Const mtbIDUpdateEqnNums As String = "MathType_B_UpdateEqnNums"

Public Const mtbIDEquationReference As String = "MathType_B_EquationReference"

Public Const mtbIDManageChapterSections As String = "MathType_M_EqnNumsChapsSects"

Public Const mtbIDInsertNextSection As String = "MathType_B_InsertNextSection"
Public Const mtbIDInsertNextChapter As String = "MathType_B_InsertNextChapter"
Public Const mtbIDMoreBreaks As String = "MathType_B_MoreBreaks"
Public Const mtbIDModifyBreak As String = "MathType_B_ModifyBreak"

Public Const mtbIDBrowsePrev As String = "MathType_B_BrowsePrev"
Public Const mtbIDBrowseType As String = "MathType_DD_BrowseType"
Public Const mtbIDBrowseNext As String = "MathType_B_BrowseNext"

Public Const mtbIDSetEqnPrefs As String = "MathType_B_SetEqnPrefs"
Public Const mtbIDFormatEqns As String = "MathType_B_FormatEqns"
Public Const mtbIDConvertEqns As String = "MathType_B_ConvertEqns"
Public Const mtbIDTeXToggle As String = "MathType_B_TeXToggle"
Public Const mtbIDExportEqns As String = "MathType_B_ExportEqns"
Public Const mtbIDMathPage As String = "MathType_B_MathPage"

Public Const mtbIDHelp As String = "MathType_B_Help"
Public Const mtbIDHelpContents As String = "MathType_B_HelpContents"
Public Const mtbIDHelpMTInWord As String = "MathType_B_HelpMTInWord"
Public Const mtbIDHelpUnlockReg As String = "MathType_B_HelpUnlockReg"
Public Const mtbIDHelpAboutMT As String = "MathType_B_HelpAboutMT"

Public Const mtbIDHelpContentsMac As String = "MathType_B_HelpContents_Mac"
Public Const mtbIDHelpMTInWordMac As String = "MathType_B_HelpMTInWord_Mac"
Public Const mtbIDHelpUnlockRegMac As String = "MathType_B_HelpUnlockReg_Mac"
Public Const mtbIDHelpAboutMTMac As String = "MathType_B_HelpAboutMT_Mac"

Public Const mtbIDWeb As String = "MathType_B_Web"
Public Const mtbIDWebHomePage As String = "MathType_B_WebHomePage"
Public Const mtbIDWebSupport As String = "MathType_B_WebSupport"
Public Const mtbIDWebEmailFeedback As String = "MathType_B_WebEmailFeedback"
Public Const mtbIDWebOrderMT As String = "MathType_B_WebOrderMT"
Public Const mtbIDFutureMT As String = "MathType_B_FutureMT"
Public Const mtbIDMTOptions As String = "MathType_DL_MTOptions"

Public Const mtbIDWebEmailFeedbackMac As String = "MathType_B_WebEmailFeedback_Mac"
Public Const mtbIDWebSupportMac As String = "MathType_B_WebSupport_Mac"
Public Const mtbIDWebOrderMTMac As String = "MathType_B_WebOrderMT_Mac"
Public Const mtbIDFutureMTMac As String = "MathType_B_FutureMT_Mac"
Public Const mtbIDMTOptionsMac As String = "MathType_MTOptions_Mac"

Public Const mtbIDOMMathPage As String = "MathType_B_OMMathPage"

#If Word Then
Public Enum States
    AlwaysEnable = 1
    Word97SelectionInFootnoteEndnotePane = 2
    Word97SelectionInCommentPane = 4
    Word97SelectionInHeaderFooter = 8
    WordXPActiveWindowViewSplitSpecial = 16
    NotInReadingView = 32
    MTReferenceExists = 64
    NotInUnsupportedView = 128
    MathPageOK = 256
    IsDocumentOpen = 512
    SelectionInTextBox = 1024
    IsFunctionalityOK = 2048
End Enum
#ElseIf PP Then
    #If Win32 Then
    Public Enum States
        AlwaysEnable = 1
        IsPresentationOpen = 2
        ViewIsNotSlideSorter = 4
        ViewIsNotOutline = 8
        BrowseOK = 16
        NotInUnsupportedView = 32
    End Enum
    #Else ' no enum support on Mac PPT
    Public Const AlwaysEnable As Long = 1
    Public Const IsPresentationOpen As Long = 2
    Public Const ViewIsNotSlideSorter As Long = 4
    Public Const ViewIsNotOutline As Long = 8
    Public Const BrowseOK As Long = 16
    Public Const NotInUnsupportedView As Long = 32
    #End If
#End If

' major version # of this API
Public Const MPAPI_VERSION As Integer = 5

'----------- Numbers we compare against with MTAPIvers ----------
'MathType 5.01 uses the MathPage 5.1 API
Public Const mpversMajVerHi As Integer = 1535   '0x05ff
Public Const mpversMajVerLo As Integer = 1281   '0x0501
Public Const mpversMinVer As Integer = 1281     '0x0501

'API Return codes
Public Const mpOK As Long = 0
Public Const mpEQN_NO_BASELINE = 1000
Public Const mpBAD_VERSION As Long = -1000
Public Const mpMTDLL_NOT_FOUND As Long = -1001
Public Const mpFILE_INVALID As Long = -1002
Public Const mpFILE_NO_ACCESS As Long = -1003
Public Const mpNOT_AN_EQUATION As Long = -1004
Public Const mpERROR As Long = -9999

'Flag values for MPDocInit
Public Const mpdFullCompatibility As Integer = 1
Public Const mpdGenMathML As Integer = 2
Public Const mpdMathZoom As Integer = 4

'style values input to MPProcessSymbol
Public Const kPlainText As Integer = 0
Public Const kBoldText As Integer = 1
Public Const kItalicText As Integer = 2

'kind values, also returned by MPAnalyzeSymbol
Public Const kSIDefault As Long = 0                 'default value
Public Const kSIGIF As Long = 1                     'needs a GIF
Public Const kSIEntity As Long = 2                  'needs entity
Public Const kSIInsertSymbolPlaceholder As Long = 3 'maybe an inserted symbol
Public Const kSISubstituted As Long = 4             'char is being changed, 0 = delete char
Public Const kSIMissingFont As Long = 5             'font not installed

'Constants for 'cur sel' or 'whole doc'
Public Const mt_RANGE_DOCUMENT = 0
Public Const mt_RANGE_SELECTION = 1

'Flag bit for MTLib.SaveWordState()
Public Const mt_SWS_TRACKCHANGES = 1
Public Const mt_SWS_SMART_CUTPASTE = 2
Public Const mt_SWS_TYPING_REPLACE_SELECTION = 4
Public Const mt_SWS_PASTE_SMART_CUTPASTE = 8

'URL codes - also see UIHelp.bas mturl's
Public Const mturlMATHTYPE_OMML2MATHMLXSL As Long = 7

'Default OMML XSL file name
Public Const kOMML2MML_XSL_FILENAME As String = "OMML2MML.XSL"

'Word exe name
Public Const kWORDEXENAME = "WINWORD.EXE"

'MTMsgBox Constants
Public Const mt_MBYESNO = 1
Public Const mt_MBYESNOCANCEL = 2
Public Const mt_MBYES = 1
Public Const mt_MBNO = 2
Public Const mt_MBCANCEL = 3

' file type constants
Public Const kFTEPS_OSPICT As Integer = 0   'EPS+WMF, EPS+PICT
Public Const kFTEPS_NONE As Integer = 1     'EPS/None
Public Const kFTEPS_TIFF As Integer = 2     'EPS/TIFF (Win only)
Public Const kFTGIF As Integer = 3          'GIF
Public Const kFTOSPICT As Integer = 4       'WMF, PICT
Public Const kFTPDF As Integer = 5          'PDF

'Values for MTLib.GetPlatform()
Public Const kPlatformMac As Long = 1
Public Const kPlatformWin As Long = 2

' Predefined Clipboard Formats copied from C:\Program Files\Microsoft Visual Studio\Common\Tools\Winapi\WIN32API.TXT
Public Const CF_TEXT As Integer = 1
Public Const CF_BITMAP As Integer = 2
Public Const CF_METAFILEPICT As Integer = 3
Public Const CF_SYLK As Integer = 4
Public Const CF_DIF As Integer = 5
Public Const CF_TIFF As Integer = 6
Public Const CF_OEMTEXT As Integer = 7
Public Const CF_DIB As Integer = 8
Public Const CF_PALETTE As Integer = 9
Public Const CF_PENDATA As Integer = 10
Public Const CF_RIFF As Integer = 11
Public Const CF_WAVE As Integer = 12
Public Const CF_UNICODETEXT As Integer = 13
Public Const CF_ENHMETAFILE As Integer = 14

Public Const HWND_TOP = 0
Public Const SWP_SHOWWINDOW = &H40

' language type values for MTSetEqnFromLangStr
Public Const mtlangTEX_INPUT As Integer = 1
Public Const mtlangMATHML As Integer = 2
Public Const mtlangMTEF As Integer = 3

#If Win32 Then
'SHGetSpecialFolderPath parameter
Public Enum mceIDLPaths
    CSIDL_PROGRAM_FILES = &H26 ' * CSIDL_PROGRAM_FILES - Version 5.0. Program Files folder. A common path is C:\Program Files.
End Enum
#End If

Type SymbolInfo
    kind As Integer         'out from MPFindSymbol, input to MPProcessSymbol (see above)
    charCode As Integer     'out from MPFindSymbol, input to MPProcessSymbol
    isUniCode As Boolean    'private to WLL
    size As Single
    style As Integer        'see style constants above
    color As Long           'RGB value
End Type

Type BorderInfo
    width As Long           'one of wdLineWidth
    style As Long           'one of wdLineStyle
    color As Long           'RGB value
End Type

'display values for GIFInfo.display
Public Const mpdtInternal As Long = 0               'use internal inline/display setting
Public Const mpdtInline As Long = 1                 'make inline
Public Const mpdtDisplay As Long = 2                'make display

Type GIFInfo
    smooth As Boolean        'True to use anti-aliasing
    bkgndColor As Long       'only needed if (smooth || ! transparent)
    size As Single           'size of surrounding text
    fillType As Long         'one of msoFillType or 0
    topBorder As BorderInfo
    leftBorder As BorderInfo
    bottomBorder As BorderInfo
    rightBorder As BorderInfo
End Type

Type GIFInfo2
    version As Integer       'version of struct/API; 1 = MTW 5.0a
    smooth As Boolean        'True to use anti-aliasing
    bkgndColor As Long       'only needed if (smooth || ! transparent)
    size As Single           'size of surrounding text
    fillType As Long         'one of msoFillType or 0
    topBorder As BorderInfo
    leftBorder As BorderInfo
    bottomBorder As BorderInfo
    rightBorder As BorderInfo
    display As Long         'one of mpdtXXX constants
End Type

'struct for MPEnumMathMLTarget2
Type MPMathMLTarget
    browser As Integer      '0-based compatibility choice, -1 = all
    minWordVer As Integer   'min version of Word that supports target (8,9,10...)
    nameMax As Integer      'max len of name
    targetName As String    'MathML target name
    descMax As Integer      'max len of description
    targetDesc As String    'MathML target description
End Type

Type MPMathMLTarget2
    version As Integer      'struct/API version; MTW5.0a = 1
    browser As Integer      '0-based compatibility choice, -1 = all
    minWordVer As Integer   'min version of Word that supports target (8,9,10...)
    nameMax As Integer      'max len of name
    targetName As String    'MathML target name
    descMax As Integer      'max len of description
    targetDesc As String    'MathML target description
    extMax As Integer       'max len of extensions
    extensions As String    'valid file extensions, first is default
End Type

' maximum length of file paths, names, etc.
Public Const MTAPI_MAX_PATH As Long = 260

' Picture specifier
Public Type MTAPI_PICT
   mm    As Long
   xExt  As Long
   yExt  As Long
   hMF   As Long
End Type

Public Type RECT
   left     As Long
   top      As Long
   right    As Long
   bottom   As Long
End Type

' Picture dimensions
Public Type MTAPI_DIMS
   baseline As Integer  ' dist of baseline from bottom (points)
   bounds   As RECT     ' bounding rectangle (points)
End Type

'Info about chap/sec break
#If Word Then
Public Type BreakInfo
    parentField As Field                'Enclosing Macrobutton Field
    hasChapter As Boolean               'True if chapter field present
    isExplicitChapterNumber As Boolean  'True if chap# is explicit
    chapterNumber As String             'Explicit chap#'s value
    isExplicitSectionNumber As Boolean  'True if sec# is explicit
    sectionNumber As String             'Explicit sec#'s value
End Type
#Else
' Field is undefined in PowerPoint
Public Type BreakInfo
    'parentField As Field                'Enclosing Macrobutton Field
    hasChapter As Boolean               'True if chapter field present
    isExplicitChapterNumber As Boolean  'True if chap# is explicit
    chapterNumber As String             'Explicit chap#'s value
    isExplicitSectionNumber As Boolean  'True if sec# is explicit
    sectionNumber As String             'Explicit sec#'s value
End Type
#End If

'GetFileVersionInfo Declarations
Type VS_FIXEDFILEINFO
   dwSignature As Long
   dwStrucVersionl As Integer
   dwStrucVersionh As Integer
   dwFileVersionMSl As Integer
   dwFileVersionMSh As Integer
   dwFileVersionLSl As Integer
   dwFileVersionLSh As Integer
   dwProductVersionMSl As Integer
   dwProductVersionMSh As Integer
   dwProductVersionLSl As Integer
   dwProductVersionLSh As Integer
   dwFileFlagsMask As Long
   dwFileFlags As Long
   dwFileOS As Long
   dwFileType As Long
   dwFileSubtype As Long
   dwFileDateMS As Long
   dwFileDateLS As Long
End Type

' return codes from MT DLL API

' success, no error
Public Const mtOK As Long = 0
' equation OLE 1.0 object on clipboard
Public Const mtOLE_EQUATION As Long = 1
' Windows metafile equation graphic (not OLE object) on clipboard
Public Const mtWMF_EQUATION As Long = 2
' Macintosh PICT equation graphic (not OLE object) on clipboard
Public Const mtMAC_PICT_EQUATION As Long = 4
' equation OLE 2.0 object on clipboard
Public Const mtWORD_EQUATION As Long = 5
Public Const mtTEXT_EQUATION As Long = 6
Public Const mtMACRO_EQUATION As Long = 7
Public Const mtOLE2_EQUATION As Long = 8
Public Const mtOMML_EQUATION As Long = 9

' error return codes

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
'  insufficient memory
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

' options values for MTInitAPI
Public Const mtinitLAUNCH_AS_NEEDED As Integer = 0
Public Const mtinitLAUNCH_NOW As Integer = 1

' options values for MTGetLastDimension
Public Const mtdimFIRST              As Integer = 1
Public Const mtdimWIDTH              As Integer = 1
Public Const mtdimHEIGHT             As Integer = 2
Public Const mtdimBASELINE           As Integer = 3
Public Const mtdimHORIZ_POS_TYPE     As Integer = 4
Public Const mtdimHORIZ_POS          As Integer = 5
Public Const mtdimLAST               As Integer = 5

' options values for MTGetTranslatorsInfo
Public Const mttrnCOUNT As Integer = 1
Public Const mttrnMAX_NAME As Integer = 2
Public Const mttrnMAX_DESC As Integer = 3
Public Const mttrnMAX_FILE As Integer = 4
Public Const mttrnOPTIONS As Integer = 5

' options values for MTXFormSetPrefs
Public Const mtxfmPREF_EXISTING As Integer = 1
Public Const mtxfmPREF_MTDEFAULT As Integer = 2
Public Const mtxfmPREF_USER As Integer = 3
Public Const mtxfmPREF_LAST As Integer = 3

' options values for MTXFormSetTranslator
Public Const mtxfmTRANSL_INC_NONE As Integer = 0
Public Const mtxfmTRANSL_INC_NAME As Integer = 1
Public Const mtxfmTRANSL_INC_DATA As Integer = 2
Public Const mtxfmTRANSL_INC_MTDEFAULT As Integer = 4
Public Const mtxfmTRANSL_INC_CLIPBOARD_EXTRA As Integer = 8

' return values from MTXFormGetStatus
Public Const mtxfmSTAT_PREF As Long = -3
Public Const mtxfmSTAT_TRANSL As Long = -2
Public Const mtxfmSTAT_ACTUAL_LEN As Long = -1

' data sources/destinations for MTXFormEqn
Public Const mtxfmPREVIOUS As Integer = -1
Public Const mtxfmCLIPBOARD As Integer = -2
Public Const mtxfmLOCAL As Integer = -3
Public Const mtxfmFILE As Integer = -4

' data formats for MTXFormEqn
Public Const mtxfmMTEF As Integer = 4
Public Const mtxfmHMTEF As Integer = 5
Public Const mtxfmPICT As Integer = 6
Public Const mtxfmTEXT As Integer = 7
Public Const mtxfmHTEXT As Integer = 8
Public Const mtxfmGIF As Integer = 9
Public Const mtxfmEPS_NONE As Integer = 10
Public Const mtxfmEPS_WMF As Integer = 11
Public Const mtxfmEPS_TIFF As Integer = 12
Public Const mtxfmRTF As Integer = 13
Public Const mtxfmPDF As Integer = 14

' option values for MTSetMTPrefs
Public Const mtprfMODE_NEXT_EQN  As Integer = 1
Public Const mtprfMODE_MTDEFAULT As Integer = 2
Public Const mtprfMODE_INLINE    As Integer = 4

' option values for MTXFormAddVarSub
Public Const mtxfmSUBST_ALL As Integer = 0
Public Const mtxfmSUBST_ONE As Integer = 1

' find/replace types for MTXFormAddVarSub substitutions
Public Const mtxfmVAR_SUB_BAD         As Integer = -1
Public Const mtxfmVAR_SUB_PLAIN_TEXT  As Integer = 0
Public Const mtxfmVAR_SUB_MTEF_TEXT   As Integer = 1
Public Const mtxfmVAR_SUB_MTEF_BINARY As Integer = 2
Public Const mtxfmVAR_SUB_DELETE      As Integer = 3
Public Const mtxfmVAR_SUB_MAX         As Integer = 4

' replace styles for MTXFormAddVarSub substitutions where type = mtxfmVAR_SUB_PLAIN_TEXT
Public Const mtxfmSTYLE_FIRST    As Integer = 1
Public Const mtxfmSTYLE_TEXT     As Integer = 1
Public Const mtxfmSTYLE_FUNCTION As Integer = 2
Public Const mtxfmSTYLE_VARIABLE As Integer = 3
Public Const mtxfmSTYLE_LCGREEK  As Integer = 4
Public Const mtxfmSTYLE_UCGREEK  As Integer = 5
Public Const mtxfmSTYLE_SYMBOL   As Integer = 6
Public Const mtxfmSTYLE_VECTOR   As Integer = 7
Public Const mtxfmSTYLE_NUMBER   As Integer = 8
Public Const mtxfmSTYLE_LAST     As Integer = 8

' tokens for MathType translator text equations
Public Const mttexteqn_START As String = "% MathType!"
Public Const mttexteqn_END As String = "% MathType!End!"

' Property names
Public Const mtprop_USE_MATHTYPE_PREFS  As String = "MTUseMTPrefs"          'True to use MathType's prefs for new equations
Public Const mtprop_PREFERENCES         As String = "MTPreferences"         'doc's settings for new equations
Public Const mtprop_PREFERENCES_FILE    As String = "MTPreferenceSource"    'pref filename
Public Const mtprop_MTW4_NUMBER_PREFS   As String = "MTEquationNumber"      'MTW4 equation number format
Public Const mtprop_NUMBER_PREFS        As String = "MTEquationNumber"      'MT eqn number format
Public Const mtprop_NUMBER_PREFS_VER    As Long = 2                         'MTW5 & newer property version number
Public Const mtprop_CUSTOM_EQNNUM_PREFS As String = "MTCustomEquationNumber" 'True if eqn number format is custom
Public Const mtprop_DEFER_FIELD_UPDATE  As String = "MTDeferFieldUpdate"    'Controls auto field updating
Public Const mtprop_EQUATION_SECTION_CHECKED As String = "MTEquationSection" 'True if 'eqn section number = 0' check has been made
Public Const mtprop_EQNREFPANE          As String = "MTEqnRefPane"          'Pane # containing insertion point where ref. is to be placed

'old Autotext entry for MTW3's equation# format
Public Const mtautotext_MT3_EQN_NUMBER_FORMAT As String = "ZMTEqnNumFormatPrefs"

' Version-independent MathType OLE ID
Public Const mtole_PROGID As String = "DSEquations"

' Styles
Public Const mtstyle_EQUATION_SECTION As String = "MTEquationSection"
Public Const mtstyle_DISPLAY_EQUATION As String = "MTDisplayEquation"
Public Const mtstyle_CONVERTED_EQUATION As String = "MTConvertedEquation"
' Table Styles
Public Const mttstyle_EB_NUMBERED_EQUATION As String = "MTEBNumberedEquation"

' Macrobuttons
Public Const mtmacro_EDIT_EQUATION_SECTION As String = "MTEditEquationSection"
Public Const mtmacro_EDIT_EQUATION_SECTION_VER As String = "2"  'MTW5 & newer property version number

