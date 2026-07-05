Attribute VB_Name = "MTLib"

'MTLib: 5.0
'=====================================================================
' (c) Copyright 1992-2015 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTLib.bas 286   12/02/15 4:43p Johns $
'=====================================================================

Option Explicit

' Our own constant for Word 2003 wdReadingView
Public Const MTReadingView As Long = 7

' Our own constant for Word 2002 wdPaneRevisions
Public Const MTPaneRevisions As Long = 18

'PromptUser() return codes
Private Const PU_UPDATE As Long = 1
Private Const PU_SKIP As Long = 2
Private Const PU_CANCEL As Long = 3

'ShowTransformError() return codes
Private Const STE_CONTINUE As Long = 1
Private Const STE_CANCEL As Long = 2

'Updating equations conversion kind
Public Const kUE_GRAPHIC As Long = 0
Public Const kUE_TEXT As Long = 1
Public Const kUE_FILE As Long = 2

'public variables for MTMsgBox options
Public gMBStyle As Long
Public gMBCaption As String
Public gMBMessage As String
Public gMBResult As Long
Public gMBQuiet As Boolean  ' programmatically set this to not display MTMsgBox

'public variable for MTPastePrefDlg
' All the EditPaste/PastePrefs code should move to separate module, but it is
' to late to do that in MT6.6 (MT-2153)
Public gPastePrefDlgCanceled As Boolean

Dim gLangResHandle As Long  'Word strings DLL handle

'struct used by routines that format/convert/export eqns
Type UpdateInfo
    prompt As Boolean           'True to prompt user
    title As String             'Title for any dialogs
    update As Boolean           'true if we're updating/converting equations
    kind As Long                'type of update, kUE_FILE, kUE_TEXT, kUE_GRAPHIC
    id As Long                  'number of equations updated
    fileName As String          'filename pattern (incl. path) to use when exporting to files
    fileType As Integer         'filetype when exporting to files
    filePattern As String       'pattern to use for filename/id substitution
End Type

'public variables used for updating equations
Public OLECount As Long, OMMLEqnCount As Long
Public PicCount As Long, MacButtonCount As Long
Public WordEqnCount As Long, TextEqnCount As Long

'data for progress dialg
Type ProgressInfo
    title As String
    numStrings As Long
    default(6) As String
    procID As Long
    activeString As Long
    form As MSForms.UserForm
End Type
Public gProgressInfo As ProgressInfo

Public Const kPROGRESS_MATHPAGEPROC As Long = 1
Private ConvertBookmarkBegin As String
Private ConvertBookmarkEnd  As String
Private equationMarker As String

Enum PasteAsType
    kMTEqn
    kOMML
End Enum

Enum ClipboardFormat
    kMathML
    kText
End Enum

Private ConvertEquationsDelay As Integer

'Activates item in Progress dlg
Sub SetActiveProgressString(id As Long)
    If gProgressInfo.activeString <> 0 Then
        With gProgressInfo.form.Controls("lbl" & gProgressInfo.activeString)
            .font.Bold = False
            .caption = gProgressInfo.default(gProgressInfo.activeString)
        End With
    End If
    gProgressInfo.activeString = id
    If id <> 0 Then
        With gProgressInfo.form.Controls("lbl" & id)
            .font.Bold = True
        End With
    End If
    DoEvents
End Sub

'Updates active string
Sub ShowProgressString(progress As String)
    With gProgressInfo.form.Controls("lbl" & gProgressInfo.activeString)
        .caption = progress
    End With
    DoEvents
End Sub

'Saves Word's current state for features we may want preserved
' These are:
'  TrackRevisions
'  SmartCutPaste
'  PasteSmartCutPaste (Word 2002 and newer only)
'  TypingReplacesSelection
Public Function SaveWordState() As Long
    Dim state As Long

    state = 0
    If ActiveDocument.TrackRevisions Then
        state = state Or mt_SWS_TRACKCHANGES
    End If

    If options.SmartCutPaste Then
        state = state Or mt_SWS_SMART_CUTPASTE
    End If

    'for Word 2002 & newer, handle PasteSmartCutPaste property too
    If Val(Application.version) >= kWordX Then
        state = SaveWordStateX(state)
    End If

    If options.ReplaceSelection Then
        state = state Or mt_SWS_TYPING_REPLACE_SELECTION
    End If

    SaveWordState = state
End Function

'WordX & newer additional version
Private Function SaveWordStateX(ByVal state As Long) As Long
    Dim opt As Object
    Set opt = options
    If opt.PasteSmartCutPaste Then
        state = state Or mt_SWS_PASTE_SMART_CUTPASTE
    End If
    SaveWordStateX = state
End Function

'Restores Word's current state based on contents of state parameter
Public Function RestoreWordState(state As Long) As Long
    Dim restoreValue As Boolean

    RestoreWordState = 0
    'only set the values if we have to
    restoreValue = (state And mt_SWS_TRACKCHANGES)
    If restoreValue <> ActiveDocument.TrackRevisions Then
        ActiveDocument.TrackRevisions = restoreValue
    End If

    restoreValue = (state And mt_SWS_SMART_CUTPASTE)
    If restoreValue <> options.SmartCutPaste Then
        options.SmartCutPaste = restoreValue
    End If

    'for Word 2002 & newer, handle PasteSmartCutPaste property too
    If Val(Application.version) >= kWordX Then
        RestoreWordStateX state
    End If

    restoreValue = (state And mt_SWS_TYPING_REPLACE_SELECTION)
    If restoreValue <> options.ReplaceSelection Then
        options.ReplaceSelection = restoreValue
    End If
End Function

'Word X & newer version
Private Function RestoreWordStateX(state As Long)
    Dim restoreValue As Boolean
    restoreValue = (state And mt_SWS_PASTE_SMART_CUTPASTE)
    Dim opt As Object
    Set opt = options
    If restoreValue <> opt.PasteSmartCutPaste Then
        opt.PasteSmartCutPaste = restoreValue
    End If
End Function

'Only call on WordX and newer
Public Function SetPasteSmartCutPaste(value As Boolean) As Boolean
    Dim opt As Object
    Set opt = options
    opt.PasteSmartCutPaste = value
    SetPasteSmartCutPaste = value
End Function

'Returns True if our cursor is in an OK location. If not, displays
'an error message and returns False.
Public Function IsCursorPlacedOK(title As String) As Boolean
    If Val(Application.version) = kWordX Then
        IsCursorPlacedOK = IsCursorPlacedOKXP(title)
    Else
        IsCursorPlacedOK = IsCursorPlacedOK97(title)
    End If
End Function

Public Function IsInEBEquation(title As String) As Boolean
    IsInEBEquation = False
    If Val(Application.version) >= kWord2007 Then
        If SelInEBEquation Then
            MsgBox MTLib.GetUserString2("1653", "3300", "This command does not work inside Equation Builder equations."), vbOKOnly + vbCritical, title
            IsInEBEquation = True
            Exit Function
        End If
    End If
End Function

Public Function IsInEBEquationTable(title As String) As Boolean
    IsInEBEquationTable = False
    If Val(Application.version) >= kWord2007 Then
        If SelInEBNumberedEqnTable() <> 0 Then
            MsgBox MTLib.GetUserString2("1657", "3301", "This operation is not allowed for previously numbered Equation Builder equations."), vbOKOnly + vbCritical, title
            IsInEBEquationTable = True
            Exit Function
        End If
    End If
End Function

Public Function IsInOutlineView(title As String) As Boolean
    IsInOutlineView = False
    If ActiveDocument.ActiveWindow.ActivePane.View.Type = wdOutlineView Then
        MsgBox MTLib.GetUserString2("1654", "3302", "This command does not work in Outline View."), vbOKOnly + vbCritical, title
        IsInOutlineView = True
        Exit Function
    End If
End Function

'Returns True if our cursor is in an OK location. If not, displays
'an error message and returns False.
Public Function IsCursorPlacedOK97(title As String) As Boolean
    If (Selection.Information(wdInFootnoteEndnotePane) Or _
        Selection.Information(wdInCommentPane) Or _
        Selection.Information(wdInHeaderFooter)) Then

        MsgBox MTLib.GetUserString2("1650", "3303", "This command only works in the main text area of your document."), vbOKOnly + vbCritical, title

        IsCursorPlacedOK97 = False
    Else
        IsCursorPlacedOK97 = True
    End If
End Function

'Returns True if our cursor is in an OK location. If not, displays
'an error message and returns False.
Public Function IsCursorPlacedOKXP(title As String) As Boolean
    If (Selection.Information(wdInFootnoteEndnotePane) Or _
        Selection.Information(wdInCommentPane) Or _
        Selection.Information(wdInHeaderFooter) Or _
        (ActiveDocument.ActiveWindow.View.SplitSpecial = MTPaneRevisions And _
        ActiveDocument.ActiveWindow.ActivePane.index > 1)) Then

        MsgBox MTLib.GetUserString2("1650", "3303", "This command only works in the main text area of your document."), vbOKOnly + vbCritical, title

        IsCursorPlacedOKXP = False
    Else
        IsCursorPlacedOKXP = True
    End If
End Function

'Returns True if commands are allowed in the current view.
'If not, displays an error message and returns False.
Public Function IsCurrentViewOK(title As String) As Boolean
    If Val(Application.version) = kWord2003 Then
        IsCurrentViewOK = IsCurrentViewOK2003(title)
    Else
        IsCurrentViewOK = True
    End If
End Function

'Returns True if commands are allowed in the current view.
'If not, displays an error message and returns False.
'In Word 2003, Reading View does not support ToggleShowCodes on a selection,
'and Selection.Fields.Add will fail unless the user has clicked in the document
Public Function IsCurrentViewOK2003(title As String) As Boolean
   If ActiveWindow.View.Type = MTReadingView Then
      MsgBox MTLib.GetUserString2("1652", "3304", "This command is not available in the current view."), vbOKOnly + vbCritical, title
      IsCurrentViewOK2003 = False
   Else
      IsCurrentViewOK2003 = True
   End If
End Function

'Returns True if a doc's open, else displays an error & returns False
Public Function IsDocumentOpen(title As String) As Boolean
    If Documents.count = 0 Then
        MsgBox MTLib.GetUserString2("1651", "3251", "This command cannot be used as there is no active document."), vbOKOnly + vbCritical, title
        IsDocumentOpen = False
    Else
        IsDocumentOpen = True
    End If
End Function

'Takes a localizable string of the form "!nnnnString", where nnnn is a 4-digit string ID.
'If current language is English, or the language DLL can't be found, just strip the prefix
Public Function GetUserString(englishString As String) As String

    Dim buffer(1023) As Byte
    Dim tmpStr As String
    Dim bufLen As Long

    bufLen = 512
    MTGetUserWString englishString, buffer(0), bufLen
    tmpStr = buffer
    GetUserString = Strings.LeftB(tmpStr, bufLen * 2)

End Function

Public Function GetUserString2(winID As String, macID As String, englishString) As String

    Dim nnnnString As String
    Dim strID As String

    #If Win32 Then
        strID = winID
    #Else
        strID = macID
    #End If

    nnnnString = "!" + strID + englishString

    GetUserString2 = GetUserString(nnnnString)

End Function

'check locale DLL used for substituting a few Word strings,
'display error if not found
Public Function CheckLocaleDLL() As Boolean
    CheckLocaleDLL = True
    If MTInitLocaleStringDLL(Val(Application.version), _
        Application.International(wdProductLanguageID)) <> 0 Then

        ShowMissingLanguageDLLError
        CheckLocaleDLL = False
    End If
End Function

'Gets an individual string by ID from our mswXXX.DLL
'Word97 strings use the ID (range 100-200)
'Word2000 and newer (which may be all-English) add 100 to the ID
'Returns "" with error message if DLL not found
'Aborts with error message if DLL found but string is missing
Public Function GetLocaleStr(origString As String) As String
    Dim stat As Long
    Dim buffer As String
    Dim bufLen As Long
    Dim stringID As Long

    GetLocaleStr = ""

    'make sure DLL is available
    stat = MTInitLocaleStringDLL( _
        Val(Application.version), _
        Application.International(wdProductLanguageID) _
    )

    If stat = mpOK Then
        bufLen = 128
        buffer = Strings.Space(bufLen)
        stat = MTGetLocaleString(origString, buffer, bufLen)
        If stat = mpOK Then
            GetLocaleStr = Strings.left$(buffer, bufLen)
        Else
            'there was an error getting the string out of the file.
            'warn user and then end the macro, this 'should' never happen
            stringID = Strings.Mid$(origString, 2, 4)
            MsgBox MTLib.GetUserString2("1602", "3202", "There is a problem with the MathType language DLL for Microsoft Word. Please contact Design Science. ID ") & stringID, _
                vbOKOnly + vbCritical, MTLib.GetUserString2("1609", "3209", "MathType Commands for Microsoft Word Error")
        End If
    Else
        ShowMissingLanguageDLLError
    End If
End Function

'Displays error telling user about missing language DLL
'Checks registry location before showing error to avoid every time
'If user doesn't want to see it always, writes the reg. value
Private Sub ShowMissingLanguageDLLError()
    Dim langName$, englishLangName$
    Dim buffer$, fileName1$, msg$
    Dim curLang As Long, stat As Long

    Dim BUFSIZE As Long
    BUFSIZE = 256

    ' The DONTSHOW option is not exposed in the UI anymore
    ' but this check remains for backward compatibility
    If GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_DONTSHOW_LANGDLLERROR) = "1" Then
        Exit Sub
    End If

    'figure out the language he should have been using
    curLang = Application.International(wdProductLanguageID)

    #If Win32 Then

    'get the localized name of the language
    buffer$ = Strings.Space(BUFSIZE)
    stat = GetLocaleInfo(curLang, &H2, buffer$, BUFSIZE)
    If stat > 0 Then
        langName$ = Strings.left$(buffer$, stat - 1)
    End If

    'the English name for the language
    buffer$ = Strings.Space(BUFSIZE)
    stat = GetLocaleInfo(curLang, &H1001, buffer$, BUFSIZE)
    If stat > 0 Then
        englishLangName$ = Strings.left$(buffer$, stat - 1)
    End If

    If Len(englishLangName$) > 0 Then
        langName$ = langName$ + " (" + englishLangName$ + ")"
    End If

    #Else

    langName = ""
    englishLangName = ""

    #End If

    'get the name(s) of the DLL(s)
    buffer$ = Strings.Space(BUFSIZE)
    stat = MTGetLocaleDLL(buffer$, BUFSIZE)
    fileName1$ = Strings.left$(buffer$, BUFSIZE)
    msg$ = _
        GetUserString2("1603", "3203", "The MathType language file ") & fileName1$ & _
        GetUserString2("1604", "3204", " for ") & langName$ & _
        GetUserString2("1605", "3205", " is missing. See the Office Support Read Me file in the Office Support folder inside your MathType folder for more information.")

    Beep
    stat = MsgBox(msg$, vbCritical + vbOKOnly, GetUserString2("1609", "3209", "MathType Commands for Microsoft Word Error"))
End Sub

'Returns True if explicit number is valid, returns value in parameter.
'If not, displays error message and selects the field.
Public Function IsValidExplicitNumber(edit As MSForms.TextBox, ByRef result As String, _
    title As String) As Boolean
    result = MTLib.ConvertEntryToNumStr$(edit.value)
    If result = "-" Then
        MsgBox MTLib.GetUserString("!2102You entered an invalid number, please try again"), _
            vbOKOnly + vbCritical, title
        edit.SetFocus
        edit.selStart = 0
        edit.SelLength = Len(edit.Text)
        IsValidExplicitNumber = False
    Else
        IsValidExplicitNumber = True
    End If
End Function

'Enables an edit control in a form
Public Sub EnableEditBox(edit As MSForms.TextBox)
    edit.enabled = True
    #If Win32 Then
    edit.BackStyle = fmBackStyleOpaque
    #End If
    edit.SetFocus
    edit.selStart = 0
    edit.SelLength = Len(edit.Text)
End Sub

'Enables an edit control in a form
Public Sub DisableEditBox(edit As MSForms.TextBox)
    edit.enabled = False
    #If Win32 Then
    edit.BackStyle = fmBackStyleTransparent
    #End If
End Sub

'Enables/disables screen updating, allows nesting, True to enable.
Public Sub SetScreenUpdate(update As Boolean)
    Static UpdateLevel As Long

    If update Then
        UpdateLevel = UpdateLevel - 1
        If UpdateLevel <= 0 Then
            Application.ScreenUpdating = True
            UpdateLevel = 0
        End If
    Else
        If UpdateLevel = 0 Then
            Application.ScreenUpdating = False
        End If
        UpdateLevel = UpdateLevel + 1
    End If
End Sub

'Returns True if style exists in the document.
Public Function styleExists(Doc As Document, style As String) As Boolean
    Dim aStyle As style
    styleExists = False
    On Error GoTo err
    Set aStyle = Doc.Styles(style)
    'if we get here the style exists
    styleExists = True
err:
End Function

