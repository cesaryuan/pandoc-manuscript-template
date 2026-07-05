Attribute VB_Name = "MTFormatEquationSection"
'MTFormatEquationSection: 5.0
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTFormatEquationSection.bas 36    5/06/14 9:54a Jimm $
'=====================================================================

Option Explicit

Private Const kChapterBreak As Long = 1
Private Const kSectionBreak As Long = 2
Private gLocSeq As String

Public Sub NewVersion()
    Dim answer As Long
    answer = MsgBox(MTLib.GetUserString("!2112This break was created by a newer version of MathType. If you change it, the newer break information will be lost."), vbOKCancel)
    If answer = vbOK Then
        MTFormatEquationSection.EditEquationSection
    End If
End Sub

'DlgMain() is entry point for editing of preceding Equation Break field.
'Displays type of section (chapter/section, increment/explicit), allows user to modify/delete.
'When found, temporarily displays section start text using our own style.
'Hidden by default, controlled by the ShowAll command.
Public Sub DlgMain()
    Dim eqnRange As Range
    Dim Doc As Document
    Dim break As BreakInfo
    Dim title As String
    Dim macroName As String
    Dim startPos As Long
    Dim endPos As Long
    
    title = MTLib.GetUserString("!2104Modify Chapter/Section Break")

    'check to see if a document is open
    If MTLib.IsDocumentOpen(title) = False Then
        GoTo abort
    End If

    'check to see if our cursor is in an OK location
    If MTLib.IsCursorPlacedOK(title) = False Then
        GoTo abort
    End If

    'check if this command is allowed in the current view
    If MTLib.IsCurrentViewOK(title) = False Then
        GoTo abort
    End If
    
    'search from insertion point backwards for equation section fields
    'search from end of selection if one exists
    break.hasChapter = False
    break.isExplicitChapterNumber = False
    break.isExplicitSectionNumber = False
    Set Doc = ActiveDocument
    Set eqnRange = Doc.Range(0, Selection.Range.end)

    If Not SearchRangeForEquationSection(eqnRange, break) Then
        MsgBox MTLib.GetUserString("!2105No equation chapter/section break was found. You can create one using the Insert Chapter/Section Break... command."), _
            vbOKOnly + vbInformation, title
        Exit Sub
    End If

    'get the MTEditEquationSectionx macro name out of the macrobutton field and run it
    break.parentField.Select
    macroName = break.parentField.Code.Text
    startPos = InStr(2, macroName, " ") + 1
    endPos = InStr(startPos, macroName, " ") - 1
    macroName = Strings.Mid$(macroName, startPos, endPos - startPos + 1)
    
    Select Case macroName
        Case "MTEditEquationSection"
            'Application.Run "MTFormatEquationSection.EditEquationSection"
            MTFormatEquationSection.EditEquationSection
        Case "MTEditEquationSection2"
            'Application.Run "MTFormatEquationSection.EditEquationSection"
            MTFormatEquationSection.EditEquationSection
        Case "MTEditEquationSection3"
            'Application.Run "MTFormatEquationSection.NewVersion"
            MTFormatEquationSection.NewVersion
    End Select

abort:
End Sub

'Handles double-click on equation section field, which contains a
'MacroButton field containing a reference to this macro.
'Upon entry the field will be selected, so this macro simply creates a
'range based on the selection and passes it to other macros in the
'MTFormatEquationSection macro for handling.
Public Sub EditEquationSection()
    Dim curRange As Range
    Dim break As BreakInfo
    Dim Doc As Document

    Set curRange = Selection.Range
    If IsEmpty(curRange) Then
        Exit Sub
    End If
    
    break.isExplicitChapterNumber = False
    break.isExplicitSectionNumber = False
    If SearchRangeForEquationSection(curRange, break) Then
        Set Doc = ActiveDocument
        FormatEquationSection Doc, break
    End If
End Sub

