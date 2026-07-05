Attribute VB_Name = "MTEqnNumFormat"
'MTFormatEquations: 5.0
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTEqnNumFormat.bas 63    5/06/14 9:54a Jimm $
'=====================================================================

'This macro brings up a dialog that allows the user to select the desired
'format for equation numbers. Only one format is stored per document for
'all new equations. The encoded format is stored in the Custom Document
'Property "MTEquationNumber2", in the form below:

'MTW5 & newer format is a string with tokens #Cx, #Sx and #Ex, where x indicates the format,
'and other characters are used literally. x = 1, R, r, A or a for above formats.
'Tokens can be in any order.
'Examples: "(#S1.#E1)" -> (1.1); "[Chapter#C1:#SA.#Ea]" -> Chapter1:A.a

'MTW4 Format is a 5-char string stored in the MTEquationNumber document property:
'   1: Left enclosure char (_ = none)
'   2: Section number char (0=none,1=Arabic,2=UpperRoman,3=LowerRoman,4=UpperAlpha,5=LowerAlpha)
'   3: Separator char (can be a string)
'   4: Equation number char (see section number char)
'   5: Right enclosure char (_ = none)
'e.g. "{1.1}"

'MT3 format is a 7-char string stored in the ZMTEqnNumFormatPrefs autotext entry:
'   1: blank
'   2: enclosure chars (0=(), 1=[], 2={}, 3=none)
'   3: blank
'   4: section number char (0=Arabic,1=UpperRoman,2=LowerRoman,3=UpperAlpha,4=LowerAlpha,5=none)
'   5: separator char (can be a string)
'   6: blank
'   7: equation number char (see section number char)
'e.g. " 0 0. 0" = (1.1)

Option Explicit

Type EqnFormatInfo
    selectionType As Long       'in: type of existing selection
    format As String            'in/out: the format
    customFormat As Boolean     'in/out: true if custom
    deferUpdate As Boolean      'in/out: true if eqnnum update is deferred
    suppressEqnNumWarning As Boolean 'in/out: true to suppress 'section# = 0' dlg on eqn# insert
    suppressEqnRefWarning As Boolean 'in/out: true to suppress dlg on eqn ref insert
    changeSelected As Boolean   'in/out: true to update existing #s in selection
    changeWholeDoc As Boolean   'in/out: true to update existing #s in doc
    changeFuture As Boolean     'out: true to use format for new numbers
    useAsDefaults As Boolean    'out: true to save settings as defaults
    dlgCanceled As Boolean      'out: true if cancelled
End Type

'Global variable for communication with dialog
Public gEqnNumFormatDlgInfo As EqnFormatInfo

Public Sub DlgMain()
    Dim origSelection As Range
    Dim Doc As Document
    Dim sel As Long
    Dim title As String
    

    'make sure locale DLL is available, exit if not
    If Not MTLib.CheckLocaleDLL() Then
        Exit Sub
    End If

    'check to see if a document is open
    title = MTLib.GetUserString("!0300Format Equation Numbers")
    If MTLib.IsDocumentOpen(title) = False Then
        GoTo abort
    End If

    'check if this command is allowed in the current view
    If MTLib.IsCurrentViewOK(title) = False Then
        GoTo abort
    End If
    
    Set Doc = ActiveDocument

    'save the user's current selection
    Set origSelection = Selection.Range
    
    'setup for dialog
    With gEqnNumFormatDlgInfo
        .format = GetEqnNumFormat(Doc)
        If .format = "" Then Exit Sub
        .customFormat = IsEqnNumFormatCustom(Doc)
        .selectionType = Selection.Type
        .deferUpdate = (MTLib.ReadDocPropString(Doc, mtprop_DEFER_FIELD_UPDATE) = "1")
        .suppressEqnNumWarning = GetDontShowEqnNumWarning()
        .suppressEqnRefWarning = GetDontShowEqnRefWarning()
        .useAsDefaults = False
    End With

    'put up the dialog & allow to disappear when closed
    MTEqnNumFormatDlg.Show
    DoEvents

    With gEqnNumFormatDlgInfo
        If Not .dlgCanceled Then
            'update format if 'new numbers' was checked
            If .changeFuture Then
                SetEqnNumFormat Doc, .format, .customFormat
            End If
                
            If .deferUpdate Then
                MTLib.WriteDocPropString Doc, mtprop_DEFER_FIELD_UPDATE, "1"
            Else
                MTLib.DeleteDocProperty Doc, mtprop_DEFER_FIELD_UPDATE
            End If
    
            SetDontShowEqnNumWarning .suppressEqnNumWarning
            SetDontShowEqnRefWarning .suppressEqnRefWarning
    
            If .useAsDefaults Then
                SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_DEFAULT_EQNNUM_FORMAT, .format
                If .customFormat Then
                    SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_DEFAULT_EQNNUM_CUSTOM, "1"
                Else
                    SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_DEFAULT_EQNNUM_CUSTOM, "0"
                End If
            End If

            'If the user selected to update existing numbers...
            If .changeSelected Or .changeWholeDoc Then
                'set range to be updated
                If .changeWholeDoc Then
                    sel = mt_RANGE_DOCUMENT
                Else
                    sel = mt_RANGE_SELECTION
                End If
                
                UpdateEquationNumbers sel, .format
            End If
            'update statistics
            MTIncrementStatisticBy "CFmtEqNum", 1
            If .customFormat Then
                MTIncrementStatisticBy "FmtEqNumAdv", 1
            End If
        End If
    End With
    
    'restore the original selection
    origSelection.Select