'Checks for the MTEquationSection style in the document, adds it if not found.
'Returns True if successful.
Public Function ValidateSectionStyle(Doc As Document) As Boolean

    Dim curStyle As style

    On Error Resume Next
    If Not styleExists(Doc, mtstyle_EQUATION_SECTION) Then
        Set curStyle = Doc.Styles.Add(mtstyle_EQUATION_SECTION, wdStyleTypeCharacter)
        If curStyle Is Nothing Then
            Set curStyle = Doc.Styles.Add(mtstyle_EQUATION_SECTION) ' MT-3103
        End If
        curStyle.BaseStyle = wdStyleDefaultParagraphFont
        curStyle.font.ColorIndex = wdRed
        'hide if ShowAll is off
        curStyle.font.Hidden = Not Doc.ActiveWindow.ActivePane.View.ShowAll
    End If
    ValidateSectionStyle = True
End Function

'Makes the MTEquationSection style not hidden, adding it if necessary.
'Returns True if successful.
Public Function ShowSectionStyle(Doc As Document) As Boolean
    Dim noErr As Boolean
    noErr = ValidateSectionStyle(Doc)
    If noErr = True Then
        Doc.Styles(mtstyle_EQUATION_SECTION).font.Hidden = False
    End If
    ShowSectionStyle = noErr
End Function

'Makes the MTEquationSection style  hidden, adding it if necessary.
'Returns True if successful.
Public Function HideSectionStyle(Doc As Document) As Boolean
    Dim noErr As Boolean
    noErr = ValidateSectionStyle(Doc)
    If noErr = True Then
        Doc.Styles(mtstyle_EQUATION_SECTION).font.Hidden = True
    End If
    HideSectionStyle = noErr
End Function

'Returns True if the MTEquationSection style is hidden, else False
Public Function IsSectionStyleHidden(Doc As Document) As Boolean
    Dim noErr As Boolean
    noErr = ValidateSectionStyle(Doc)
    If noErr = True Then
        IsSectionStyleHidden = Doc.Styles(mtstyle_EQUATION_SECTION).font.Hidden
    Else
        IsSectionStyleHidden = False
    End If
End Function

'Checks for the MTConvertedEquation style in the document, adds it if not found.
'Returns True if successful.
Public Function ValidateConvertedStyle(Doc As Document) As Boolean
    Dim curStyle As style

    If Not styleExists(Doc, mtstyle_CONVERTED_EQUATION) Then
        Set curStyle = Doc.Styles.Add(mtstyle_CONVERTED_EQUATION, wdStyleTypeCharacter)
        curStyle.BaseStyle = wdStyleDefaultParagraphFont
    End If
    ValidateConvertedStyle = True
End Function

'If directory doesn't exist, ask user if they want to create it.
'Returns False if "no", or create fails
Public Function CreateFolder(folder As String, caption As String) As Boolean
    Dim msg As String
    Dim stat As Long

    CreateFolder = False
    On Error GoTo createErr
    If dir$(folder, vbDirectory) = "" Then
        msg = MTLib.GetUserString("!0641This folder does not exist. Would you like to create it?")
        stat = MsgBox(folder & vbCrLf & msg, vbQuestion + vbYesNo, caption)
        If stat = vbYes Then
            If MTCreateDirectory(folder) <> mtOK Then
createErr:
                msg = MTLib.GetUserString("!0645This folder cannot be created or accessed. Please select another folder and try again.")
                MsgBox folder & vbCrLf & msg, vbCritical, caption
                Exit Function
            End If
        Else
            Exit Function
        End If
    End If
    CreateFolder = True
End Function

' Returns True if the path argument contains a volume and an absolute path
Public Function IsVolumeAndAbsolutePath(path As String) As Boolean
#If Win32 Then
    ' check for "c:\xxx" (Windows specific) or "\\xxx"
    If (Strings.Mid$(path, 2, 2) = ":" & Application.PathSeparator) Or _
        (Strings.left$(path, 2) = Application.PathSeparator & Application.PathSeparator) Then
        IsVolumeAndAbsolutePath = True
    Else
        IsVolumeAndAbsolutePath = False
    End If
#Else
    'check for ":", means rel. path
    IsVolumeAndAbsolutePath = Not (Strings.left$(path, 1) = Application.PathSeparator)
#End If
End Function

'Returns filename portion of possible full path
'If simple filename supplied, just return it
Public Function GetFileNameFromPath(path As String) As String
    Dim start As Long, nameLen As Long

    start = InStrR(path, Application.PathSeparator)
    If start = 0 Then
        GetFileNameFromPath = path
    Else
        nameLen = Len(path) - start
        GetFileNameFromPath = Strings.right$(path, nameLen)
    End If
End Function

'Returns parent directory portion of full path
'If simple filename supplied, returns null string ""
Public Function GetParentDirFromPath(path As String) As String
    Dim start As Long

    start = InStrR(path, Application.PathSeparator)
    If start = 0 Then
        GetParentDirFromPath = ""
    Else
        GetParentDirFromPath = Strings.left$(path, start - 1)
    End If
End Function

'Finds position of last occurrence of target string in search string.
'Returns 1-based position, or 0 if not found
'Performs case-insensitive search
'Word2000 has InStrRev, but Word97 doesn't, hence this function
Function InStrR(search As String, target As String) As Long
    Dim lastPos As Long
    Dim newPos As Long

    InStrR = 0
    'check for valid search strings
    If (Len(search$) = 0 Or Len(target$) = 0) Then
        Exit Function
    End If

    lastPos = 0
    Do
        newPos = InStr(lastPos + 1, search$, target$, vbBinaryCompare)
        If newPos = 0 Then
            Exit Do
        Else
            lastPos = newPos
        End If
    Loop

    InStrR = lastPos
End Function

'Gets the location of MathType from the registry
Public Function GetMathTypeDir() As String

#If Win32 Then
    GetMathTypeDir = GetPreference(HKEY_LOCAL_MACHINE, mtreg_MT_HKLM_DIRECTORIES, mtreg_MT_PROGDIR_KEY)
    ' if we couldn't get the path from registry then give it the default
    If GetMathTypeDir = "" Then
        GetMathTypeDir = "C:\Program Files (x86)\MathType"
    End If
#Else
    Dim path(1023) As Byte
    Dim pathLen As Long
    Dim stat As Long
    Dim tmpStr As String

    pathLen = 512

    stat = MTGetWPathToMathType(path(0), pathLen)
    If stat = mtOK Then
        tmpStr = path
        GetMathTypeDir = Strings.LeftB(tmpStr, pathLen * 2)
    Else
        GetMathTypeDir = ""
    End If
#End If

End Function

'Gets the location of MathType's Language directory from the registry
Public Function GetMTLanguageDir() As String
    GetMTLanguageDir = GetPreference(HKEY_LOCAL_MACHINE, mtreg_MT_HKLM_DIRECTORIES, mtreg_MT_LANGUAGEDIR_KEY)
End Function

'Gets the location of MathType's Preferences directory from the registry
'If registry value missing, returns MathType directory.
Public Function GetMTPrefDir() As String
    Dim path As String

    'get the location of Mathtype from the registry
    path = GetPreference(HKEY_LOCAL_MACHINE, mtreg_MT_HKLM_DIRECTORIES, mtreg_MT_PREFDIR_KEY)

    If path = "" Then
        path = GetMathTypeDir
    End If

    'return the results
    GetMTPrefDir = path
End Function

'Brings up a dialog to choose a MathType preference file.
'Starts in MathType's Preferences dialog.
'Returns True if a selection was made, in this case
'the file name parameter contains the selected filename.
Public Function ChoosePrefFile(fileName As String) As Boolean
    Dim result As Long, nullStart As Long

    fileName = Strings.String(MTAPI_MAX_PATH, 0) 'init string for file path and name
    result = MTOpenFileDialog(1, MTLib.GetUserString2("1612", "3212", "Choose MathType Preference File"), GetMTPrefDir, fileName, MTAPI_MAX_PATH)

    'strip trailing NULL characters
    If result = 0 Then
        ChoosePrefFile = False
    Else
        nullStart = InStr(1, fileName, Strings.Chr(0), vbBinaryCompare)
        If nullStart > 0 Then
            fileName = Strings.left$(fileName, nullStart - 1)
        End If
        ChoosePrefFile = True
    End If
End Function

'returns as a string the preferences stored in the current Word document by
'the Equation Preferences menu item.
Public Function GetPrefsFromDoc$()
   GetPrefsFromDoc$ = ReadDocPropString$(ActiveDocument, mtprop_PREFERENCES)
End Function

'Returns MathType's "defaults for new equations" settings as a string.
'If an error occurs, returns an empty string ("")
Public Function GetPrefsFromMType$()
    Dim strSize As Integer
    Dim returnStr$
    Dim result

    'get the size of the string
    strSize = MTGetPrefsMTDefault("", 0)
    If strSize > 0 Then
        'initialize the string variable to that size
        returnStr$ = Strings.Space(strSize)
        'get the string
        result = MTGetPrefsMTDefault(returnStr$, strSize)
        'check for errors
        If result = mtOK Then
            GetPrefsFromMType$ = returnStr$
        Else
            MsgBox MTLib.GetUserString2("1613", "3213", "Error getting preferences from MathType"), vbExclamation, MTLib.GetUserString2("1614", "3214", "MathType Macro Error")
            GetPrefsFromMType$ = ""
        End If
    Else
        MsgBox MTLib.GetUserString2("1613", "3213", "Error getting preferences from MathType"), vbExclamation, MTLib.GetUserString2("1614", "3214", "MathType Macro Error")
        GetPrefsFromMType$ = ""
    End If
End Function

'If there is an equation on the clipboard, returns the equation preferences
'in that equation as a string.
'If an error occurs, returns an empty string ("")
Public Function GetPrefsFromClipboard$()
    Dim strSize As Integer
    Dim returnStr$
    Dim result

    'get the size of the string
    strSize = MTGetPrefsFromClipboard("", 0)
    If strSize > 0 Then
        'initialize the string variable to that size
        returnStr$ = Strings.Space(strSize)
        'get the string
        result = MTGetPrefsFromClipboard(returnStr$, strSize)
        'check for errors
        If result = mtOK Then
            GetPrefsFromClipboard$ = returnStr$
        ElseIf result = mtNOT_EQUATION Then
            MsgBox MTLib.GetUserString2("1615", "3215", "There was not an equation on the clipboard"), vbExclamation, MTLib.GetUserString2("1616", "3216", "MathType Macro Message")
            GetPrefsFromClipboard$ = ""
        ElseIf result = mtDATA_NOT_FOUND Then
            MsgBox MTLib.GetUserString2("1617", "3217", "The equation on the clipboard did not have any preferences attached to it"), vbExclamation, MTLib.GetUserString2("1616", "3216", "MathType Macro Message")
            GetPrefsFromClipboard$ = ""
        Else
            MsgBox MTLib.GetUserString2("1618", "3218", "Error getting preferences from the clipboard"), vbExclamation, MTLib.GetUserString2("1616", "3216", "MathType Macro Message")
            GetPrefsFromClipboard$ = ""
        End If
    Else
        MsgBox MTLib.GetUserString2("1618", "3218", "Error getting preferences from the clipboard"), vbExclamation, MTLib.GetUserString2("1616", "3216", "MathType Macro Message")
        GetPrefsFromClipboard$ = ""
    End If
End Function

'given a file name, this will return the MathType preferences in that file
'as a string. The file must be a MathType Preference file.
'If an error occurs, returns an empty string ("")
Public Function GetPrefsFromFile$(locFileName$)
    Dim strSize As Integer
    Dim returnStr$
    Dim result

    'get the size of the string
    strSize = MTGetPrefsFromFile(locFileName$, "", 0)
    'initialize the string variable to that size
    If strSize > 0 Then
        returnStr$ = Strings.Space(strSize)
        'get the string
        result = MTGetPrefsFromFile(locFileName$, returnStr$, strSize)
        'check for errors
        If result = mtOK Then          'good data - return the string
            GetPrefsFromFile$ = returnStr$
        ElseIf result = mtMEMORY Then     'out of memory. probably ReturnStr$ was not large enough
            MsgBox MTLib.GetUserString2("1619", "3219", "Error getting preferences from the file"), vbExclamation, MTLib.GetUserString2("1620", "3220", "MathType Macro Message")
            GetPrefsFromFile$ = ""
        ElseIf result = mtBAD_FILE Then    'Bad file - it didn't have the right data in it
            MsgBox MTLib.GetUserString2("1621", "3221", "The preference file contained bad data or did not contain preferences at all."), vbExclamation, MTLib.GetUserString2("1620", "3220", "MathType Macro Message")
            GetPrefsFromFile$ = ""
        Else                        'generic bad result
            MsgBox MTLib.GetUserString2("1622", "3222", "Error getting preferences from the file"), vbExclamation, MTLib.GetUserString2("1620", "3220", "MathType Macro Message")
            GetPrefsFromFile$ = ""
        End If
    ElseIf strSize = mtFILE_NOT_FOUND Then        'file not found
        MsgBox MTLib.GetUserString2("1623", "3223", "File not found. Either the location or the file name was incorrect."), vbExclamation, MTLib.GetUserString2("1620", "3220", "MathType Macro Message")
        GetPrefsFromFile$ = ""
    Else                            'generic bad result
        MsgBox MTLib.GetUserString2("1622", "3222", "Error getting preferences from the file"), vbExclamation, MTLib.GetUserString2("1620", "3220", "MathType Macro Message")
        GetPrefsFromFile$ = ""
    End If
End Function

'Sets prefs that MathType will use for the next new equation.
'Returns MTSetMTPrefs result code.
Public Function SetPrefsForNextEqn(prefStr As String, inline As Boolean) As Long
    Dim stat As Long
    Dim options As Integer

    options = mtprfMODE_NEXT_EQN
    If inline Then options = options + mtprfMODE_INLINE

    'set preferences for next transformed equation
    stat = MTSetMTPrefs(options, prefStr, -1)
    If stat <> mtOK Then
        MsgBox MTLib.GetUserString("!1100There was a problem sending your equation preferences for " _
            + "this document to MathType. This equation will use MathType's " _
            + "'New Equation' preferences."), vbExclamation, _
            MTLib.GetUserString("!1101MathType Preferences Problem")
    End If
    SetPrefsForNextEqn = stat
End Function

'Returns id of equation, 1 = OLE1, 2 = OLE2, 0 = unknown
Public Function IsEquationProgID(progID As String) As Long
    Dim uProgID As String
    uProgID = Strings.UCase(progID)

    If uProgID = "EQUATION" Then
        IsEquationProgID = 1
    ElseIf InStr(1, uProgID, "EQUATION.", vbBinaryCompare) = 1 Then
        IsEquationProgID = 2
    Else
        IsEquationProgID = 0
    End If
End Function

'Returns id of equation, 1 = OLE1, 2 = OLE2, 0 = unknown
Public Function IsShapeEquation(oFormat As OLEFormat) As Long

    'accessing ProgID may cause an error 5825 on corrupt objects, so check ClassType first

    Dim strToCheck As String
    IsShapeEquation = 0
    On Error Resume Next

    ' check ClassType
    strToCheck = Strings.UCase(oFormat.ClassType)
    If strToCheck = "EQUATION" Then
        IsShapeEquation = 1
    ElseIf InStr(1, strToCheck, "EQUATION.", vbBinaryCompare) = 1 Then
        IsShapeEquation = 2
    ElseIf (GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_NO_CHECK_PROG_ID) <> 1) Then
        ' check progID
        strToCheck = Strings.UCase(oFormat.progID)
        If strToCheck = "EQUATION" Then
            IsShapeEquation = 1
        ElseIf InStr(1, strToCheck, "EQUATION.", vbBinaryCompare) = 1 Then
            IsShapeEquation = 2
        End If
    End If

End Function

Public Sub MTEditEquationOpen()
    MTEditEquation False
End Sub

Public Sub MTEditEquationInPlace()
    MTEditEquation True
End Sub

Public Sub MTEditEquation(inPlace As Boolean)

    On Error GoTo bye
    'if it's a floating object, keep it as such
    If Selection.InlineShapes.count <= 0 Then
        Dim myID As Long
        If inPlace Then
            myID = 30003 'id for the Edit menu
        Else
            myID = 30006 'id for the Format menu
        End If
        Dim editMenu As CommandBarControl
        Set editMenu = CommandBars.ActiveMenuBar.FindControl(Type:=msoControlPopup, id:=myID)
        If Not editMenu Is Nothing Then editMenu.Execute
        Exit Sub
    End If

    Dim curInlineShape As InlineShape
    Set curInlineShape = Selection.InlineShapes(1)

    If (curInlineShape.OLEFormat Is Nothing) Then
        Exit Sub
    End If

    Dim oleObj As Object
    Set oleObj = curInlineShape.OLEFormat

    If (inPlace) Then
        oleObj.DoVerb (1)
    Else
        oleObj.DoVerb (0)
    End If

bye:
End Sub