'Handles editing of the equation section field passed in.
Private Sub FormatEquationSection(Doc As Document, ByRef break As BreakInfo)
    Dim styleHidden As Boolean
    Dim wordState As Long
    
    'show our section style so that the section names appear
    styleHidden = MTLib.IsSectionStyleHidden(Doc)
    If styleHidden Then
        MTLib.ShowSectionStyle Doc
    End If
    
    'save settings & turn off revision tracking
    wordState = MTLib.SaveWordState
    ActiveDocument.TrackRevisions = False

    'run the dialog
    ShowEquationSectionDlg break

    'restore original options settings
    MTLib.RestoreWordState (wordState)

    'restore section style's state
    If styleHidden Then
       MTLib.HideSectionStyle Doc
    End If
    
    'cleanup
    Set break.parentField = Nothing
End Sub

'searches the given range backwards for an equation chapter/section break field
'if found, examines it to determine it its an increment or an explicit value
'looking for {{MTSec \h}{...}}, which is an increment-style section, or
'{{MTSec \r n \h}{...}}, which is an explicit section (n is the value)
'Chapter break is optional, section break is mandatory
'returns True if found, and passes back break info
Private Function SearchRangeForEquationSection(eqnRange As Range, ByRef break As BreakInfo) As Boolean
    Dim numFieldsInRange As Long
    Dim curField As Long
    Dim sectionFound As Boolean
    Dim parentSearch As String
    Dim aField As Field
    
    sectionFound = False
    
    numFieldsInRange = eqnRange.Fields.count
    If numFieldsInRange > 0 Then
        Application.StatusBar = MTLib.GetUserString("!0705Searching for equation section...")
        
        gLocSeq = MTLib.GetLocaleStr("!0102SEQ")
        parentSearch = Strings.LCase$(gLocSeq & " MTSec " & Strings.ChrW(&H5C))
        
        For curField = numFieldsInRange To 1 Step -1
            Set aField = eqnRange.Fields(curField)
            'if we find a chapter...
            If curField > 3 Then
                If IsChapterBreak(aField, break) Then
                    'look for preceding section break
                    If IsSectionBreak(aField.Previous, break) Then
                        If IsBreakParent(eqnRange.Fields(curField - 3), parentSearch, break) Then
                            sectionFound = True
                            Exit For
                        End If
                    End If
                End If
            End If
            If curField > 2 Then
                If IsSectionBreak(aField, break) Then
                    If IsBreakParent(eqnRange.Fields(curField - 2), parentSearch, break) Then
                        sectionFound = True
                        Exit For
                    End If
                End If
            End If
        Next curField
        
        Application.StatusBar = ""
    End If

    SearchRangeForEquationSection = sectionFound
End Function

'Returns True if field is the parent of the break
Private Function IsBreakParent(aField As Field, search As String, ByRef break As BreakInfo) As Boolean
    If InStr(1, Strings.LCase$(aField.Code.Text), search, vbBinaryCompare) > 0 Then
        Set break.parentField = aField
        IsBreakParent = True
    End If
End Function

'Returns True if field is a ChapterBreak field; if so fills in info appropriately
Private Function IsChapterBreak(aField As Field, ByRef break As BreakInfo) As Boolean
    Dim stat As Boolean

    stat = IsExplicitField(aField, kChapterBreak, break)
    If Not stat Then
        stat = IsNextField(aField, kChapterBreak, break)
    End If
    break.hasChapter = stat
    IsChapterBreak = stat
End Function

'Returns True if field is a SectionBreak field; if so fills in info appropriately
Private Function IsSectionBreak(aField As Field, ByRef break As BreakInfo) As Boolean
    Dim stat As Boolean
    stat = IsExplicitField(aField, kSectionBreak, break)
    If Not stat Then
        stat = IsNextField(aField, kSectionBreak, break)
    End If
    IsSectionBreak = stat
End Function