abort:
End Sub

'Updates all equation number fields in the range
'Range is mt_RANGE_SELECTION or mt_RANGE_DOCUMENT
'At finish updates fields to correct all numbers
Private Sub UpdateEquationNumbers(sel As Long, format As String)
    Dim wordState As Long
    Dim fieldCount As Long
    Dim fcState As Boolean
    Dim aRange As Range
    Dim aStory As Range

    'set the mouse pointer
    System.Cursor = wdCursorWait

    MTLib.SetScreenUpdate False
    
    'want Field Codes not shown, restore at end
    fcState = ActiveWindow.View.ShowFieldCodes
    If fcState = True Then
        ActiveWindow.View.ShowFieldCodes = False
    End If
    
    'save Word's state, then turn TrackChanges off
    wordState = MTLib.SaveWordState()
    ActiveDocument.TrackRevisions = False
    
    'initial display in statusbar
    fieldCount = 0
    Application.StatusBar = MTLib.GetUserString("!0310Updating equation number: ")
    
    If sel = mt_RANGE_SELECTION Then
        Set aRange = Selection.Range
        UpdateEquationNumbersInRange aRange, format, fieldCount
    Else
        For Each aStory In ActiveDocument.StoryRanges
            Do
                Set aRange = aStory
                UpdateEquationNumbersInRange aRange, format, fieldCount
                'if this story has a valid NextRange, process it
                Set aStory = aStory.NextStoryRange
            Loop While (IsObjectValid(aStory))
        Next 'story
    End If

    'restore Field Codes
    ActiveWindow.View.ShowFieldCodes = fcState
    Application.StatusBar = ""
    
    'force update all fields if we made any changes
    If fieldCount > 0 Then
        MTUpdateFields.UpdateFields mt_RANGE_DOCUMENT, True
    End If
    
    'restore Word's state, (TrackRevisions)
    MTLib.RestoreWordState wordState
    
    MTLib.SetScreenUpdate True
    System.Cursor = wdCursorNormal
End Sub