'Called by our EditPicture wrapper
'tests selected shape & converts into a MathType equation if its a MT picture
Public Sub MTEditPicture()
    Dim inline As Boolean
    Dim stat As Long
    Dim curInlineShape As InlineShape
    Dim curShape As Shape
    Dim newShape As Shape
    Dim ueInfo As UpdateInfo

    InitEqnCounts

    On Error GoTo bye

    'if it's a floating object, keep it as such
    If Selection.Type = wdSelectionShape Then
        Set curShape = Selection.ShapeRange(1)
        inline = False
    Else
        Set curInlineShape = Selection.InlineShapes(1)
        inline = True
    End If

    'reset XForm prefs/translators
    stat = MTXFormReset()
    If stat = mtOK Then
        Selection.Copy
        ueInfo.kind = kUE_GRAPHIC   'only need kind set
        stat = MTLib.TransformGraphicEquation(ueInfo)
        If stat = mtOK Then
            If inline Then
                Selection.Range.PasteSpecial placement:=wdInLine
                curInlineShape.delete
                #If Win32 Then
                Selection.InlineShapes(1).Activate
                #End If
            Else
                Selection.Range.PasteSpecial placement:=wdFloatOverText
                'place the new shape where the old one is
                Set newShape = ActiveDocument.Range.ShapeRange(1)
                newShape.left = curShape.left
                newShape.top = curShape.top
                curShape.delete
                #If Win32 Then
                newShape.Activate
                #End If
            End If
        Else
            If inline Then
                #If Win32 Then
                curInlineShape.Activate
                #End If
            Else
                #If Win32 Then
                curShape.Activate
                #End If
            End If
        End If
    End If

bye:
End Sub

'Updates graphic equations (OLE/Picture equations) in doc/selection.
'Handles floating shapes first, incl. text boxes (frames) which can contain inline shapes.
'Returns 0 if OK, -1 to abort update
Public Function UpdateGraphics(docRange As Long, ByRef ueInfo As UpdateInfo) As Long
    Dim isEquation As Long
    Dim curShapeRange As ShapeRange
    Dim curFloatShape As Shape
    Dim selRange As Range
    Dim oldRange As Range
    Dim isNonCompatModeOffice2011 As Boolean
    Dim curGroupShapex As Shape
    Dim curInlineShape As InlineShape

    UpdateGraphics = 0
    isNonCompatModeOffice2011 = False

    'keep the user's selection so we can restore it
    Set oldRange = Selection.Range

    'if "Update Whole Document" was selected, set the range to the whole document
    'otherwise, set the range to the current selection
    If ActiveDocument.shapes.count = 0 Then GoTo doInlines

    On Error Resume Next
    If docRange = mt_RANGE_DOCUMENT Then
        Set curShapeRange = ActiveDocument.Range.ShapeRange
    Else
        Set curShapeRange = Selection.Range.ShapeRange
    End If

    'Mac Word 2011 in non-compatibility mode will not find any
    'equations if there is a text box anywhere in the document
    If curShapeRange Is Nothing Then
        isNonCompatModeOffice2011 = True
        GoTo doInlines
    End If

    'loop through the floating graphics first
    If curShapeRange.count > 0 Then
        For Each curFloatShape In curShapeRange
            'if it is an OLE equation or a picture...
            isEquation = 0
            If curFloatShape.Type = msoEmbeddedOLEObject Then
                isEquation = MTLib.IsShapeEquation(curFloatShape.OLEFormat)
            End If

            If isEquation = 0 Then
                isEquation = (curFloatShape.Type = msoPicture)
            End If

            'see if we have inline eqns in a floating text box!
            If isEquation = 0 Then
                If curFloatShape.Type = msoTextBox Or curFloatShape.Type = msoAutoShape Then
                    If curFloatShape.TextFrame.HasText Then
                        If curFloatShape.TextFrame.TextRange.InlineShapes.count > 0 Then
                            If UpdateInlineShapesInRange(curFloatShape.TextFrame.TextRange, ueInfo) = -1 Then
                                UpdateGraphics = -1
                                Exit Function
                            End If
                        End If
                    End If
                Else
                    For Each curGroupShapex In curFloatShape.GroupItems
                        Dim index As Integer
                        For index = 0 To curGroupShapex.TextFrame.TextRange.InlineShapes.count
                            Set curInlineShape = curGroupShapex.TextFrame.TextRange.InlineShapes(index)
                            Dim eqnType As Long
                            eqnType = 0
                            If curInlineShape.Type = wdInlineShapeEmbeddedOLEObject Then
                                eqnType = MTLib.IsShapeEquation(curInlineShape.OLEFormat)
                            End If
                            If eqnType = 0 And (curInlineShape.Type = wdInlineShapeLinkedPicture Or curInlineShape.Type = wdInlineShapeLinkedPicture) Then
                                eqnType = 2
                            End If
                            UpdateInlineShape curInlineShape, eqnType, ueInfo
                        Next index
                    Next curGroupShapex

                End If
            End If

            If isEquation <> 0 Then
                If UpdateFloatingShape(curFloatShape, isEquation, ueInfo) = -1 Then
                    UpdateGraphics = -1
                    Exit Function
                End If
            End If
        Next curFloatShape
    End If

doInlines:
    'process selection
    If docRange = mt_RANGE_SELECTION Then
        If UpdateInlineShapesInRange(oldRange, ueInfo) = -1 Then
            UpdateGraphics = -1
        End If
        Exit Function
    End If

    'process entire document
    Dim currentRange As Range
    For Each currentRange In ActiveDocument.StoryRanges
        If currentRange.storyType <> wdTextFrameStory Or isNonCompatModeOffice2011 Then 'these are processed above as floating graphics
            If UpdateInlineShapesInRange(currentRange, ueInfo) = -1 Then
                UpdateGraphics = -1
            End If
        End If
    Next currentRange

End Function

'Updates a floating shape, prompts if requested.
'Returns 0 if OK, -1 to abort update, -2 if update failed
Private Function UpdateFloatingShape(curShape As Shape, eqnType As Long, ByRef ueInfo As UpdateInfo) As Long

    Dim doPaste As Long, stat As Long, numShapes As Long
    Dim newFloatShape As Shape

    UpdateFloatingShape = -2

    'select the object & copy to clipboard
    curShape.Select
    On Error GoTo copyerr
    Selection.Copy 'may fail with error 4198
    DelayClipboardCopy
    On Error GoTo 0

    'save count of shapes
    numShapes = ActiveDocument.Range.ShapeRange.count
    'Transform it
    stat = TransformGraphicEquation(ueInfo)
    If stat <> mtNOT_EQUATION Then
        'if it was an equation, ask user if we're prompting
        doPaste = promptUser(ueInfo)

        'if user cancelled, exit loop
        If doPaste = PU_CANCEL Then
            UpdateFloatingShape = -1
            Exit Function
        ElseIf doPaste = PU_UPDATE Then
            'if we're going to paste, check if any other error occurred
            If stat = mtOK Then
                If ueInfo.update Then
                    If ueInfo.kind = kUE_FILE Then
                        Selection.InsertAfter GetExportFilename(ueInfo)
                    Else
                        PasteTranslatedEquation ueInfo, wdFloatOverText
                        'if we pasted a new floating object (shape)...
                        If numShapes < ActiveDocument.Range.ShapeRange.count Then
                            'place the new shape where the old one is
                            Set newFloatShape = ActiveDocument.Range.ShapeRange(1)
                            newFloatShape.left = curShape.left
                            newFloatShape.top = curShape.top
                        End If
                    End If

                    'delete original object
                    curShape.delete
                End If

                If eqnType = 1 Then
                    UpdateCount mtOLE_EQUATION
                ElseIf eqnType = 2 Then
                    UpdateCount mtOLE2_EQUATION
                Else
                    UpdateCount mtWMF_EQUATION
                End If
                ueInfo.id = ueInfo.id + 1
                UpdateFloatingShape = 0

            'show error message, abort if user Cancels
            ElseIf stat < 0 Then
                If ShowTransformError(stat, ueInfo.title) = STE_CANCEL Then
                    UpdateFloatingShape = -1
                    Exit Function
                End If
            End If
        End If
    End If
    Exit Function
copyerr:
    If err.Number = 4198 Then 'ignore error 4198 and proceed
        Exit Function
    End If
    err.Raise err.Number, err.Source, err.Description, err.helpFile, err.HelpContext
End Function

'Updates all inline shapes in the range, prompts if requested.
'Returns 0 if OK, -1 to abort update
Private Function UpdateInlineShapesInRange(selRange As Range, ByRef ueInfo As UpdateInfo) As Long

   Dim eqnType As Long
   Dim myRange As Range
   Dim curShape As InlineShape
   Dim stat As Long
   Dim msg As String
   Dim curRangeEnd As Long

   UpdateInlineShapesInRange = 0
   ClearUndo

   On Error GoTo err

   'Use a For Each inside of a While to get each shape because:
   '1) just a For Each would re-process the "updated" equation as well
   '2) just a While or For/Next and accessing InlineShapes as an array
   '   fails in Office XP due to holes in the array
   Set myRange = selRange
   'loop through all inline graphics in the selected range
   While (myRange.InlineShapes.count > 0)
      VBAEmptyClipboard
      'get the first shape in the current range
      For Each curShape In myRange.InlineShapes
         'if it is an OLE equation or a picture...
         eqnType = 0
         If curShape.Type = wdInlineShapeEmbeddedOLEObject Then
             eqnType = MTLib.IsShapeEquation(curShape.OLEFormat)
         End If
         If eqnType = 0 And (curShape.Type = wdInlineShapePicture Or _
                             curShape.Type = wdInlineShapeLinkedPicture) Then
             eqnType = 2
         End If
         If eqnType <> 0 Then
            stat = UpdateInlineShape(curShape, eqnType, ueInfo)
            If stat = -1 Then
               UpdateInlineShapesInRange = -1
               Exit Function
            End If
            If stat = -2 Then
               curRangeEnd = curShape.Range.end
            Else
               curRangeEnd = Selection.Range.end
            End If
         Else
            curRangeEnd = curShape.Range.end
         End If

         'reset current range & get next 'first' inlineshape
         Set myRange = selRange
         myRange.start = curRangeEnd
         Exit For
      Next
nextShape:
   Wend

   Exit Function

err:
   'handle deleted object error (possibly due to equation that was corrupted but was fixed by Word)
   If err.Number = 5825 Then
      Resume nextShape
   End If

   #If Mac Then
   If err.Number = 5097 Then
        Undo
        msg = MTLib.GetUserString("!3336Error: Please run Convert Equations again, and select 'Switch to Office 2011' when prompted.")
   Else
        msg = MTLib.GetUserString2("1670", "3270", "Error ") & err.Number & MTLib.GetUserString2("1671", "3271", " occurred, ") & err.Description
   End If
   #Else
   msg = MTLib.GetUserString2("1670", "3270", "Error ") & err.Number & MTLib.GetUserString2("1671", "3271", " occurred, ") & err.Description
   #End If

   MsgBox msg, vbExclamation, ueInfo.title
   UpdateInlineShapesInRange = -1
End Function

'place As WdOLEPlacement not supported in W97 - use Variant instead
Private Sub PasteTranslatedEquation(ByRef ueInfo As UpdateInfo, place As Variant)
    'apply style to inserted text
    Dim pasteStart As Long

    pasteStart = Selection.Range.start
    Selection.PasteSpecial placement:=place
    If ueInfo.kind = kUE_TEXT Then
        CleanUpPasteKludge
        Dim aRange As Range
        Set aRange = Selection.Range
        aRange.SetRange start:=pasteStart, end:=Selection.Range.end
        Dim noApply As String
        noApply = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_NOAPPLY_CONVERTED_EQN_STYLE)
        If noApply = "0" Or Len(noApply) = 0 Then
            aRange.style = mtstyle_CONVERTED_EQUATION
        End If
    End If
End Sub

Private Sub PostProcessConvertedEquation(ByRef ueInfo As UpdateInfo, place As Variant)

    'apply style to inserted text
    Dim pasteStart As Long

    pasteStart = Selection.Range.start
    If ueInfo.kind = kUE_TEXT Then
        CleanUpPasteKludge
        Dim aRange As Range
        Set aRange = Selection.Range
        aRange.SetRange start:=pasteStart, end:=Selection.Range.end
        Dim noApply As String
        noApply = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_NOAPPLY_CONVERTED_EQN_STYLE)
        If noApply = "0" Or Len(noApply) = 0 Then
            aRange.style = mtstyle_CONVERTED_EQUATION
        End If
    End If

End Sub

'Updates a single inline shape, prompts if requested.
'Returns 0 if OK, -1 to abort update, -2 if update failed
Private Function UpdateInlineShape(curShape As InlineShape, eqnType As Long, ByRef ueInfo As UpdateInfo) As Long

    Dim doPaste As Long, stat As Long
    Dim shapeInField, retval As Boolean
    Dim MTEF() As Byte
    Dim MTEFLen As Long

    shapeInField = False
    UpdateInlineShape = -2
    'do these tests to avoid 'command failed' error that happens
    'if graphic is inside a MacroButton field & field codes are hidden
    '(happens with older MT 1.1 DDE equations)
    If Not (curShape.Field Is Nothing) Then
        If curShape.Field.Type = wdFieldMacroButton Then
            shapeInField = True
            curShape.Field.ShowCodes = True
        End If
    End If

    curShape.Select
    On Error Resume Next

    retval = GetMTEF(curShape, MTEF, MTEFLen)
    If (retval = True And ueInfo.kind = kUE_FILE) Then
        ConvertAndInsertEquation ueInfo, Selection.Range, MTEF, MTEFLen, vbNullString
    End If
    If retval = True Then
        doPaste = promptUser(ueInfo)
        'if user cancelled, exit loop
        If doPaste = PU_CANCEL Then
            UpdateInlineShape = -1
            Exit Function
        ElseIf doPaste = PU_UPDATE Then
            If ueInfo.update Then
                If shapeInField Then
                    curShape.Field.delete
                Else
                    InsertEquationMarkers (".")
                    curShape.delete
                End If

                If ueInfo.kind = kUE_FILE Then
                    'insert filename
                    Selection.InsertAfter GetExportFilename(ueInfo)
                Else    'graphic or text
                    ConvertAndInsertEquation ueInfo, Selection.Range, MTEF, MTEFLen, vbNullString
                End If
            End If
            If eqnType = 1 Then
                UpdateCount mtOLE_EQUATION
            ElseIf eqnType = 2 Then
                UpdateCount mtOLE2_EQUATION
            Else
                UpdateCount mtWMF_EQUATION
            End If
            ueInfo.id = ueInfo.id + 1
            UpdateInlineShape = 0

            DeleteEquationMarkers
        End If
    End If
    Exit Function
copyerr:
    If err.Number = 4198 Or err.Number = 5690 Then 'equation appears in deleted text; ignore it
        Exit Function
    End If
    err.Raise err.Number, err.Source, err.Description, err.helpFile, err.HelpContext
End Function

Public Function UpdateOMMLEqns(docRange As Long, ByRef ueInfo As UpdateInfo) As Long

    UpdateOMMLEqns = 0
    Dim curShapeRange As ShapeRange
    Dim curFloatShape As Shape
    Dim saveRange As Range

    Set saveRange = Selection.Range

    WriteLog "Entering UpdateOMMLEqns"

    'if "Update Whole Document" was selected, set the range to the whole document
    'otherwise, set the range to the current selection
    If ActiveDocument.shapes.count = 0 Then GoTo doInlines
    If docRange = mt_RANGE_DOCUMENT Then
        Set curShapeRange = ActiveDocument.Range.ShapeRange
    Else
        Set curShapeRange = Selection.Range.ShapeRange
    End If

    'loop through the floating graphics first
    If curShapeRange.count > 0 Then
        For Each curFloatShape In curShapeRange
            'see if we have inline eqns in a floating text box!
            If curFloatShape.Type = msoTextBox Or curFloatShape.Type = msoAutoShape Then
                If curFloatShape.TextFrame.HasText Then
                    If UpdateOMMLByRange(curFloatShape.TextFrame.TextRange, ueInfo) = -1 Then
                        UpdateOMMLEqns = -1
                        Exit Function
                    End If
                End If
            End If
        Next curFloatShape
    End If

doInlines:
    'process selection
    If docRange = mt_RANGE_SELECTION Then
        If UpdateOMMLByRange(saveRange, ueInfo) = -1 Then
            UpdateOMMLEqns = -1
        End If
        Exit Function
    End If

    'process entire document
    Dim currentRange As Range
    For Each currentRange In ActiveDocument.StoryRanges
        If currentRange.storyType <> wdTextFrameStory Then 'these are processed above as floating graphics
            If UpdateOMMLByRange(currentRange, ueInfo) = -1 Then
                UpdateOMMLEqns = -1
            End If
        End If
    Next currentRange

    WriteLog "Exiting UpdateOMMLEqns"
bye:
End Function

Public Function UpdateOMMLByRange(ByRef selRange As Range, ByRef ueInfo As UpdateInfo) As Long

    If Val(Application.version) >= kWord2007 Then
        UpdateOMMLByRange = UpdateOMMLByRange2007(selRange, ueInfo)
    End If
    UpdateOMMLByRange = UpdateOMMLByRange2003(selRange, ueInfo)