'Returns True if field is a 'Next' field; if so fills in info appropriately
Private Function IsNextField(aField As Field, kind As Long, ByRef break As BreakInfo) As Boolean
    Dim kindStr As String
    
    IsNextField = False
    If kind = kChapterBreak Then
        kindStr = Strings.LCase$(gLocSeq & " MTChap " + Strings.ChrW(&H5C) + "h")
    Else
        kindStr = Strings.LCase$(gLocSeq & " MTSec " + Strings.ChrW(&H5C) + "h")
    End If
    If InStr(1, Strings.LCase$(aField.Code.Text), kindStr, vbBinaryCompare) > 0 Then
        If kind = kChapterBreak Then
            break.isExplicitChapterNumber = False
        Else
            break.isExplicitSectionNumber = False
        End If
        IsNextField = True
    End If
End Function

'Returns True if field is an 'Explicit' field; if so fills in info appropriately
Private Function IsExplicitField(aField As Field, kind As Long, ByRef break As BreakInfo) As Boolean
    Dim numStart As Long
    Dim numLen As Long
    Dim fieldText As String
    Dim kindStr As String
    
    IsExplicitField = False
    fieldText = Strings.LCase$(aField.Code.Text)
    If kind = kChapterBreak Then
        kindStr = Strings.LCase$(gLocSeq & " MTChap " + Strings.ChrW(&H5C) + "r")
    Else
        kindStr = Strings.LCase$(gLocSeq & " MTSec " + Strings.ChrW(&H5C) + "r")
    End If

    If InStr(1, fieldText, kindStr, vbBinaryCompare) > 0 Then
        numStart = InStr(fieldText, Strings.ChrW(&H5C) + "r") + 3
        numLen = InStr(fieldText, Strings.ChrW(&H5C) + "h") - numStart
        'return False if we can't find the \h
        If numLen > 0 Then
            If kind = kChapterBreak Then
                break.chapterNumber = Strings.Trim(Strings.Mid(fieldText, numStart, numLen))
                break.isExplicitChapterNumber = True
            Else
                break.sectionNumber = Strings.Trim(Strings.Mid(fieldText, numStart, numLen))
                break.isExplicitSectionNumber = True
            End If
            IsExplicitField = True
        End If
    End If
End Function

'bring up the dialog and get results
Private Sub ShowEquationSectionDlg(ByRef break As BreakInfo)
    Dim wordState As Long
    Dim Doc As Document

    'select the entire section field
    break.parentField.Select

    'init globals used to communicate with the dialog
    With MTSecNum.gBreakDlg
        .delete = False
        .showDelete = True
        .dlgCanceled = False
        .hasChapter = break.hasChapter
        .isExplicitChapterNumber = break.isExplicitChapterNumber
        .isExplicitSectionNumber = break.isExplicitSectionNumber
        .chapterNumber = break.chapterNumber
        .sectionNumber = break.sectionNumber
    End With

    MTSectionNum.Show

    If Not MTSecNum.gBreakDlg.dlgCanceled Then
        MTIncrementStatisticBy "CModCSBrk", 1
        DoEvents
        If MTSecNum.gBreakDlg.delete = True Then
            DeleteCurrentEquationSection break.parentField
            'update existing equation numbers, don't force
            MTUpdateFields.UpdateFields mt_RANGE_DOCUMENT, False
        Else
            'save Word's state, then turn TrackChanges off (to avoid field change showing)
            MTLib.SetScreenUpdate False
            wordState = MTLib.SaveWordState
            ActiveDocument.TrackRevisions = False
            
            'delete current section and re-insert
            DeleteCurrentEquationSection break.parentField
            Set Doc = ActiveDocument
            MTSecNum.InsertNewEquationSection Doc, MTSecNum.gBreakDlg

            'restore Word's state, (i.e. TrackRevisions)
            MTLib.RestoreWordState wordState
            MTLib.SetScreenUpdate True
        End If
    End If
End Sub
'deletes field
Private Function DeleteCurrentEquationSection(delField As Field)
    If Not IsEmpty(delField) Then
        delField.delete
    End If
End Function