Private Sub UpdateEquationNumbersInRange(myRange As Range, format As String, ByRef fieldCount As Long)
    Dim curField As Field
    Dim length As Integer
    Dim msg As String
    Dim existingBookmarks() As String
    Dim index As Long, bookmarkCount As Long
    Dim i As Long, numfields As Long, countBefore As Long
    Dim sMTChap As String
    Dim sMTSec As String
    Dim sMTEqn  As String
    
    msg = MTLib.GetUserString("!0310Updating equation number: ")
    sMTChap = "MTChap " + Strings.ChrW(&H5C) + "c"
    sMTSec = "MTSec " + Strings.ChrW(&H5C) + "c"
    sMTEqn = "MTEqn " + Strings.ChrW(&H5C) + "c"

    'loop thru all equation number fields
    'can't use For Each, it gets confused by the deletions and insertions
    i = 1
    numfields = myRange.Fields.count
    Do While i <= numfields
        Set curField = myRange.Fields(i)
        If ((curField.Type = wdFieldSequence) And _
            (InStr(1, curField.Code.Text, sMTChap, vbBinaryCompare) Or _
             InStr(1, curField.Code.Text, sMTSec, vbBinaryCompare) Or _
             InStr(1, curField.Code.Text, sMTEqn, vbBinaryCompare))) Then

            'update statusbar
            fieldCount = fieldCount + 1
            Application.StatusBar = msg & fieldCount
            
            'mark the current bookmark if it exists (so cross-references aren't lost)
            curField.Select
            Selection.MoveRight wdCharacter, 1, wdExtend
            bookmarkCount = Selection.Bookmarks.count
            If bookmarkCount > 0 Then
                ReDim existingBookmarks(bookmarkCount)
                For index = 1 To bookmarkCount
                    existingBookmarks(index) = Selection.Bookmarks(index).name
                Next index
            End If
            
            'delete the old number
            Selection.delete
            countBefore = myRange.Fields.count
            
            'insert fieldchars, macrobutton (for references), and
            'field to increment eqn#
            Selection.Fields.Add Selection.Range, wdFieldMacroButton, "MTPlaceRef"
            Selection.moveLeft wdCharacter, 1, wdExtend
            Selection.Fields.ToggleShowCodes    'turn codes on
            Selection.Collapse wdCollapseEnd
            Selection.moveLeft wdCharacter, 1
            Selection.Fields.Add Selection.Range, wdFieldSequence, "MTEqn " + Strings.ChrW(&H5C) + "h"
            
            'insert the actual data
            length = InsertEquationNumber(ActiveDocument, format)
            
            'bookmark the whole thing if old bookmark(s) existed
            If bookmarkCount > 0 Then
                Selection.moveLeft wdCharacter, length, wdExtend
                For index = 1 To bookmarkCount
                    Selection.Bookmarks.Add existingBookmarks(index), Selection.Range
                Next index
               
                Selection.Collapse wdCollapseEnd
            End If
                                
            Selection.MoveRight wdCharacter, 1, wdExtend
            Selection.Fields.ToggleShowCodes 'hide the codes
            
            'move past the current number so that we don't simply do it over again
            numfields = myRange.Fields.count
            i = i + (numfields - countBefore)
        Else
            i = i + 1
        End If
    Loop
    
End Sub

'inserts template as defined by selections
'returns length (in characters) of the template
Public Function InsertEquationNumber(Doc As Document, format As String) As Long
    Dim literal As String
    Dim length As Long
    Dim start As Long
    Dim pos As Long

    'set initial length to zero
    length = 0
    
    start = 1
    pos = InStr(start, format, "#", vbBinaryCompare)
    While (pos > 0)
        literal = Strings.Mid$(format, start, pos - start)
        Selection.InsertBefore literal
        Selection.Collapse wdCollapseEnd
        length = length + Len(literal)

        length = length + InsertNumberComponent(Doc, format, pos)
        start = pos
        pos = InStr(start, format, "#", vbBinaryCompare)
    Wend
    literal = Strings.Mid$(format, start)
    Selection.InsertBefore literal
    Selection.Collapse wdCollapseEnd
    length = length + Len(literal)

    'return length of inserted text
    InsertEquationNumber = length
End Function
    
'Inserts field according to next char in format string (1,A,a,I,i).
'Inserts # if none of these, also advances pos accordingly
'Returns length of text inserted
Private Function InsertNumberComponent(Doc As Document, format As String, ByRef pos As Long) As Long
    Dim token1 As String
    Dim token2 As String
    Dim length As Long
    
    pos = pos + 1
    token1 = Strings.Mid$(format, pos, 1)
    If InStr(1, "CSE", token1, vbBinaryCompare) > 0 Then
        pos = pos + 1
        token2 = Strings.Mid$(format, pos, 1)
        If InStr(1, "1IiAa", token2, vbBinaryCompare) > 0 Then
            length = InsertComponentField(Doc, token1, token2)
            Selection.Collapse wdCollapseEnd
            pos = pos + 1
        Else
            Selection.InsertBefore "#"
            Selection.Collapse wdCollapseEnd
            pos = pos - 1
            length = 1
        End If
    Else
        Selection.InsertBefore "#"
        Selection.Collapse wdCollapseEnd
        length = 1
    End If
    
    InsertNumberComponent = length
End Function