End Function

Public Function UpdateOMMLByRange2003(ByRef selRange As Range, ByRef ueInfo As UpdateInfo) As Long

    ' Word 2003 can extract OMML from OMML Images but not OMML Equations
    ' while Word 2007 can't extract OMML from OMML Images, but can work
    ' with OMML equations directly.  No other versions of Word can deal
    ' with either format
    WriteLog "Word pre-2007 Detected, converting OMML Images"
    UpdateOMMLImagesInRange selRange, ueInfo

    UpdateOMMLByRange2003 = 0

End Function

Private Sub ClearUndo()
    ActiveDocument.UndoClear
End Sub

Private Sub Undo()
    Do
    Loop While (ActiveDocument.Undo(1) = True)
End Sub

Public Function UpdateOMMLByRange2007(ByRef selRange As Range, ByRef ueInfo As UpdateInfo) As Long

    Dim doPaste As Long, index As Long
    Dim rng As Range
    Dim ommlStr As String
    Dim ommlEqn As Object
    Dim mtEqn As InlineShape
    Dim xformErr As Boolean, xformResult As Boolean

    UpdateOMMLByRange2007 = 0

    WriteLog "Word 2007 Detected, converting OMML equations"
    For index = selRange.OMaths.count To 1 Step -1

        Set ommlEqn = selRange.OMaths(index)

        ClearUndo
        Set rng = ommlEqn.Range
        rng.Select
        ' get update action
        doPaste = promptUser(ueInfo)
        'exit if cancelled
        If doPaste = PU_CANCEL Then
            UpdateOMMLByRange2007 = -1
            GoTo bye
        ElseIf doPaste = PU_UPDATE Then

            'get OMML string
            #If Win32 Then
            ommlStr = GetOMMLFromOMMLEqn(ommlEqn)
            #Else ' If Mac
            rng.Copy
            DelayClipboardCopy
            Dim bufLen As Long
            Dim stat As Long
            bufLen = 0
            stat = MTGetMathMLFromClipboardText(vbNullString, bufLen)
            If (stat = mtOK And bufLen > 0) Then
                ommlStr = Strings.Space(bufLen)
                stat = MTGetMathMLFromClipboardText(ommlStr, bufLen)
            End If
            #End If

            'convert OMML to MT eqn
            If InsertMTEqnFromOMML(rng, ommlStr) = False Then
                UpdateOMMLByRange2007 = -1
                Undo
                GoTo bye
            End If
            ActiveDocument.Fields.update

            ' Transform the new MT Eqn to Text if necessary
            If ueInfo.kind = kUE_TEXT Then
                Set mtEqn = rng.InlineShapes(1)
                xformErr = False
                xformResult = XformMTEqnToText(mtEqn, ueInfo, xformErr)
                If xformErr Then
                    Undo
                Else
                    UpdateCount mtOMML_EQUATION
                    ueInfo.id = ueInfo.id + 1
                End If
                If Not xformResult Then
                    UpdateOMMLByRange2007 = -1
                    GoTo bye
                End If
            Else
                UpdateCount mtOMML_EQUATION
                ueInfo.id = ueInfo.id + 1
            End If

        End If ' Do Paste
    Next index

bye:

End Function


Public Sub UpdateOMMLImagesInRange(ByRef selRange As Range, ByRef ueInfo As UpdateInfo)

    Dim bufStr As String
    Dim bufLen As Long
    Dim curShape As InlineShape, mtEqn As InlineShape
    Dim ommlStr As String
    Dim rng As Range
    Dim pos As Long
    Dim stat As Long, doPaste As Long, index As Long
    Dim xformErr As Boolean, xformResult As Boolean

    WriteLog "Entering UpdateOMMLImagesInRange"

    For index = selRange.InlineShapes.count To 1 Step -1

       Set curShape = selRange.InlineShapes(index)

       If (curShape.Type = wdInlineShapePicture Or _
            curShape.Type = wdInlineShapeLinkedPicture) Then

            curShape.Range.Copy
            DelayClipboardCopy

            WriteLog "Found InlineShapePicture"

            'get buffer length
            Dim gotOmml As Boolean
            gotOmml = False
            bufLen = 0

            ' Word 2007 always reads from the PNG OA format
            ' Word 2003 always reads from the PNG OA format unless a registry
            ' override exists
            Dim ommlInRTF As Boolean
            ommlInRTF = False
            If (Val(Application.version) <= kWord2003 And _
                GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_2003_OMML_IN_RTF) = "1") Then
                ommlInRTF = True
            End If

            If (ommlInRTF) Then
                stat = MTGetOMMLFromClipboardRTF(vbNullString, bufLen)
                If (stat = mtOK And bufLen > 0) Then
                    bufStr = Strings.Space(bufLen)
                    stat = MTGetOMMLFromClipboardRTF(bufStr, bufLen)
                    If (stat = mtOK) Then
                        gotOmml = True
                    End If
                End If
            Else
                stat = MTGetOMMLFromClipboardPNGOA(vbNullString, bufLen)
                If (stat = mtOK And bufLen > 0) Then
                    bufStr = Strings.Space(bufLen)
                    stat = MTGetOMMLFromClipboardPNGOA(bufStr, bufLen)
                    If (stat = mtOK) Then
                        gotOmml = True
                    End If
                End If
            End If

            If gotOmml Then

                bufStr = Strings.RTrim(bufStr)
                WriteLog "OMML detected in image"
                ClearUndo

                ' select equation for user and get update action
                curShape.Range.Select
                InsertEquationMarkers "."
                doPaste = promptUser(ueInfo)
                'exit if cancelled
                If doPaste = PU_CANCEL Then
                    GoTo bye
                ElseIf doPaste = PU_UPDATE Then
                    pos = InStr(bufStr, "wordDocument>")
                    If pos > 0 Then
                        ommlStr = Strings.left(bufStr, pos + 13)
                    Else
                        ommlStr = Strings.left(bufStr, bufLen)
                    End If

                    Set rng = curShape.Range

                    If (InsertMTEqnFromOMML(rng, ommlStr) = False) Then
                        Undo
                        GoTo bye
                    End If

                    ' Transform the new MT Eqn to Text if necessary
                    If ueInfo.kind = kUE_TEXT Then
                        Set mtEqn = rng.InlineShapes(1)
                        xformErr = False
                        xformResult = XformMTEqnToText(mtEqn, ueInfo, xformErr)
                        If xformErr Then
                            Undo
                        Else
                            UpdateCount mtOMML_EQUATION
                            ueInfo.id = ueInfo.id + 1
                        End If
                        If Not xformResult Then
                            GoTo bye
                        End If
                    Else
                        UpdateCount mtOMML_EQUATION
                        ueInfo.id = ueInfo.id + 1
                    End If

                End If

            ElseIf stat <> mtNOT_EQUATION Then
                WriteLog "Error occurred in MTGetOMMLFromClipboardRTF"
                If ShowTransformError(stat, ueInfo.title) = STE_CANCEL Then
                    GoTo bye
                End If
            Else
                WriteLog "No OMML data detected"
            End If
       End If
    Next index
bye:
WriteLog "Exiting UpdateOMMLImagesInRange"
End Sub

' returns True on success, False on Failure
Function InsertMTEqnFromOMML(rng As Range, ommlStr As String) As Boolean

    Dim mmlStr As String
    Dim stat As Long

    InsertMTEqnFromOMML = False

    WriteLog "Entering InsertMTEqnFromOMML"

    On Error GoTo bye
#If Win32 Then
    mmlStr = XformOMML2MML(ommlStr)
    If mmlStr = "omml_xform_error" Then
        Exit Function
    End If
#Else
    mmlStr = ommlStr
#End If

    InsertMTEqnFromOMML = InsertMTEqnFromMML(rng, mmlStr, True)
bye:
End Function

Function InsertMTEqnFromMML(rng As Range, mmlStr As String, checkForDisplay As Boolean) As Boolean

    Dim blankEqnPath As String
    blankEqnPath = GetMathTypeDir & Application.PathSeparator & "Office Support" & Application.PathSeparator & "BlankEqn.doc"

    If (dir(blankEqnPath) <= "") Then
        MsgBox MTLib.GetUserString2("1680", "3305", "Required file (") & blankEqnPath & MTLib.GetUserString2("1681", "3306", ") could not be found. Please reinstall MathType")
        Exit Function
    End If

    If (rng.start <> rng.end) Then
        rng.delete
    End If

    Selection.InsertAfter ". "
    Selection.moveLeft

RemoveQuoteFieldCode:
    rng.InsertFile blankEqnPath, "MTBlankEqn"
    rng.Expand (wdWord)

    WriteLog "Blank MT Eqn inserted"

    Dim mtEqn As InlineShape
    If rng.InlineShapes.count = 1 Then
        Set mtEqn = rng.InlineShapes(1)
    Else
        Dim currField As Field
        Dim deletedQuoteField As Boolean
        deletedQuoteField = False
        For Each currField In rng.Fields
            If currField.Type = wdFieldQuote Then
                currField.delete
                deletedQuoteField = True
            End If
        Next
        If deletedQuoteField Then
            GoTo RemoveQuoteFieldCode
        Else
            Set mtEqn = rng.InlineShapes(rng.InlineShapes.count)
        End If
    End If

    If Not SetMTData(mtEqn, mmlStr) Then
        GoTo bye
    End If

    WriteLog "Detecting display equation status"
    DeleteEquationMarkers
    Selection.Collapse wdCollapseEnd 'display eqn: remove whole line selection
    Selection.MoveRight wdCharacter, 1, wdMove 'inline eqn: move to end of eqn
    Selection.MoveRight wdCharacter, 2, wdExtend
    Selection.delete

    If checkForDisplay Then

        Dim atEnd, atBeginning
        atEnd = False
        atBeginning = False

        ' test to see if the equation is alone in a paragraph
        rng.Select
        With Selection
            .Collapse wdCollapseEnd

            ' are we at the end of a paragraph ?
            If Strings.left$(.Text, 1) = Strings.Chr(13) Then
                atEnd = True
            End If
            ' are we at the beginning of a paragraph or top of a column?
            Dim numMoved As Integer
            numMoved = .moveLeft(wdCharacter, 2, wdMove)
            If (Strings.left$(.Text, 1) = Strings.Chr(13) Or _
                Strings.left$(.Text, 1) = Strings.Chr(14) Or _
                numMoved <> 2) Then
                atBeginning = True
            End If
        End With

        'omml display eqns are the only thing in a paragraph
        If (atBeginning And atEnd) Then
            rng.Select
            ApplyDisplayStyle rng
            rng.InsertBefore vbTab
        End If

    End If

    InsertMTEqnFromMML = True
bye:
    WriteLog "Exiting InsertMTEqnFromMML"

End Function

Function GetOMMLFromOMMLEqn(ommlEqn) As String

    GetOMMLFromOMMLEqn = ""

    On Error GoTo bye

    Dim rng
    Set rng = ommlEqn.Range
    If Val(Application.version) >= kWord2007 Then
        Dim woXML As String
        woXML = rng.WordOpenXML
        GetOMMLFromOMMLEqn = woXML
    End If

bye:

End Function

Function XformOMML2MML(ommlStr) As String

#If Win32 Then

    XformOMML2MML = ""

    On Error GoTo err

    'get path to omml2mml.xsl
    Dim ssPath As String
    ssPath = GetOmml2MmlXslFile

    'check for file not found
    If Len(ssPath) < 1 Then
        ShowOmml2mmlFileError
        GoTo err
    End If

    'read omml2mml.xsl
    Dim Source As MSXML2.DOMDocument30
    Set Source = New MSXML2.DOMDocument30
    Dim style As MSXML2.DOMDocument30
    Set style = New MSXML2.DOMDocument30
    style.Load ssPath

    'check for correct version of omml2mml.xsl
    If Not CheckVersion_Omml2MmlXslFile2(ssPath) Then
        ShowOmml2mmlFileError
        GoTo err
    End If

    Source.LoadXML ommlStr

    Dim outStr As String
    outStr = Source.transformNode(style.DocumentElement)

    'Debug.Print outStr
    XformOMML2MML = outStr

    Exit Function

#End If

err:
    XformOMML2MML = "omml_xform_error"

End Function

Private Sub ShowOmml2mmlFileError()

    Dim Message As String
    Dim title As String

    title = MTLib.GetUserString2("1682", "3307", "Problem Converting OMML to MathML")
    Message = MTLib.GetUserString2("1683", "3308", "The style sheet (omml2mml.xsl) required for this operation was not found or is out of date. Click Yes to find out how to get it or No to cancel.")

    If MsgBox(Message, vbYesNo + vbCritical, title) = vbYes Then
        MTGetURL mturlMATHTYPE_OMML2MATHMLXSL, True, "", 0
    End If

End Sub

Function CheckVersion_Omml2MmlXslFile2(fileName As String) As Boolean

    CheckVersion_Omml2MmlXslFile2 = False

#If Win32 Then

    Const kSIGNATURE_SP2 As String = "&#x2146"
    Const kSIGNATURE_BETA As String = "&#2146"

    Dim fileContents As String
    Dim hFile As Long

    'open, read, close file
    hFile = FreeFile
    Open fileName For Input As #hFile
    fileContents = Input$(LOF(hFile), hFile)
    Close #hFile

    'search for SP2 signature
    Dim strLoc As Long
    strLoc = InStr(1, fileContents, kSIGNATURE_SP2, vbTextCompare)
    If (strLoc >= 1) Then
        CheckVersion_Omml2MmlXslFile2 = True
    Else
        'search for beta signature
        strLoc = InStr(1, fileContents, kSIGNATURE_BETA, vbTextCompare)
        If (strLoc >= 1) Then
            CheckVersion_Omml2MmlXslFile2 = True
        End If
    End If
#End If

End Function

#If Win32 Then
Function CheckVersion_Omml2MmlXslFile(xmlDoc As MSXML2.DOMDocument30) As Boolean

    CheckVersion_Omml2MmlXslFile = False

    On Error GoTo err

    Const kXMLSignatureSearch As String = "xsl:when"
    Const kXMLSignature As String = "string-length($chAcc)=0"

    Dim strLoc As Long
    Dim currentItem As IXMLDOMNode
    Dim nodeList As IXMLDOMNodeList
    Set nodeList = xmlDoc.getElementsByTagName(kXMLSignatureSearch)

    For Each currentItem In nodeList
        strLoc = InStr(1, currentItem.XML, kXMLSignature, vbTextCompare)
        If (strLoc >= 1) Then
            CheckVersion_Omml2MmlXslFile = True
            Exit Function
        End If
    Next

err:

End Function
#End If

Function GetOmml2MmlXslFile() As String

    Dim ommlFullName As String

#If Win32 Then
    'check the MathType registry keys for the location of the file
    If (FileExists(GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_OMML2MML_XSL_DIRNAME), _
                   GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_OMML2MML_XSL_FILENAME), _
                   ommlFullName)) Then
        GetOmml2MmlXslFile = ommlFullName
        Exit Function
    End If

    'check installed location of Office
    If (FileExists(GetOfficePath(Val(Application.version)), _
                   kOMML2MML_XSL_FILENAME, _
                   ommlFullName)) Then
        GetOmml2MmlXslFile = ommlFullName
        Exit Function
    End If

    'check the MathType\Office Support folder
    If (FileExists(GetMathTypeDir & "\Office Support", _
                   kOMML2MML_XSL_FILENAME, _
                   ommlFullName)) Then
        GetOmml2MmlXslFile = ommlFullName
        Exit Function
    End If
#End If

    'otherwise return a null string
    GetOmml2MmlXslFile = ""

End Function

Function GetOfficePath(officeVersion As Integer) As String

    GetOfficePath = ""

#If Win32 Then
    Dim officePath As String
    Dim filePath As String
    Dim section As String
    Dim section2 As String
    Dim progFilesOffice As String

    If (officeVersion = kWord2007) Then
        section = msoreg_WORD2007DIR_SECTION
        section2 = msoreg_WORD2007DIR_SECTION2
        progFilesOffice = "Microsoft Office\Office12"
    ElseIf (officeVersion = kWord2010) Then
        section = msoreg_WORD2010DIR_SECTION
        section2 = msoreg_WORD2010DIR_SECTION2
        progFilesOffice = "Microsoft Office\Office14"
    ElseIf (officeVersion = kWord2013) Then
        section = msoreg_WORD2013DIR_SECTION
        section2 = msoreg_WORD2013DIR_SECTION2
        progFilesOffice = "Microsoft Office\Office15"
    ElseIf (officeVersion = kWord2016) Then
        section = msoreg_WORD2016DIR_SECTION
        section2 = msoreg_WORD2016DIR_SECTION2
        progFilesOffice = "Microsoft Office\root\Office16"
    Else
        Exit Function
    End If

    'check the Office install folder - look for location in the MS Office hive of the registry
    If (FileExists(GetPreference(HKEY_CURRENT_USER, section, msoreg_WORDDIR_KEY), _
                   "", _
                   officePath)) Then
      If (FileExists(officePath, kOMML2MML_XSL_FILENAME, filePath)) Then
        GetOfficePath = officePath
        Exit Function
      End If
    End If

    'check the Office install folder in Program Files
    If (FileExists(GetSpecialFolderA(CSIDL_PROGRAM_FILES) & progFilesOffice, _
                   "", _
                   officePath)) Then
      If (FileExists(officePath, kOMML2MML_XSL_FILENAME, filePath)) Then
        GetOfficePath = officePath
        Exit Function
      End If
    End If

    'check the Office install folder - look for location in the MS Office hive of the registry
    If (FileExists(GetPreference(HKEY_LOCAL_MACHINE, section2, msoreg_WORDDIR_KEY2), _
                   "", _
                   officePath)) Then
      If (FileExists(officePath, kOMML2MML_XSL_FILENAME, filePath)) Then
        GetOfficePath = officePath
        Exit Function
      End If
    End If
#End If

End Function

#If Win32 Then
Public Function GetSpecialFolderA(ByVal eSpecialFolder As mceIDLPaths) As String
    On Error GoTo done

    Dim Ret As Boolean
    Dim str As String
    str = Strings.Space$(260)

    Ret = SHGetSpecialFolderPath(0, str, eSpecialFolder, False)
    If Strings.Trim$(str) <> Strings.Chr(0) Then
        str = Strings.left$(str, InStr(str, Strings.Chr(0)) - 1) & "\"
    End If

    GetSpecialFolderA = str

done:
    GetSpecialFolderA = ""
End Function
#End If

'combines path and name as a full path and returns it in fullName
'function returns true if the file exists, else false
Private Function FileExists(path As String, name As String, ByRef fullName As String) As Boolean

    FileExists = False

    If Len(path) > 0 Then
        If Strings.right$(path, 1) <> "\" Then
            path = path & "\"
        End If
    End If

    fullName = path & name

    If Len(dir$(fullName)) > 0 And Len(fullName) > 0 Then
      If Not dir(fullName, vbDirectory) = vbNullString Then
        FileExists = True
      End If
    End If

End Function

' Transform an mt eqn to text.  Called by OMML conversion routines
' Modifies user Selection and Clipboard
Function XformMTEqnToText(mtEqn As InlineShape, ByRef ueInfo As UpdateInfo, ByRef xformErr As Boolean) As Boolean

    Dim stat As Long
    Dim retval As Boolean
    Dim MTEF() As Byte
    Dim MTEFLen As Long

    XformMTEqnToText = True
    xformErr = False

    On Error GoTo bye

    'copy to clipboard & transform it
    mtEqn.Select

    retval = GetMTEF(mtEqn, MTEF, MTEFLen)
    If retval = True Then
        'check if any other error occurred
        If stat = mtOK Then
            If ueInfo.update Then
                Selection.delete
                If ueInfo.kind = kUE_FILE Then
                    'insert filename
                    Selection.InsertAfter GetExportFilename(ueInfo)
                Else    'graphic or text
                    ConvertAndInsertEquation ueInfo, Selection.Range, MTEF, MTEFLen, vbNullString
                End If
            End If
            ueInfo.id = ueInfo.id + 1
        'show error message, abort if user Cancels
        ElseIf stat < 0 Then
            xformErr = True
            If ShowTransformError(stat, ueInfo.title) = STE_CANCEL Then
                XformMTEqnToText = False
                Exit Function
            End If
        End If
    End If
    Exit Function

bye:

End Function

'Updates all field equations (Word EQ fields and MathType 1.x macro equations).
'Returns 0 if OK, -1 to abort update
Public Function UpdateFields(docRange As Long, ByRef ueInfo As UpdateInfo) As Long

    Dim stat As Long, doPaste As Long
    Dim curField As Field
    Dim selRange As Range
    Dim fieldText As String

    UpdateFields = 0

    'if "Update Whole Document" was selected, set the range to the whole document
    'otherwise, set the range to the current selection
    If docRange = mt_RANGE_DOCUMENT Then
        Set selRange = ActiveDocument.Range
    Else
        Set selRange = Selection.Range
    End If

    'loop through the graphics
    For Each curField In selRange.Fields
        curField.Select

        'see if it is an equation
        fieldText = Strings.LCase$(curField.Code.Text)
        If (curField.Type = wdFieldFormula) Or _
            (Len(fieldText) > 1 And _
             InStr(1, fieldText, "edittexteqn", vbBinaryCompare) <> 0 Or _
                InStr(1, fieldText, "editdispeqn", vbBinaryCompare)) <> 0 Then
            ' "EditTextEqn", "EditDispEqn"
            'copy to clipboard & transform it
            Selection.Copy
            DelayClipboardCopy

            stat = TransformEquation(ueInfo)
            If stat <> mtNOT_EQUATION Then
                'if it was an equation, ask user if we're prompting
                doPaste = promptUser(ueInfo)

                'if user cancelled, exit loop
                If doPaste = PU_CANCEL Then
                    UpdateFields = -1
                    GoTo bye
                ElseIf doPaste = PU_UPDATE Then
                    'if we're going to paste, check if any other error occurred
                    If stat = mtOK Then
                        If ueInfo.update Then
                            curField.delete
                            If ueInfo.kind = kUE_FILE Then
                                'insert filename
                                Selection.InsertAfter GetExportFilename(ueInfo)
                            Else    'graphic or text
                                PasteTranslatedEquation ueInfo, wdInLine
                            End If
                        End If
                        ueInfo.id = ueInfo.id + 1
                        UpdateCount mtWORD_EQUATION

                    'show error message, abort if user clicked Cancel
                    ElseIf stat < 0 Then
                        If ShowTransformError(stat, ueInfo.title) = STE_CANCEL Then
                            UpdateFields = -1
                            GoTo bye
                        End If
                    End If
                End If
            End If
        End If
    Next curField

bye:
End Function

Public Function UpdateTextEqns(docRange As Long, ByRef ueInfo As UpdateInfo) As Long

    UpdateTextEqns = 0

    'process selection
    If docRange = mt_RANGE_SELECTION Then
        If UpdateTextEqnsInRange(Selection.Range, ueInfo) = -1 Then
            UpdateTextEqns = -1
        End If
        Exit Function
    End If

    'save view, and set it to Print Preview
    Dim saveView As WdViewType
    saveView = ActiveWindow.View.Type
    ActiveWindow.View.Type = wdPrintView

    'process entire document
    Dim currentRange As Range
    For Each currentRange In ActiveDocument.StoryRanges
        If UpdateTextEqnsInRange(currentRange, ueInfo) = -1 Then
            UpdateTextEqns = -1
        End If
    Next currentRange

    'restore view
    ActiveWindow.View.Type = saveView

End Function

'Updates text equations that have MT4 (or greater) comments.
'Finds eqn by looking for the pattern
'   MathType<sep>Translator<sep>        where sep may vary
'Determines end of text eqn by looking for
'   MathType<sep>End<sep>
'Both the start and end of text eqn are assumed to begin on new line
'and both lines are included in selection which gets replaced.
Public Function UpdateTextEqnsInRange(docRange As Range, ByRef ueInfo As UpdateInfo) As Long

    Const mtTE_MATHTYPE As String = "MathType"
    Const mtTE_TRANSLATOR As String = "Translator"
    Const mtTE_END As String = "End"
    Const mtTE_MTEF As String = "MTEF"

    Dim stat As Long, doPaste As Long
    Dim dist As Long, oldLen As Long, sep$, eqnEnd$
    Dim count1 As Long, count2 As Long, eqnStart As Long
    Dim selRange As Range
    Dim eqnRange As Range
    Dim lenTrans As Long, lenMT As Long
    Dim eqnRangeEnd As Long

    UpdateTextEqnsInRange = 0
    Set selRange = docRange

    selRange.Select

    'set selection to match the desired range
    Selection.start = selRange.start
    Selection.end = selRange.end

    lenMT = Len(mtTE_MATHTYPE)
    lenTrans = Len(mtTE_TRANSLATOR)

    Do While True
        'look for "MathType<sep>Translator<sep>count1<sep>count2<sep>"
        'count1 = num chars prior to "MathType" included in the comment
        'count2 = num chars following "MathType<sep>End" included in the comment
        Selection.find.Execute FindText:=mtTE_MATHTYPE, MatchCase:=False, _
            MatchWholeWord:=False, forward:=True, Wrap:=wdFindStop

        'Figure out if we found something and if it is in our selected range
        If Not Selection.find.found Or Not (Selection.Range.InRange(selRange)) Then
            GoTo bye
        End If

        dist = Selection.MoveRight(wdCharacter, lenTrans + 2, wdExtend)
        If dist = 0 Then GoTo bye   'can't extend selection, so quit search
        'ensure we've found an equation
        If Strings.Mid(Selection.Text, lenMT + 2, lenTrans) <> mtTE_TRANSLATOR Then
            'reset selection to after "MathType" that was found
            Selection.MoveRight wdCharacter, -dist, wdExtend
            GoTo continue
        End If
        sep$ = Strings.Mid(Selection.Text, lenMT + 1, 1)
        If Strings.Mid(Selection.Text, lenMT + lenTrans + 2, 1) <> sep$ Then
            'if error, reset selection to after "MathType" that was found
            Selection.MoveRight wdCharacter, -dist, wdExtend
            GoTo continue
        End If

        'find start count
        Selection.MoveRight wdCharacter, 2, wdExtend
        count1 = ParseCommentCount( _
            Strings.Mid(Selection.Text, Len(Selection.Text) - 2, 2), sep$, 2)
        Selection.MoveRight wdCharacter, 2, wdExtend

        'find end count
        count2 = ParseCommentCount( _
            Strings.Mid(Selection.Text, Len(Selection.Text) - 2, 2), sep$, 1)
        count2 = count2 - 1
        If count2 < 0 Then
            count2 = 0
        End If

        'locate & save comment start (Outline mode can change this during Sel.Extend)
        Selection.MoveStart wdCharacter, -count1
        eqnStart = Selection.start

        'extend the selection to the end of the equation
        eqnEnd$ = Strings.LCase$(mtTE_MATHTYPE + sep$ + mtTE_END + sep$)
        Do Until InStr(1, Strings.LCase$(Selection.Text), eqnEnd$, vbBinaryCompare) > 0
            'extend selection to end of equation, avoid infinite looping
            'by checking that selection len increases with each extension
            oldLen = Len(Selection.Text)
            Selection.Extend sep$
            If oldLen = Len(Selection.Text) Then GoTo bye
        Loop

        'adjust sel end to ensure it's end of eqnEnd$
        If Strings.right$(Selection.Text, Len(eqnEnd$)) <> eqnEnd$ Then
            Selection.MoveEnd wdCharacter, _
                InStr(1, Strings.LCase$(Selection.Text), eqnEnd$, vbBinaryCompare) - Len(Selection.Text) + Len(eqnEnd$) - 1
        End If

        'extend selection to the end of the comment following End<sep>
        'allow for count<sep>count<sep> missing (early MT4 betas didn't have them)
        Selection.MoveEnd wdCharacter, 2
        If Strings.right(Selection.Text, 1) = sep$ Then
            Selection.MoveEnd wdCharacter, 2    'skip next count<sep>
        Else
            Selection.MoveEnd wdCharacter, -2   'return to end
        End If
        Selection.MoveEnd wdCharacter, count2

        'check that selected equation contains MTEF tag
        If InStr(1, Strings.LCase$(Selection.Text), Strings.LCase$(mtTE_MATHTYPE + sep$ + mtTE_MTEF + sep$), vbBinaryCompare) = 0 Then
            GoTo continue
        End If

        'final check that we're still inside the desired range
        If Not (Selection.Range.InRange(selRange)) Then GoTo bye

        'define eqnRange using saved eqnStart
        Set eqnRange = Selection.Range
        eqnRange.start = eqnStart

        Dim strTextEqn As String
        strTextEqn = eqnRange.Text
        If (Len(strTextEqn) > 0) Then

            'if it was an equation, get update action
            doPaste = promptUser(ueInfo)

            'exit if cancelled
            If doPaste = PU_CANCEL Then
                UpdateTextEqnsInRange = -1
                GoTo bye
            ElseIf doPaste = PU_UPDATE Then
                'if we're going to paste, check if any other error occurred
                If stat = mtOK Then
                    If ueInfo.update Then
                        Selection.Collapse wdCollapseEnd
                        eqnRangeEnd = eqnRange.end
                        If ueInfo.kind = kUE_FILE Then
                            'insert filename
                            Selection.InsertAfter GetExportFilename(ueInfo)
                        Else    'graphic or text
                            Dim dummyByteArray() As Byte
                            ConvertAndInsertEquation ueInfo, eqnRange, dummyByteArray, 0, strTextEqn
                            PostProcessConvertedEquation ueInfo, wdInLine
                        End If

                        'handle situation at end of doc w/mo trailing CR...
                        If eqnRange.end > eqnRangeEnd Then
                           eqnRange.end = eqnRangeEnd - 1
                        End If

                        'when converting from Text->Text, delete will leave the trailing CR
                        'so insert a 'Z' after the CR to ensure its deletion
                        eqnRange.InsertAfter "Z"
                        eqnRange.delete
                        If ueInfo.kind = kUE_GRAPHIC Then
                            'skip graphic we just pasted
                            Selection.move wdCharacter, 1
                        End If
                    End If
                    ueInfo.id = ueInfo.id + 1
                    UpdateCount mtTEXT_EQUATION
                'show error message, abort if user Cancels
                ElseIf stat < 0 Then
                    If ShowTransformError(stat, ueInfo.title) = STE_CANCEL Then
                        UpdateTextEqnsInRange = -1
                        GoTo bye
                    End If
                End If
            End If
        End If

continue:
        Selection.Collapse wdCollapseEnd
    Loop
bye:
End Function

'Ugly kludge to compensate for fact that Word's paste behavior
'appears to be OS dependent. If text on clipboard contains a trailing CR-LF,
'it gets truncated on Win 9x and XP, but kept on NT. MS couldn't offer any help,
'so we always append a 'Z' to text eqns on clipboard to avoid this problem.
'This function trims off the trailing 'Z'.
Private Sub CleanUpPasteKludge()
    Selection.moveLeft wdCharacter, 1, wdExtend
    If Selection.Text = "Z" Then
        Selection.delete
    Else
        Selection.MoveRight wdCharacter, 1, wdMove
    End If
End Sub

'Parses <comment$> for count and returns found value, or <default>.
'Comment is of form '<sep$>n' where n is a single digit.
Private Function ParseCommentCount(comment$, sep$, default As Long) As Long
    If Strings.left$(comment$, 1) = sep$ Then
        ParseCommentCount = Val(Strings.right$(comment$, 1))
    Else
        ParseCommentCount = default
    End If
End Function

'Transforms clipboard graphic into an equation.
'Format depends on how MathType has been configured by a
'previous call to MTXFormSetTranslator.
'The transformed equation is left on the clipboard.
'Returns mtOK, mtNOT_EQUATION or other error code
Private Function TransformGraphicEquation(ByRef ueInfo As UpdateInfo) As Long
    TransformGraphicEquation = mtNOT_EQUATION

    'The following DoEvents statement was added as a fix
    'Word 2002 hangs for 15-20 seconds in MTEquationOnClipboard() call
    'during Export Equations
    DoEvents

    'Use API call to check clipboard contents first.
    'Fails to detect WMFs with MTEF after Word 2000.  See MT-1131
    If MTEquationOnClipboard() = mtNOT_EQUATION Then
        Exit Function
    End If

    TransformGraphicEquation = TransformEquation(ueInfo)
End Function

'Transforms clipboard graphic into an equation.
'Format depends on how MathType has been configured by a
'previous call to MTXFormSetTranslator.
'The transformed equation is left on the clipboard.
'Returns mtOK, mtNOT_EQUATION or other error
Private Function TransformEquation(ByRef ueInfo As UpdateInfo) As Long
    Dim stat As Long
    Dim dummyStr1 As String, dummyStr2 As String, path As String
    Dim dummyDims As MTAPI_DIMS
    Dim destFormat As Integer
    Dim dest As Integer

    On Error GoTo err

    stat = mtNOT_EQUATION

    'as long as everything's OK, update the equation
    'set aside some buffers
    dummyStr1 = Strings.Space(1)
    dummyStr2 = Strings.Space(1)
    With dummyDims
        .baseline = 0
        .bounds.bottom = 0
        .bounds.left = 0
        .bounds.right = 0
        .bounds.top = 0
    End With

    If ueInfo.kind = kUE_FILE Then
        dest = mtxfmFILE
        destFormat = GetMTExportType(ueInfo.fileType)
        path = ueInfo.fileName & Application.PathSeparator & MTExportEquations.GetFileNameFromPattern(ueInfo.filePattern, ueInfo.id)
    Else
        dest = mtxfmCLIPBOARD
        destFormat = mtxfmTEXT
        path = Strings.Space(1)
    End If

    'do the update
    stat = MTXFormEqn(mtxfmCLIPBOARD, mtxfmTEXT, dummyStr1, 1, dest, destFormat, dummyStr2, 1, path, dummyDims)
    If stat < 0 Then
        Select Case stat
        Case mtBAD_PATH, mtFILE_ACCESS, mtFILE_WRITE_ERROR, mtTRANSLATOR_ERROR, mtPREFERENCE_ERROR
            stat = stat
        Case Else
            stat = mtNOT_EQUATION
        End Select
    End If
    GoTo bye