'Inserts the actual field, returns length of insertion
Private Function InsertComponentField(Doc As Document, break As String, styleChar As String) As Long
    Dim breakStyle As String
    Dim length As Long
    Dim fieldText As String

    Select Case styleChar
    Case "1"
        breakStyle = "!0105Arabic"
    Case "I"
        breakStyle = "!0108Roman"
    Case "i"
        breakStyle = "!0111roman"
    Case "A"
        breakStyle = "!0114Alphabetic"
    Case "a"
        breakStyle = "!0117alphabetic"
    Case Else
        breakStyle = ""
    End Select
    
    'append switch only if we have a real string (fixes Greek Word bug!)
    If Len(breakStyle) > 1 Then
        breakStyle = Strings.ChrW(&H5C) + "* " & MTLib.GetLocaleStr(breakStyle)
    End If

    'get the right kind of field text based on type of break
    Select Case break
    Case "C"
        fieldText = "MTChap " + Strings.ChrW(&H5C) + "c "
    Case "S"
        fieldText = "MTSec " + Strings.ChrW(&H5C) + "c "
    Case "E"
        fieldText = "MTEqn " + Strings.ChrW(&H5C) + "c "
    End Select
    
    If Len(fieldText) > 0 Then
        Doc.Fields.Add Selection.Range, wdFieldSequence, fieldText + breakStyle
        Selection.Collapse wdCollapseEnd
        length = 1
    End If

    InsertComponentField = length
End Function

'returns the equation number format to use for this document
'looks for doc property, else gets format from registry, else uses built-in defaults
'in doc converts MTW4 or MTW3 format if found and MTW5 format doesn't exist
'if no property found, creates one
Public Function GetEqnNumFormat(Doc As Document) As String
    Dim format As String
    Dim custom As Boolean
    Dim OKCancel As Integer
    Dim latestVer As Long

    'check for new doc property
    latestVer = MTLib.NewerDocPropertyVersion(Doc, mtprop_NUMBER_PREFS, mtprop_NUMBER_PREFS_VER)
    If latestVer <> 0 Then
        OKCancel = MsgBox(MTLib.GetUserString("!2900The equation number formatting was created by a newer version of MathType. If you continue the newer formatting information will be lost. Do you wish to continue?"), vbOKCancel)
        If OKCancel = vbOK Then
            MTLib.DeleteDocProperty Doc, mtprop_NUMBER_PREFS & latestVer
        Else
            GetEqnNumFormat = ""
            Exit Function
        End If
    End If
    
    If MTLib.DocPropertyExists(Doc, mtprop_NUMBER_PREFS & mtprop_NUMBER_PREFS_VER) Then
        'found the MT5 property; do nothing
    ElseIf ProcessMTW4DocProperty(Doc) Then
        'found and processed the MT4 property
    ElseIf ProcessMTW3DocProperty(Doc) Then
        'found and processed the MT3 property
    Else
        'if there is a registry default...
        format = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_DEFAULT_EQNNUM_FORMAT)
        If format <> "" Then
            '... use it
            custom = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_DEFAULT_EQNNUM_CUSTOM)
        Else
            '... use built-in default "(1.1)"
            format = "(#S1.#E1)"
            custom = False
        End If
        SetEqnNumFormat Doc, format, custom
    End If

    're-read in case we converted
    GetEqnNumFormat = MTLib.ReadDocPropString$(Doc, mtprop_NUMBER_PREFS & mtprop_NUMBER_PREFS_VER)
End Function

Function IsEqnNumFormatCustom(Doc As Document) As Boolean
    IsEqnNumFormatCustom = (MTLib.ReadDocPropString$(Doc, mtprop_CUSTOM_EQNNUM_PREFS) = "1")
End Function

'writes the equation number format & custom format properties
Private Sub SetEqnNumFormat(Doc As Document, format As String, custom As Boolean)
   MTLib.WriteDocPropString Doc, mtprop_NUMBER_PREFS & mtprop_NUMBER_PREFS_VER, format
    If custom Then
        MTLib.WriteDocPropString Doc, mtprop_CUSTOM_EQNNUM_PREFS, "1"
    Else
        MTLib.DeleteDocProperty Doc, mtprop_CUSTOM_EQNNUM_PREFS
    End If
End Sub

Private Function ProcessMTW4DocProperty(Doc As Document) As Boolean
    Dim format As String
    ProcessMTW4DocProperty = False
    format = MTLib.ReadDocPropString$(Doc, mtprop_MTW4_NUMBER_PREFS)
    If format <> "" Then
        format = ConvertMTW4Format(format)
        If format <> "" Then
            SetEqnNumFormat Doc, format, False
            MTLib.DeleteDocProperty Doc, mtprop_MTW4_NUMBER_PREFS
            ProcessMTW4DocProperty = True
        End If
    End If
End Function