err:
    If err.Number = 5690 Or err.Number = 4198 Then
        'the user has revisions on, and this is an old revision that has been deleted
        stat = -2
        Resume bye
    Else
        err.Raise err.Number
        Stop
    End If
bye:
    TransformEquation = stat
End Function

Public Function ConvertAndInsertEquation(ByRef ueInfo As UpdateInfo, eqnRng As Range, MTEF() As Byte, MTEFLen As Long, strTextEqn As String) As Long

    If (ueInfo.kind = kUE_GRAPHIC) Then
        ConvertAndInsertEquation = ConvertAndInsertEquation_MathType(eqnRng, MTEF, MTEFLen, strTextEqn)
    ElseIf (ueInfo.kind = kUE_TEXT Or ueInfo.kind = kUE_FILE) Then
        ConvertAndInsertEquation = ConvertAndInsertEquation_Text(eqnRng, MTEF, MTEFLen, strTextEqn, ueInfo)
    End If

End Function

'convert MTEF or Text equation to a MathType equation
Public Function ConvertAndInsertEquation_MathType(eqnRng As Range, MTEF() As Byte, MTEFLen As Long, strTextEqn As String) As Long

    ConvertAndInsertEquation_MathType = mtERROR

    Dim blankEqnPath As String
    Dim mtEqn As InlineShape
    Dim stat As Long
    Dim myObj As Object
    Dim rngStart As Long, rngEnd As Long

    'save eqnRng postions
    rngStart = eqnRng.start
    rngEnd = eqnRng.end

    'insert blankeqn.doc
    blankEqnPath = GetMathTypeDir & Application.PathSeparator & "Office Support" & Application.PathSeparator & "BlankEqn.doc"
    eqnRng.Collapse wdCollapseEnd
    eqnRng.InsertFile blankEqnPath, "MTBlankEqn"
    eqnRng.start = rngStart

    'set range to blank eqn
    eqnRng.SetRange eqnRng.start, eqnRng.end + 27

    'get ole object
    If eqnRng.InlineShapes.count > 0 Then
        Set mtEqn = eqnRng.InlineShapes(1)
    Else
        Exit Function
    End If

    'activate it
    ActivateMT mtEqn
    Set myObj = mtEqn.OLEFormat.Object

    'set up for convert
    Dim path As String, strDest As String
    Dim dummyDims As MTAPI_DIMS
    With dummyDims
        .baseline = 0
        .bounds.bottom = 0
        .bounds.left = 0
        .bounds.right = 0
        .bounds.top = 0
    End With

    path = Strings.Space(1)
    Const STRDESTLEN As Integer = 10000 'todo
    strDest = Strings.Space(STRDESTLEN)

    'convert equation without a translator to convert to MTEF
    stat = MTXFormSetTranslator(0, vbNull)

    'convert the equation
    If (Len(strTextEqn) > 0) Then
        stat = MTXFormEqn(mtxfmLOCAL, mtxfmTEXT, strTextEqn, Len(strTextEqn), mtxfmLOCAL, mtxfmTEXT, strDest, STRDESTLEN, path, dummyDims)
    Else
        stat = MTXFormEqnBytes(mtxfmLOCAL, mtxfmMTEF, MTEF(0), MTEFLen, mtxfmLOCAL, mtxfmTEXT, strDest, STRDESTLEN, path, dummyDims)
    End If

    'set the equation with the MTEF string
    Dim bTextEqn() As Byte
    bTextEqn = strDest
    stat = MTSetEqnFromLangStr(myObj, mtlangMATHML, bTextEqn(0), Len(strDest))

    'clean up
    eqnRng.SetRange rngStart, rngEnd
    stat = ShutdownMT(mtEqn)
    ConvertAndInsertEquation_MathType = stat

End Function

'convert MTEF or Text equation to a text equation
Public Function ConvertAndInsertEquation_Text(eqnRng As Range, MTEF() As Byte, MTEFLen As Long, strTextEqn As String, ByRef ueInfo As UpdateInfo) As Long

    ConvertAndInsertEquation_Text = mtERROR

    Dim stat As Long
    Dim path As String, strDest As String
    Dim dummyDims As MTAPI_DIMS
    Dim destFormat As Integer
    Dim dest As Integer

    On Error GoTo err

    'initialize vars
    stat = mtNOT_EQUATION

    With dummyDims
        .baseline = 0
        .bounds.bottom = 0
        .bounds.left = 0
        .bounds.right = 0
        .bounds.top = 0
    End With

    Const STRDESTLEN As Integer = 10000
    strDest = Strings.Space(STRDESTLEN)

    If ueInfo.kind = kUE_FILE Then
        'export equation
        dest = mtxfmFILE
        destFormat = GetMTExportType(ueInfo.fileType)
        path = ueInfo.fileName & Application.PathSeparator & MTExportEquations.GetFileNameFromPattern(ueInfo.filePattern, ueInfo.id)
    Else
        'convert equation
        dest = mtxfmLOCAL
        destFormat = mtxfmTEXT
        path = Strings.Space(1)
    End If

    'do the conversion
    If (Len(strTextEqn) > 0) Then
        'convert a string
        stat = MTXFormEqn(mtxfmLOCAL, mtxfmTEXT, strTextEqn, Len(strTextEqn), dest, destFormat, strDest, STRDESTLEN, path, dummyDims)
    Else
        'convert an array of bytes
        stat = MTXFormEqnBytes(mtxfmLOCAL, mtxfmMTEF, MTEF(0), MTEFLen, dest, destFormat, strDest, STRDESTLEN, path, dummyDims)
    End If
    If stat < 0 Then
        'error
        Select Case stat
        Case mtBAD_PATH, mtFILE_ACCESS, mtFILE_WRITE_ERROR, mtTRANSLATOR_ERROR, mtPREFERENCE_ERROR
            stat = stat
        Case Else
            stat = mtNOT_EQUATION
        End Select
    ElseIf ueInfo.kind <> kUE_FILE Then
        'insert translated equation
        strDest = Strings.Trim(strDest)
        strDest = RemoveNull(strDest)
        Selection.Text = strDest

        'apply style to inserted text
        Dim pasteStart As Long

        pasteStart = Selection.Range.start
        If ueInfo.kind = kUE_TEXT Then
            'CleanUpPasteKludge
            Dim aRange As Range
            Set aRange = Selection.Range
            aRange.SetRange start:=pasteStart, end:=Selection.Range.end
            Dim noApply As String
            noApply = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_NOAPPLY_CONVERTED_EQN_STYLE)
            If noApply = "0" Or Len(noApply) = 0 Then
                aRange.style = mtstyle_CONVERTED_EQUATION
            End If
        End If
    End If
    GoTo bye

err:
    If err.Number = 5690 Or err.Number = 4198 Then
        'the user has revisions on, and this is an old revision that has been deleted
        stat = -2
        Resume bye
    Else
        err.Raise err.Number
        Stop
    End If
bye:
    ConvertAndInsertEquation_Text = stat

End Function

Public Function GetMTEF(curShape As InlineShape, ByRef MTEF() As Byte, ByRef MTEFLen As Long) As Boolean

    GetMTEF = False

    If (curShape.OLEFormat Is Nothing) Then
        Exit Function
    End If

    Dim myObj As Object
    Dim stat As Long

    ActivateMT curShape
    Set myObj = curShape.OLEFormat.Object

    MTEFLen = 0
    Dim temp(10) As Byte

    stat = MTGetLangBytesFromEqn(myObj, mtlangMTEF, temp(0), MTEFLen)
    If (stat = mtOK And MTEFLen > 0) Then
        ReDim MTEF(MTEFLen)
        stat = MTGetLangBytesFromEqn(myObj, mtlangMTEF, MTEF(0), MTEFLen)
        If stat = mtOK Then
            GetMTEF = True
        End If
    End If
    
    stat = ShutdownMT(curShape)

End Function

Public Sub SetQuiet(MBQuiet As Boolean)
    gMBQuiet = MBQuiet
End Sub

'Displays error message based on transform result code.
'Returns STE_CONTINUE to continue, STE_CANCEL to stop
Public Function ShowTransformError(id As Long, title As String) As Long

    Dim stat As Long, Message As String, screenFreeze As Boolean

    'force display of selected equation that caused the error
    If Application.ScreenUpdating = False Then
        screenFreeze = True
        SetScreenUpdate True
        Selection.Range.Select
    End If

    Select Case id
    Case mtTRANSLATOR_ERROR
        Message = GetUserString2("1626", "3226", "There was an error translating to the new format. This equation was not updated.")
    Case mtPREFERENCE_ERROR
        Message = GetUserString2("1628", "3228", "There was a problem sending the preference information (styles, sizes, and spacing settings) to MathType. This equation was not updated.")
    Case mtBAD_PATH, mtFILE_ACCESS, mtFILE_WRITE_ERROR
        Message = GetUserString2("1642", "3242", "There was a problem writing this equation to disk (e.g. the disk may be full). This equation was not exported.")
    Case Else
        Message = GetUserString2("1629", "3229", "There was an unknown error updating this equation. This equation was not updated.")
    End Select

    gMBStyle = mt_MBYESNO
    gMBCaption = title
    gMBMessage = Message + " " + MTLib.GetUserString2("1641", "3241", "Do you want to continue?")
    If Not gMBQuiet Then
        Beep
        MTMsgBox.Show
    End If
    WriteLog Message

    If gMBQuiet Or gMBResult = mt_MBYES Then
        ShowTransformError = STE_CONTINUE
    Else
        ShowTransformError = STE_CANCEL
    End If

    'restore screen update state
    If screenFreeze Then SetScreenUpdate False
End Function

'Returns path based on update info
Private Function GetExportFilename(ByRef ueInfo As UpdateInfo) As String
    GetExportFilename = "<<" & _
        MTExportEquations.GetFileNameFromPattern(ueInfo.filePattern, ueInfo.id) & ">>"
End Function

'Returns XFormEqn ID for fileType
Private Function GetMTExportType(fileType As Integer) As Integer
    Select Case fileType
    Case kFTEPS_OSPICT
        GetMTExportType = mtxfmEPS_WMF
    Case kFTEPS_TIFF
        GetMTExportType = mtxfmEPS_TIFF
    Case kFTEPS_NONE
        GetMTExportType = mtxfmEPS_NONE
    Case kFTGIF
        GetMTExportType = mtxfmGIF
    Case kFTOSPICT
        GetMTExportType = mtxfmPICT
    Case kFTPDF
        GetMTExportType = mtxfmPDF
    End Select
End Function

'Depending on update info either returns PU_UPDATE or asks user if they want to update eqn
'Returns PU_UPDATE, PU_SKIP, PU_CANCEL (uses our custom MgsBox as VBA's doesn't update)
Private Function promptUser(ByRef ueInfo As UpdateInfo) As Long
    promptUser = PU_UPDATE
    If ueInfo.prompt Then
        gMBStyle = mt_MBYESNOCANCEL
        gMBCaption = ueInfo.title
        gMBMessage = MTLib.GetUserString2("1624", "3224", "Do you want to update this equation?")
        MTMsgBox.Show
        DoEvents
        Select Case gMBResult
        Case mt_MBYES
            promptUser = PU_UPDATE
        Case mt_MBNO
            promptUser = PU_SKIP
        Case Else
            promptUser = PU_CANCEL
        End Select
    End If
End Function

Public Sub InitEqnCounts()
    OLECount = 0
    PicCount = 0
    MacButtonCount = 0
    WordEqnCount = 0
    TextEqnCount = 0
    OMMLEqnCount = 0
End Sub

'Update the equation counts based on the equation type
Private Sub UpdateCount(eqnType As Long)
    Dim Message As String
    Dim pageNo As Long

    'update the count
    Select Case eqnType
    Case mtOLE_EQUATION, mtOLE2_EQUATION
        OLECount = OLECount + 1
        Message = GetUserString2("1660", "3260", "Processing MathType Equation: ") & OLECount
    Case mtWMF_EQUATION, mtMAC_PICT_EQUATION
        PicCount = PicCount + 1
        Message = GetUserString2("1661", "3261", "Processing Picture Equation: ") & PicCount
    Case mtWORD_EQUATION
        WordEqnCount = WordEqnCount + 1
        Message = GetUserString2("1663", "3263", "Processing Word Equation Field: ") & WordEqnCount
    Case mtTEXT_EQUATION
        TextEqnCount = TextEqnCount + 1
        Message = GetUserString2("1664", "3264", "Processing Text Equation: ") & TextEqnCount
    Case mtMACRO_EQUATION
        MacButtonCount = MacButtonCount + 1
        Message = GetUserString2("1665", "3265", "Processing Macro Equation: ") & MacButtonCount
    Case mtOMML_EQUATION
        OMMLEqnCount = OMMLEqnCount + 1
        Message = GetUserString2("1668", "3268", "Processing OMML Equation: ") & OMMLEqnCount
    Case Else
    End Select
    If Message <> "" Then
        pageNo = Selection.Information(wdActiveEndPageNumber)
        If pageNo > 0 Then
            Message = Message + GetUserString2("1666", "3266", " (page ") + Conversion.str(pageNo) + GetUserString2("1667", "3267", ")")
        End If
        Application.StatusBar = Message
    End If
End Sub

'displays stats about updated equations
Public Sub StatBox(Message$, title$)
    Dim dlgText$, myCR$

    'if at least one equation was updated
    If (OLECount + PicCount + MacButtonCount + WordEqnCount + TextEqnCount + OMMLEqnCount > 0) Then
        myCR$ = Strings.Chr(13) & Strings.Chr(10)

        'set up the dialog text
        dlgText$ = Message$ + myCR$ + myCR$
        If OLECount > 0 Then
            dlgText$ = dlgText$ & OLECount & "  " + MTLib.GetUserString2("1630", "3230", "MathType equation objects") + myCR$
        End If
        If PicCount > 0 Then
            dlgText$ = dlgText$ & PicCount & "  " + MTLib.GetUserString2("1632", "3232", "Non-OLE MathType equation pictures") + myCR$
        End If
        If MacButtonCount > 0 Then
            dlgText$ = dlgText$ & MacButtonCount & "  " + MTLib.GetUserString2("1633", "3233", "MathType 1.x macro equations") + myCR$
        End If
        If WordEqnCount > 0 Then
            dlgText$ = dlgText$ & WordEqnCount & "  " + MTLib.GetUserString2("1634", "3234", "Microsoft Word equation fields") + myCR$
        End If
        If TextEqnCount > 0 Then
            dlgText$ = dlgText$ & TextEqnCount & "  " + MTLib.GetUserString2("1635", "3235", "MathType translator text equations") + myCR$
        End If
        If OMMLEqnCount > 0 Then
            dlgText$ = dlgText$ & OMMLEqnCount & "  " + MTLib.GetUserString2("1637", "3237", "Word 2007 (OMML) equations") + myCR$
        End If


        'beep and display the dialog
        Beep
        MsgBox dlgText$, vbInformation, title$
    Else                        'else if no equations updated
        Beep
        MsgBox MTLib.GetUserString2("1636", "3236", "No equations were found and/or updated."), vbInformation, title$
    End If
End Sub

'Adds a Document Property to the document given a string, a property name, and a
'document object. This function will automatically split the string into multiple
'properties if it is too long. Use ReadDocPropString to read the string.
'Deletes all existing sequentially indexed names (name, name 1, name 2, etc.)
Public Function WriteDocPropString(docToUse As Document, propName As String, propText As String) As Integer
    Dim subString As String, remainder As String
    Dim itExists As Boolean
    Dim aProp As DocumentProperty
    Dim index As Long
    Dim curPropName As String
    Dim nameLen As Long

    On Error GoTo err

    'first delete existing propName property and all derivations (propName 1, ...)
    'Deletes all properties with names that match the base of this property
    '& where the rest of the name can be converted to a number
    MTLib.DeleteDocProperty docToUse, propName

    'when we find a match we delete it and then loop again
    'although less efficient, avoids deleting objects from a
    'collection that we're enumerating
    nameLen = Len(propName)
    itExists = False
    Do
        itExists = False
        For Each aProp In docToUse.CustomDocumentProperties
            If Strings.left(aProp.name, nameLen) = propName Then
                If (IsNumeric(Strings.right(aProp.name, Len(aProp.name) - nameLen))) Then
                    aProp.delete
                    itExists = True
                    Exit For
                End If
            End If
        Next aProp
    Loop While (itExists)

    'set up the variables for the loop
    remainder = propText
    index = 0
    curPropName = propName
    'write the properties
    While Len(remainder) > 0
       'clip off the next chunk
       If Len(remainder) > 255 Then
          subString = Strings.left$(remainder, 255)
          remainder = Strings.right$(remainder, Len(remainder) - 255)
       Else
          subString = remainder
          remainder = ""
       End If

        docToUse.CustomDocumentProperties.Add name:=curPropName, _
           LinkToContent:=False, Type:=msoPropertyTypeString, value:=subString

       'increment the name and counter
       index = index + 1
       curPropName = propName + Conversion.str$(index)
    Wend

    WriteDocPropString = 0
    GoTo done
err:
   WriteDocPropString = -1
done:
End Function

'Deletes document property, OK to call if it doesn't exist
Public Sub DeleteDocProperty(Doc As Document, prop As String)
    On Error GoTo Error
    Doc.CustomDocumentProperties(prop).delete
Error:
End Sub

'Reads a Document Property from the document, a property name, and a
'document object. This function will automatically find all the parts of
'properties split up by WriteDocPropString.
'if property does not exist, returns an empty string ("")
Public Function ReadDocPropString$(docToUse As Document, propName As String)
   Dim propExists As Boolean
   Dim prop As DocumentProperty
   Dim totalString As String
   Dim index As Long
   Dim curPropName As String

   On Error GoTo err

   index = 0
   curPropName = propName
   Do
      'see if the property exists
      On Error GoTo NoProp
      propExists = False
      Set prop = docToUse.CustomDocumentProperties(curPropName)
      propExists = True
NoProp:

      'read it if it's there
      If propExists Then
         totalString = totalString + docToUse.CustomDocumentProperties(curPropName).value
         index = index + 1
         curPropName = propName + Conversion.str$(index)
      End If
   Loop While propExists

   ReadDocPropString$ = totalString
   GoTo done

err:
   ReadDocPropString$ = ""
done:
End Function

'returns True if the active document contains the custom doc property
Public Function DocPropertyExists(Doc As Document, propName As String) As Boolean
    Dim name As String

    DocPropertyExists = False
    On Error GoTo Error
    name = Doc.CustomDocumentProperties(propName).name
    DocPropertyExists = True
Error:
End Function

'returns version number of a newer version of a custom doc property
Public Function NewerDocPropertyVersion(Doc As Document, propName As String, propVer As Long) As Long
    Dim prop As DocumentProperty
    Dim ver As Long

    NewerDocPropertyVersion = 0
    ' look ahead 5 versions
    For ver = propVer + 1 To propVer + 5
        On Error Resume Next
        Set prop = Doc.CustomDocumentProperties(propName & ver)
        If Not prop Is Nothing Then
            NewerDocPropertyVersion = ver
            Exit Function
        End If
    Next ver
End Function

'Adds 'eqn-platform' property to doc
Public Sub AddEqnPlatformProperty(Doc As Document)
    Dim propName As String
    If GetPlatform() = kPlatformMac Then
        propName = mtprop_HAS_MAC_EQNS
    Else
        propName = mtprop_HAS_WIN_EQNS
    End If
    On Error GoTo done
    Doc.CustomDocumentProperties.Add propName, False, msoPropertyTypeBoolean, True
done:
End Sub

'Returns title without making doc dirty
Public Function GetDocTitle(Doc As Document) As String
    Dim oldSaved As Boolean
    oldSaved = Doc.saved
    GetDocTitle = Doc.BuiltInDocumentProperties(wdPropertyTitle)
    Doc.saved = oldSaved
End Function

'Deletes all 'eqn-platform' properties from active doc
Public Sub DeleteEqnPlatformProperties(Doc As Document)
    DeleteDocProperty Doc, mtprop_HAS_WIN_EQNS
    DeleteDocProperty Doc, mtprop_HAS_MAC_EQNS
End Sub

'Returns True if the active document contains MathType settings
Public Function DocContainsEquationSettings(Doc As Document) As Boolean
    DocContainsEquationSettings = DocPropertyExists(Doc, mtprop_PREFERENCES)
End Function

'returns True if doc is configured to use its own settings for new equations,
'instead of MathType's current settings
'if settings don't exist in doc, returns False.
'if settings exist but mtprop_USE_MATHTYPE_PREFS also exists, returns False.
'if settings exist and mtprop_USE_MATHTYPE_PREFS does not exists, returns True.
Public Function DocUsesEquationSettings(Doc As Document) As Boolean
    Dim prop As DocumentProperty
    Dim uses As Boolean

    uses = DocContainsEquationSettings(Doc)
    If uses Then
        uses = (DocPropertyExists(Doc, mtprop_USE_MATHTYPE_PREFS) = False)
    End If

    DocUsesEquationSettings = uses
End Function

Public Sub ShowPreviewDialog(preview As String, parent As Long)
    Dim helpFile As String
    Dim buffer As String, count As Long, result As Long
    Dim helpID As Integer

    'get a pretty-printed version of the prefs
    count = MTConvertPrefsToUIForm(preview, buffer, 0)
    buffer = Strings.Space(count + 1)
    result = MTConvertPrefsToUIForm(preview, buffer, count)

    'Get the name of the current help file from the registry
    helpFile = GetMTHelpFile()

    #If Win32 Then
        helpID = 117
    #Else
        helpID = 3250
    #End If

    'preview settings using the DLL's function
    result = MTPreviewDialog( _
        parent, _
        GetUserString$("!2700Preview Preferences"), _
        buffer, _
        GetUserString$("!0007Close"), _
        GetUserString$("!0004Help"), _
        helpID, _
        helpFile)
End Sub

'Verifies user-entered section number in section number dialogs.
'If input string is empty or doesn't start with a digit or alphabetic
'character an error string ("-") is returned.
'If the first character is a digit, the string itself is returned.
'If the first character is alphabetic, it is converted to a numeric
'value, e.g. A=1, B=2 etc.
Public Function ConvertEntryToNumStr$(userEntry$)
    Dim counter As Long, place As Long, i As Long, digit As Long, test$

    If Len(userEntry$) = 0 Then
        ConvertEntryToNumStr$ = "-"
        Exit Function
    End If

    test$ = Strings.LTrim(userEntry$)
    Select Case Asc(test$)
        Case Asc("0") To Asc("9")
            ConvertEntryToNumStr$ = Strings.LTrim(Conversion.str$(Val(test$)))
        Case Asc("A") To Asc("Z")
            counter = 0
            place = 0
            For i = Len(test$) To 1 Step -1
                digit = Asc(Strings.Mid$(test$, i, 1)) - Asc("A") + 1
                counter = counter + (digit * power(26, place))
                place = place + 1
            Next i
            ConvertEntryToNumStr$ = Conversion.str$(counter)
        Case Asc("a") To Asc("z")
            counter = 0
            place = 0
            For i = Len(test$) To 1 Step -1
                digit = Asc(Strings.Mid$(test$, i, 1)) - Asc("a") + 1
                counter = counter + (digit * power(26, place))
                place = place + 1
            Next i
            ConvertEntryToNumStr$ = Conversion.str$(counter)
        Case Else
            ConvertEntryToNumStr$ = "-"
    End Select
End Function

'Returns base raised to the power of exponent
Private Function power(base As Long, exponent As Long) As Long
    Dim total As Long, j As Long

    If exponent > 0 Then
        total = 1
        For j = 1 To exponent
            total = total * base
        Next j
        power = total
    Else
        power = 1
    End If
End Function

'Returns kPlatformWin or kPlatformMac
Public Function GetPlatform() As Long
    If InStr(1, Strings.LCase$(System.OperatingSystem), "macintosh", vbBinaryCompare) > 0 Then    ' "Macintosh"
        GetPlatform = kPlatformMac
    Else
        GetPlatform = kPlatformWin
    End If
End Function

'Returns kPlatformWin or kPlatformMac
Public Function GetPlatformString() As String
    If GetPlatform() = kPlatformMac Then
        GetPlatformString = "Macintosh"
    Else
        GetPlatformString = "Windows"
    End If
End Function

Public Sub DelayClipboardCopy()

    ' see http://wiles.dessci/jira/browse/MT-3732

    If (ConvertEquationsDelay <= 0) Then
        Dim strDelay As String
        strDelay = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_CONVERTEQNS_DELAY)

        If IsNumeric(strDelay) Then
            ConvertEquationsDelay = CInt(strDelay)
        Else
            ConvertEquationsDelay = 1000
        End If
    End If

    Delay (ConvertEquationsDelay)

End Sub

Public Sub Delay(timeout As Long)
    Dim start As Long
    start = MTGetTickCount()
    Do While (MTGetTickCount() < (start + timeout))
        DoEvents    'Yield
    Loop
End Sub

'Deletes old macros from active template installed by old versions of MathType
Public Sub MTCleanup()
#If Win32 Then
    Dim myComponent As VBComponent
    Dim theTemplate As Template

    Set theTemplate = ActiveDocument.AttachedTemplate

    On Error Resume Next

    'First remove the simple ones - macros that were all ours
    For Each myComponent In theTemplate.VBProject.VBComponents
        'search for a name used in any known version of MathType. We want to delete
        'macros from the current version also, in case there has been a change.
        Select Case UCase(myComponent.name)
        Case "CLOSEEQN", "NEWDISPEQN", "NEWTEXTEQN", "PASTEDISPEQN", "PASTETEXTEQN", "EDITDISPEQN", _
            "EDITTEXTEQN", "MTAUTO", "MTEQNNUM", "MTEQNNUMFORMAT", "MTLIB", "MTLOADPREFERENCES", _
            "MTMARKREF", "MTPLACEREF", "MTPREFERENCES", "MTSECNUM", "MTUPDATEEQUATIONS", _
            "MTCONVERTEQUATIONS", "MTFORMATEQUATIONS", "MTSECNUMNEXT", "MTSETEQNPREFS", _
            "MTUPDATEFIELDS", "MTCONVERTEQATIONSDLG", "MTDISPEQNPREFS", "MTEQNNUMFORMATDLG", _
            "MTEQPREFSRC", "MTFORMATEQN", "MTSECTIONNUM", "MTSETEQPREFS", "OLDSHOWALL", _
            "OLDINSERTEQUATION", "OLDEDITPICTURE"
            WriteLog "Removing template: " & myComponent.name
            Rem todo
            Rem theTemplate.VBProject.VBComponents.Remove myComponent
        End Select
    Next myComponent

    For Each myComponent In theTemplate.VBProject.VBComponents
        Select Case UCase(myComponent.name)
        'remove our line from AutoNew and AutoOpen if it exists
        Case "AUTONEW", "AUTOOPEN"
            EditAutoMacros theTemplate, myComponent.name

        'rename InsertEquation, ShowAll and EditPicture
        Case "INSERTEQUATION", "SHOWALL", "EDITPICTURE"
            myComponent.name = "Old" & myComponent.name
        End Select
    Next myComponent

    'save the changes
    theTemplate.Save
#End If
End Sub

'get list of files/dirs in a directory, storing the names in an array and returning the count
Public Function GetDirAsArray(dirPath As String, attributes As Long, filenames() As String) As Long
    Const kAllocSize As Long = 100
    Dim fileName As String
    Dim numFiles As Long
    Dim parentDir As String

    parentDir = NormalizeFolder(dirPath)

    ReDim filenames(kAllocSize) As String

    numFiles = 0
    fileName = dir(parentDir, attributes)
    While fileName <> ""
        Dim addItem As Boolean
        addItem = True
        'if we're parsing directories, only return directories
        If attributes = vbDirectory Then
            #If Win32 Then
            If fileName = "." Or fileName = ".." Then addItem = False
            #End If
            If addItem Then
                addItem = (GetAttr(dirPath & fileName) And vbDirectory) = vbDirectory
            End If
        End If

        If addItem Then
            numFiles = numFiles + 1
            If numFiles > UBound(filenames) Then
                ReDim Preserve filenames(UBound(filenames) + kAllocSize) As String
            End If
            filenames(numFiles) = fileName
        End If
        fileName = dir 'read next
    Wend

    GetDirAsArray = numFiles
End Function

'delete a directory and its contents
Public Function DeleteDirectory(dirPath As String) As Boolean
    Dim numFiles As Long
    Dim files() As String
    Dim stat As Boolean
    Dim i As Long
    Dim theDir As String
    theDir = NormalizeFolder(dirPath)

    stat = False 'failure
    On Error GoTo finish

    ' delete any subdirectories first
    numFiles = GetDirAsArray(theDir, vbDirectory, files)

    If numFiles > 0 Then
        For i = 1 To numFiles
            stat = DeleteDirectory(theDir & files(i))
        Next
    End If

    ' delete files second
    numFiles = GetDirAsArray(theDir, vbNormal, files)

    If numFiles > 0 Then
        For i = 1 To numFiles
            Kill theDir & files(i)
        Next
    End If

    RmDir theDir

    stat = True ' return success

finish:
    DeleteDirectory = stat
End Function

'appends final path separator if not already present
Public Function NormalizeFolder(folderPath As String) As String
    If Strings.right$(folderPath, 1) = Application.PathSeparator Then
        NormalizeFolder = folderPath
    Else
        NormalizeFolder = folderPath & Application.PathSeparator
    End If
End Function

'If existing macro contains the phrase MTLoadPreferences.MAIN, remove that line
Public Sub EditAutoMacros(theTemplate As Template, macroName As String)
#If Win32 Then
    Dim lineNo As Long
    On Error GoTo err
    With theTemplate.VBProject.VBComponents(macroName).CodeModule
        'find and remove the code, starting line 1, col 1, thru line 120
        lineNo = 1
        If (.find("MTLoadPreferences.MAIN", lineNo, 1, .CountOfLines, 120, False, False, False)) Then
            .DeleteLines lineNo
        End If
    End With
err:
#End If
End Sub

'Replaces file extension, appends ext if none found. Ext should NOT contain "."
Public Function ReplaceExtension(fileName As String, ext As String) As String
    Dim pos As Long
    pos = InStrR(fileName, ".")
    If pos = 0 Then
        ReplaceExtension = fileName & "." & ext
    Else
        ReplaceExtension = Strings.left$(fileName, pos) & ext
    End If
End Function

' This function is here for legacy reasons.
' It now forwards calls to MTHelp.MTHelpTopic
Public Sub MTHelpTopic(topic As Long)
    #If Win32 Then
    MTHelp.MTHelpTopic topic
    #Else
    MTHelpLaunch topic
    #End If
End Sub

' This is a factory method for obtaining an instance of the MTCommandsDispatch
' class from other templates, e.g. MathType Commands 6 for Word.dot
' The MTCommandsDispatch class methods are invoked using CallByName, and
' merely forward the call to the appropriate module method in WordCommands.dot
Public Function new_MTCommandsDispatchClass()
    'WriteLog "instantiating new MTCommandsDispatchClass"
    Set new_MTCommandsDispatchClass = New MTCommandsDispatchCls
End Function

Public Function MTCommandsPresent()
    MTCommandsPresent = True
End Function

' Reads the registry to determine if verbose logging is on
Private Function GetVerboseLogging() As Boolean

    Static bReadPrefFile As Boolean
    Static bVerboseLogging As Boolean

    If (bReadPrefFile = False) Then

    On Error Resume Next
    Dim regValue As String
    regValue = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_VERBOSE_LOGGING_KEY)
    Select Case regValue
    Case "1"
            bVerboseLogging = True
    Case "0"
            bVerboseLogging = False
    Case Else
            bVerboseLogging = False
    End Select

        bReadPrefFile = True
    End If

    GetVerboseLogging = bVerboseLogging

End Function

' duplicates Asserts.WriteLog from the MathType Commands 6 for Word template
Public Sub WriteLog(msg As String)

    If GetVerboseLogging() = False Then
        Exit Sub
    End If

    On Error GoTo bye
    Dim folder As String
    Dim fileNum As Integer
    fileNum = 0
    Dim logFileName As String
    logFileName = "MT_VBA_Asserts.log"
    Dim logFilePath As String

    #If Win32 Then
        Dim fso As Object
        Set fso = CreateObject("Scripting.FileSystemObject")
        folder = fso.GetSpecialFolder(2)
        folder = folder & "\"
    #Else
        'Application.PathSeparator is undefined in Office 2011
        #If Word Then
        folder = Application.path & ":"
        #Else
        folder = ""
        #End If
    #End If

    logFilePath = folder & logFileName
    fileNum = FileSystem.FreeFile
    If fileNum <> 0 Then
        Open logFilePath For Append Access Write Shared As #fileNum
        Print #fileNum, Now & ": "; msg
    End If
bye:
    If fileNum <> 0 Then
        Close #fileNum
    End If
End Sub

Public Function hasOMMLSupport() As Boolean
    hasOMMLSupport = False
    If Val(Application.version) = kWord2003 And IsWord2003SP3 Then
        hasOMMLSupport = True
    ElseIf Val(Application.version) >= kWord2007 Then
        hasOMMLSupport = True
    End If
End Function

Public Function IsWord2003SP3() As Boolean

    IsWord2003SP3 = False

    If Val(Application.version) <> kWord2003 Then
        Exit Function
    End If

    Dim version As Integer
    version = GetWordMinorVersion(Word.Application.Build)

    If version >= kWord2003SP3MinorVersion Then
        IsWord2003SP3 = True
    End If

End Function

Public Function IsWord2007SP2() As Boolean

    IsWord2007SP2 = False

    If Val(Application.version) <> kWord2007 Then
        Exit Function
    End If

    Dim version As Integer
    version = GetWordMinorVersion(Word.Application.Build)

    If version >= kWord2007SP2MinorVersion Then
        IsWord2007SP2 = True
    End If