'Returns MTW5 format converted from MTW4 format
Private Function ConvertMTW4Format(mtw4Format As String) As String
    Dim encl1 As String, encl2 As String
    Dim section As String
    Dim equation As String
    Dim sep As String

    encl1 = Strings.Mid$(mtw4Format, 1, 1)
    If encl1 = "(" Then
        encl2 = ")"
    ElseIf encl1 = "[" Then
        encl2 = "]"
    ElseIf encl1 = "{" Then
        encl2 = "}"
    ElseIf encl1 = "<" Then
        encl2 = ">"
    Else
        encl2 = encl1
    End If
    
    Select Case Strings.Mid(mtw4Format, 2, 1)
    Case "0"
        section = ""    'none
    Case "1"
        section = "#S1" 'Arabic
    Case "2"
        section = "#SI" 'Roman UC
    Case "3"
        section = "#Si" 'Roman LC
    Case "4"
        section = "#SA" 'Alphabetic UC
    Case "5"
        section = "#Sa" 'Alphabetic LC
    Case Else
        section = "#S1" 'Arabic default if bad format
    End Select
    
    sep = Strings.Mid$(mtw4Format, 3, Len(mtw4Format) - 4)

    Select Case Strings.Mid(mtw4Format, 4, 1)
    Case "0"
        equation = ""   'none
    Case "1"
        equation = "#E1" 'Arabic
    Case "2"
        equation = "#EI" 'Roman UC
    Case "3"
        equation = "#Ei" 'Roman LC
    Case "4"
        equation = "#EA" 'Alphabetic UC
    Case "5"
        equation = "#Ea" 'Alphabetic LC
    Case Else
        equation = "#E1" 'Arabic default if bad format
    End Select

    ConvertMTW4Format = encl1 & section & sep & equation & encl2
        
End Function

Private Function ProcessMTW3DocProperty(Doc As Document) As Boolean
'if an AutoText entry for the 'MT 3 and before' equation number format exists,
'convert it to the new format and then delete it
'Note: if format is already stored as a property, it is NOT overwritten
    Dim autoText As AutoTextEntry
    Dim format As String
    
    'try to read old AutoText entry, will error if it doesn't exist
    ProcessMTW3DocProperty = False
    On Error GoTo bye
    Set autoText = Doc.AttachedTemplate.AutoTextEntries(mtautotext_MT3_EQN_NUMBER_FORMAT)
    
    'first convert MTW3 format to MTW4 format
    format = ConvertMTW3Format(autoText.value)
    If format <> "" Then
        'then convert MTW4 format to MTW5 format
        format = ConvertMTW4Format(format)
        'save as non-custom
        SetEqnNumFormat Doc, format, False
        'delete the AutoText Entry from the template
        Doc.AttachedTemplate.AutoTextEntries(mtautotext_MT3_EQN_NUMBER_FORMAT).delete
        Doc.AttachedTemplate.Save
    End If
    ProcessMTW3DocProperty = True

bye:
End Function

'converts mtw3 equation number format to mtw4 equation number format
Private Function ConvertMTW3Format(MTW3Format As String) As String
    Dim Enclosure$, separator$
    Dim sectionStyle$, equationStyle$
    Dim sectionStyleNum, equationStyleNum As Long
    
    'set enclosure chars as defined by MT3 format
    Select Case Val(Strings.Mid(MTW3Format, 2, 1))
    Case 0
        Enclosure$ = "()"
    Case 1
        Enclosure$ = "[]"
    Case 2
        Enclosure$ = "{}"
    Case Else
        Enclosure$ = "__"
    End Select
    
    sectionStyleNum = Val(Strings.Mid(MTW3Format, 4, 1))
    If sectionStyleNum = 5 Then
        sectionStyleNum = 0
    Else
        sectionStyleNum = sectionStyleNum + 1
    End If
    sectionStyle$ = Strings.format(sectionStyleNum)
    
    equationStyleNum = Val(Strings.Mid(MTW3Format, 7, 1))
    If equationStyleNum = 5 Then
        equationStyleNum = 0
    Else
        equationStyleNum = equationStyleNum + 1
    End If
    equationStyle$ = Strings.format(equationStyleNum)
    
    separator$ = Strings.Mid(MTW3Format, 5, 1)
    
    ConvertMTW3Format = Strings.left(Enclosure$, 1) + sectionStyle$ + separator$ + _
        equationStyle$ + Strings.right(Enclosure$, 1)
        
End Function