End Function

Public Function IsWord2007SP2Installed() As Boolean

    IsWord2007SP2Installed = False

#If Win32 Then
    'Get path and filename of MS Word
    Dim wordPath As String
    If Not (FileExists(GetOfficePath(kWord2007), kWORDEXENAME, wordPath)) Then Exit Function

    'Get size of buffer for the call to GetFileVersionInfo
    Dim bufferLen As Long
    Dim dummy As Long
    Dim buffer() As Byte
    bufferLen = GetFileVersionInfoSize(wordPath, dummy)
    If bufferLen < 1 Then Exit Function
    ReDim buffer(bufferLen)

    'Get version info
    Dim retval As Integer
    Dim versionPointer As Long
    Dim versionBufferLen As Long
    Dim versionBuffer As VS_FIXEDFILEINFO
    retval = GetFileVersionInfo(wordPath, 0, bufferLen, buffer(0))
    retval = VerQueryValue(buffer(0), "\", versionPointer, versionBufferLen)
    MoveMemory versionBuffer, versionPointer, Len(versionBuffer)

    'Extract version info
    Dim fileVer As String
    fileVer = Strings.format$(versionBuffer.dwFileVersionMSh) & "." & _
                      Strings.format$(versionBuffer.dwFileVersionMSl) & "." & _
                      Strings.format$(versionBuffer.dwFileVersionLSh) & "." & _
                      Strings.format$(versionBuffer.dwFileVersionLSl)

    'Check if version is >= Office SP2
    Dim version As Integer
    version = GetWordMinorVersion(fileVer)
    If version >= kWord2007SP2MinorVersion Then IsWord2007SP2Installed = True
#End If
End Function

'The Word version can be obtained in a number of different ways:
'- from vba word.application.build:
'   - 11.0.8169
'- Method 1: from add/remove programs click-here-for support info:
'   - 11.0.8173.0
'- Method 2: from file/properties/version
'   - 11.0.8169.0
'- Method 3: from about box:
'   - 11.8169.8202 SP3
' for a description of the above methods see: http://support.microsoft.com/kb/821549
'
' This routine will reference Word.Application.Build, which appears to match the version number obtained
' using Method 2 without the final ".0"
'
' According to Method 2, these are the Word version numbers:
' Release version: 11.0.5604.0
'             SP1: 11.0.6359.0
'             SP2: 11.0.6568.0
'             SP3: 11.0.8169.0
'
' The format of the version number is aa.b.cccc.d:
'    aa: the major version of Word
'    bb: unknown
'    cc: unknown - assumed to be minor version number, it is the value that this routine returns
'    dd: unknown

Public Function GetWordMinorVersion(version As String) As Integer

    Dim numPeriods As Integer
    numPeriods = GetCountOfStrInStr(version, ".")

    'get last char
    Dim lastchar As Integer
    If numPeriods = 2 Then
        lastchar = Len(version)
    ElseIf numPeriods = 3 Then
        lastchar = InStrR(version, ".")
        lastchar = lastchar - 1
    Else
        GetWordMinorVersion = 0
        Exit Function
    End If

    'get first char
    Dim firstchar As Integer
    firstchar = InStr(version, ".")
    firstchar = firstchar + 1
    firstchar = InStr(firstchar, version, ".")
    firstchar = firstchar + 1

    'get the string
    Dim minorversion As String
    minorversion = Strings.Mid(version, firstchar, lastchar - firstchar + 1)

    'return it as an integer
    GetWordMinorVersion = Int(minorversion)

End Function

' returns a count of the number of times searchfor appears in search
Public Function GetCountOfStrInStr(search As String, searchfor As String) As Integer
    Dim start As Integer
    Dim count As Integer
    start = 1
    count = 0
    While (start <> 0)
        start = InStr(start, search, searchfor)
        If start > 0 Then
            count = count + 1
            start = start + 1
        End If
    Wend
    GetCountOfStrInStr = count

End Function

' insert markers just before and after the current selection
Private Sub InsertEquationMarkers(marker As String)

    'save string for use in DeleteEquationMarkers
    equationMarker = marker

    'insert periods
    Selection.InsertBefore marker
    Selection.InsertAfter marker

    'restore the selection
    Selection.MoveStart wdCharacter, Len(marker)
    Selection.MoveEnd wdCharacter, -(Len(marker))

    Dim BeginRange As Range
    Dim EndRange As Range

    Set BeginRange = Selection.Range
    BeginRange.start = BeginRange.start - Len(marker)
    BeginRange.end = BeginRange.start

    Set EndRange = Selection.Range
    EndRange.end = EndRange.end + Len(marker)
    EndRange.start = EndRange.end

    'get begin bookmark
    ConvertBookmarkBegin = "DSIEqnMarkerStart" 'MTPlaceRef.CreateBookmarkName

    'bookmark the start for later deletion
    ActiveDocument.Bookmarks.Add name:=ConvertBookmarkBegin, Range:=BeginRange

    'get end bookmark
    ConvertBookmarkEnd = "DSIEqnMarkerEnd" 'MTPlaceRef.CreateBookmarkName

    'bookmark the end for later deletion
    ActiveDocument.Bookmarks.Add name:=ConvertBookmarkEnd, Range:=EndRange

End Sub

'deletes the equation markers created in InsertEquationMarkers
Private Sub DeleteEquationMarkers()

    'begin marker
    If ActiveDocument.Bookmarks.Exists(ConvertBookmarkBegin) Then
        'move to begin bookmark, delete period, delete bookmark
        ActiveDocument.Bookmarks(ConvertBookmarkBegin).Select
        Selection.MoveRight wdCharacter, Len(equationMarker), wdExtend
        Selection.delete
        If ActiveDocument.Bookmarks.Exists(ConvertBookmarkBegin) Then
            ActiveDocument.Bookmarks(ConvertBookmarkBegin).delete
        End If
    End If

    'end marker
    If ActiveDocument.Bookmarks.Exists(ConvertBookmarkEnd) Then
        'move to end bookmark, delete period, delete bookmark
        ActiveDocument.Bookmarks(ConvertBookmarkEnd).Select
        Selection.moveLeft wdCharacter, 1 + Len(equationMarker)
        If Selection.Characters(1) = vbCr And Selection.Characters(1) = vbLf Then
            Selection.delete
        Else
            Selection.MoveRight wdCharacter, 1
        End If
        Selection.MoveRight wdCharacter, Len(equationMarker), wdExtend
        Selection.delete
        If ActiveDocument.Bookmarks.Exists(ConvertBookmarkEnd) Then
            ActiveDocument.Bookmarks(ConvertBookmarkEnd).delete
        End If
    End If

End Sub

Public Sub PasteMathMLAsMTEqn(format As ClipboardFormat)

    Dim bufStr As String
    Dim bufLen As Long
    Dim stat As Long
    bufLen = 0

    #If Win32 Then
    'get length of mathml
    If (format = kMathML) Then
        stat = MTGetMathMLFromClipboard(vbNullString, bufLen)
    Else
        stat = MTGetMathMLFromClipboardText(vbNullString, bufLen)
    End If

    If (stat = mtOK And bufLen > 0) Then

        'get mathml
        bufStr = Strings.Space(bufLen)

        If (format = kMathML) Then
            stat = MTGetMathMLFromClipboard(bufStr, bufLen)
        Else
            stat = MTGetMathMLFromClipboardText(bufStr, bufLen)
        End If
    Else
        Exit Sub
    End If
    #Else
    bufStr = VBAGetClipboardData
    #End If

    'convert to mathtype equation
    If (stat = mtOK) Then
        If options.ReplaceSelection And (Selection.start <> Selection.end) Then
            Selection.delete
        Else
            Selection.Collapse wdCollapseStart
        End If
        InsertMTEqnFromMML Selection.Range, bufStr, False
        Selection.Collapse wdCollapseEnd 'display eqn: remove whole line selection
    End If

End Sub

Public Function VBAGetClipboardData() As String

    Dim cbText As String
    Const cbTextLen As Long = 30000
    Dim stat As Long

    cbText = Strings.Space(cbTextLen)
    Dim size As Long
    size = cbTextLen
    stat = MTGetClipboardText(cbText, size)
    If (stat = mtOK) Then
        cbText = Strings.left$(cbText, size)
        cbText = Strings.Trim(cbText)
    Else
        cbText = vbNullString
    End If
    VBAGetClipboardData = cbText

End Function

Public Sub VBAEmptyClipboard()
#If Win32 Then
    If (OpenClipboard(0)) Then
        EmptyClipboard
        CloseClipboard
    End If
#End If
End Sub

Public Function InsertNamespace(mathML As String) As String
    Const kNAMESPACE As String = "http://www.w3.org/1998/Math/MathML"
    Const kXMLNS As String = "xmlns"

    InsertNamespace = mathML

    'check if the namespace already exists
    Dim pos As Integer
    pos = InStr(1, mathML, kNAMESPACE)
    If (pos > 0) Then
        'proper namespace is already in the MathML, nothing to do
        Exit Function
    End If

    'search for a prefix
    mathML = Strings.Trim(mathML)

    'find the start of the first tag
    Dim startMathElement As Integer
    startMathElement = InStr(1, mathML, "<")

    'find the end of the first tag
    Dim endMathElement As Integer
    endMathElement = InStr(1, mathML, ">")

    If (startMathElement > 0 And endMathElement > 0) Then
        'get the first tag
        Dim firstTag As String
        firstTag = Strings.Mid(mathML, startMathElement, endMathElement - startMathElement + 1)

        Dim nameSpace As String
        Dim colon As Integer
        colon = InStr(1, firstTag, ":")
        If (colon > 0) Then
            'prefix found
            Dim startPrefix As Integer
            startPrefix = InStrRev(firstTag, "<", colon)
            nameSpace = kXMLNS & ":" & Strings.Mid(firstTag, startPrefix + 1, colon - startPrefix - 1) & "=" & """" & kNAMESPACE & """"
        Else
            'prefix not found
            nameSpace = kXMLNS & "=" & """" & kNAMESPACE & """"
        End If
            InsertNamespace = Strings.left(mathML, endMathElement - 1) & " " & nameSpace & Strings.right(mathML, Len(mathML) - endMathElement + 1)
    End If

End Function

'This routine assumes that MathML exists in text format on the clipboard
Public Sub AddNamespaceToMathMLInTextOnClipboard()
    Dim strCB As String

    'get clipboard data
    strCB = VBAGetClipboardData
    If (Len(strCB) = 0) Then Exit Sub

    'insert Microsoft required namespace
    strCB = InsertNamespace(strCB)

    'set clipboard data
    MTSetClipboardText strCB

End Sub

'This routine assumes that MathML exists in text format on the clipboard
'and checks to see that non mathml does not precede it or follow it
Public Function IsOnlyMathMLOnClipboard() As Boolean

    IsOnlyMathMLOnClipboard = False

    'get clipboard data
    Dim strCB As String
    strCB = VBAGetClipboardData
    strCB = replace(strCB, vbCrLf, " ")
    strCB = replace(strCB, vbCr, " ")
    strCB = replace(strCB, vbLf, " ")
    strCB = Strings.Trim(strCB)

    'first character must be '<', last char must be '>'
    If Not (Strings.left(strCB, 1) = "<") Or Not (Strings.right(strCB, 1) = ">") Then
        Exit Function
    End If

    'get first tag
    Dim endFirstTag As Integer
    endFirstTag = InStr(1, strCB, ">")
    Dim firstTag As String
    firstTag = Strings.left(strCB, endFirstTag)

    'check that first tag contains 'math'
    If (InStr(1, Strings.LCase$(firstTag), "math", vbBinaryCompare) = 0) Then
        Exit Function
    End If

    'get last tag
    Dim startLastTag As Integer
    startLastTag = InStrRev(strCB, "<")
    Dim lastTag As String
    lastTag = Strings.right(strCB, Len(strCB) - startLastTag + 1)

    'check that last tag contains 'math'
    If (InStr(1, Strings.LCase$(lastTag), "math", vbBinaryCompare) = 0) Then
        Exit Function
    End If

    IsOnlyMathMLOnClipboard = True

End Function

' All the EditPaste/PastePrefs code should move to separate module, but it is
' to late to do that in MT6.6 (MT-2153)
Public Sub MTEditPaste()

On Error Resume Next

    Dim pastePref As Boolean
    pastePref = GetOptionPasteMathMLAsMTEquation
    If MTLib.gPastePrefDlgCanceled Then Exit Sub
        If pastePref Then
            PasteMathMLAsMTEqn currentClipboardFormat
            IncrementStatisticMathML PasteAsType.kMTEqn, currentClipboardFormat
        Else
            AddNamespaceToMathMLInTextOnClipboard
            Selection.Paste
            IncrementStatisticMathML PasteAsType.kOMML, currentClipboardFormat
        End If
End Sub

'returns previously saved user preference for pasting
'handwritten math as a MathType equation, or as OMML
Public Function GetOptionPasteMathMLAsMTEquation() As Boolean

    GetOptionPasteMathMLAsMTEquation = True

   ' since there is no OMML support before Word 2007, never show the dialog
    If Val(Application.version) < kWord2007 Then
        Exit Function
    End If

    Dim askOnPaste As String
    askOnPaste = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_ASK_ON_PASTE)
    If Len(askOnPaste) = 0 Or askOnPaste = "1" Then
        MTPastePrefDlg.Show
        DoEvents
    End If

    Dim pastePref As String
    pastePref = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_MATHML_PASTEAS)
    If Len(pastePref) > 0 And pastePref = "omml" Then
        GetOptionPasteMathMLAsMTEquation = False
    End If

End Function

Private Sub IncrementStatisticMathML(pasteType As PasteAsType, format As ClipboardFormat)

    Dim isOwner As Boolean
    isOwner = False
    #If Win32 Then
    MTIsMathInputPanelTheClipboardOwner isOwner
    #End If

    'see the table in the 6.6 func spec for a description of the following
    'http://tesla.dessci/development/swdev/products/mathtype/MTW66/FuncSpec.htm

    If isOwner = True And pasteType = kMTEqn And format = kMathML Then
        MTIncrementStatisticBy "PstMathMLFromMIPToWordasMT", 1

    'ElseIf isOwner = True And pasteType = kMTEqn And format = kText Then
        'cannot happen - when MIP is owner pastetype must be mathml

    ElseIf isOwner = True And pasteType = kOMML And format = kMathML Then
        MTIncrementStatisticBy "PstMathMLFromMIPToWordasMML", 1

    'ElseIf isOwner = True And pasteType = kOMML And format = kText Then
        'cannot happen - when MIP is owner pastetype must be mathml

    ElseIf isOwner = False And pasteType = kMTEqn And format = kMathML Then
        MTIncrementStatisticBy "PstMathMLClipToWordasMT", 1

    ElseIf isOwner = False And pasteType = kMTEqn And format = kText Then
        MTIncrementStatisticBy "PstMathMLFromCFTEXTToWordasMT", 1

    ElseIf isOwner = False And pasteType = kOMML And format = kMathML Then
        MTIncrementStatisticBy "PstMathMLClipToWordasMML", 1

    ElseIf isOwner = False And pasteType = kOMML And format = kText Then
        MTIncrementStatisticBy "PstMathMLFromCFTEXTToWordasMML", 1

    End If

End Sub

Public Function RemoveNull(strRemove As String) As String
    Dim nullStart As Long
    nullStart = InStr(1, strRemove, Strings.Chr(0), vbBinaryCompare)
    If nullStart > 0 Then
        strRemove = Strings.left$(strRemove, nullStart - 1)
    End If
    RemoveNull = strRemove
End Function

' case-insensitive sort of strArray
Public Sub QuickSort(strArray() As String, intBottom As Integer, intTop As Integer)

    Dim strPivot As String, strTemp As String
    Dim intBottomTemp As Integer, intTopTemp As Integer

    intBottomTemp = intBottom
    intTopTemp = intTop

    strPivot = strArray((intBottom + intTop) \ 2)

    While (intBottomTemp <= intTopTemp)

        While (Strings.UCase(strArray(intBottomTemp)) < Strings.UCase(strPivot) And intBottomTemp < intTop)
            intBottomTemp = intBottomTemp + 1
        Wend

        While (Strings.UCase(strPivot) < Strings.UCase(strArray(intTopTemp)) And intTopTemp > intBottom)
            intTopTemp = intTopTemp - 1
        Wend

        If intBottomTemp < intTopTemp Then
            strTemp = strArray(intBottomTemp)
            strArray(intBottomTemp) = strArray(intTopTemp)
            strArray(intTopTemp) = strTemp
        End If

        If intBottomTemp <= intTopTemp Then
            intBottomTemp = intBottomTemp + 1
            intTopTemp = intTopTemp - 1
        End If

    Wend

    If (intBottom < intTopTemp) Then QuickSort strArray, intBottom, intTopTemp
    If (intBottomTemp < intTop) Then QuickSort strArray, intBottomTemp, intTop

End Sub
