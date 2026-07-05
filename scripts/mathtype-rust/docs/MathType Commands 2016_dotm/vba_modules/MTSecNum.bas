Attribute VB_Name = "MTSecNum"
'MTSecNum: 5.0
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTSecNum.bas 50    5/06/14 9:54a Jimm $
'=====================================================================


Option Explicit

Public Type BreakDlgInfo
    hasChapter As Boolean               'True if a chapter# exists
    isExplicitChapterNumber As Boolean  'True if we have an explict chapter#
    isExplicitSectionNumber As Boolean  'True if we have an explict section#
    chapterNumber As String                'Explicit chapter#
    sectionNumber As String                 'Explicit section#
    dlgCanceled As Boolean              'True if cancelled
    delete As Boolean                   'True if section should be deleted
    showDelete As Boolean               'True if Delete button should be shown
End Type

'module-level variables so they can be modified by dialogs.
Public gBreakDlg As BreakDlgInfo

'Entry point for inserting chapter/section break. Eqn# always reset to 1.
'Can be either section, or chapter & section (section set to explicit)
'Numbers can be 'next' or explicit. Field codes inserted at the current cursor position.
'The field codes inserted look like (next, explicit):
'{MacroButton MTEditEquationSection Equation Section (Next) {Seq MTEqn \h}{Seq MTSec \h}}
'{MacroButton MTEditEquationSection Equation Section n {Seq MTEqn \h}{Seq MTSec \r n \h}}
'where n is the explicit number to which the number is set.
'For chapter fields, a {Seq MTChap \r n \h} or {Seq MTChap \h} field is added after MTSec.
'The explicit value can be alphabetic or numeric.
'The "Equation [Chapter] Section *" text is formatted with the MTEquationSection character style,
'which this macro creates if it doesn't already exist. Normally this font has the hidden
'style and therefore does not appear in the document.
Public Sub DlgMain()
Attribute DlgMain.VB_Description = "Inserts a new equation section at the insertion point."
    Dim Doc As Document
    Dim title As String
    
    Set Doc = ActiveDocument
    
    If InitalizeState(title) = False Then
        GoTo abort
    End If
    
    'bring up the dialog and get results
    gBreakDlg.isExplicitChapterNumber = False
    gBreakDlg.isExplicitSectionNumber = False
    gBreakDlg.hasChapter = False
    gBreakDlg.showDelete = False
    gBreakDlg.dlgCanceled = False

    MTSectionNum.Show

    If Not gBreakDlg.dlgCanceled Then
        'force dialog to disappear
        DoEvents
        MTLib.SetScreenUpdate False
        
        'insert the new section
        InsertNewEquationSection Doc, gBreakDlg

        MTLib.SetScreenUpdate True
        
        DisplayBreifly Doc
    End If
abort:
End Sub

' returns true on success, false otherwise
Private Function InitalizeState(ByRef title As String) As Boolean
    title = MTLib.GetUserString("!2101Insert Chapter/Section Break")
    'check to see if a document is open
    If MTLib.IsDocumentOpen(title) = False Then
        GoTo abort
    End If

    'disallow this command if inside an EB equation
    If MTLib.IsInEBEquation(title) = True Then
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
    
    'if there is a selection, collapse to start
    If Selection.Type <> wdSelectionIP Then
        Selection.Collapse direction:=wdCollapseStart
    End If
    InitalizeState = True
    Exit Function
abort:
    InitalizeState = False
End Function

Private Sub DisplayBreifly(Doc As Document)
        'if sections hidden, show briefly (.5 sec) so user sees something 'happen'
        If MTLib.IsSectionStyleHidden(Doc) Then
            MTLib.ShowSectionStyle Doc
            MTLib.Delay 500
            MTLib.HideSectionStyle Doc
        End If
        MTIncrementStatisticBy "CCSBrk", 1
End Sub

' convienence entry point for inserting a new "next" chapter break without using a dialog
Public Sub InsertNextChapterBreak()
    Dim Doc As Document
    Dim title As String
    
    Set Doc = ActiveDocument
    
    If InitalizeState(title) = False Then
        GoTo abort
    End If
    
    ' initalize the state as if the dialog were set to insert a new chapter
    gBreakDlg.chapterNumber = ""
    gBreakDlg.delete = False
    gBreakDlg.dlgCanceled = False
    gBreakDlg.hasChapter = True
    gBreakDlg.isExplicitChapterNumber = False
    gBreakDlg.isExplicitSectionNumber = True
    gBreakDlg.sectionNumber = "1"
    gBreakDlg.showDelete = False
    
    MTLib.SetScreenUpdate False
    InsertNewEquationSection ActiveDocument, gBreakDlg
    MTLib.SetScreenUpdate True
    
    DisplayBreifly Doc
abort:
End Sub

' convienence entry point for inserting a new "next" section break without using a dialog
Public Sub InsertNextSectionBreak()
    Dim Doc As Document
    Dim title As String
    
    Set Doc = ActiveDocument
    
    If InitalizeState(title) = False Then
        GoTo abort
    End If
    
    ' initalize the state as if the dialog were set to insert a new chapter
    gBreakDlg.chapterNumber = ""
    gBreakDlg.delete = False
    gBreakDlg.dlgCanceled = False
    gBreakDlg.hasChapter = False
    gBreakDlg.isExplicitChapterNumber = False
    gBreakDlg.isExplicitSectionNumber = False
    gBreakDlg.sectionNumber = ""
    gBreakDlg.showDelete = False
    
    MTLib.SetScreenUpdate False
    InsertNewEquationSection ActiveDocument, gBreakDlg
    MTLib.SetScreenUpdate True
    
    DisplayBreifly Doc
abort:
End Sub

'inserts equation section fields at insertion point/before current selection
'inserts increment-style section unless isExplicitSectionNumber is True
'if so, sectionNumber$ must contain number to use
Public Function InsertNewEquationSection(Doc As Document, break As BreakDlgInfo) As Long
    Dim oldStyle As style
    Dim styleExists As Boolean
    Dim inTable As Boolean
    Dim inRow As Long
    Dim inCol As Long
    Dim tagRange As Range
    Dim breakTag As String
    Dim count As Long
    Dim saveShowAll As Boolean
    
    'save ShowAll state
    saveShowAll = Doc.ActiveWindow.View.ShowAll
    Doc.ActiveWindow.View.ShowAll = False

    styleExists = MTLib.ValidateSectionStyle(Doc)
    
    'insert fields before any existing selection.
    Selection.Collapse wdCollapseStart

    inTable = Selection.Information(wdWithInTable)
    inCol = Selection.Information(wdEndOfRangeColumnNumber)
    inRow = Selection.Information(wdEndOfRangeRowNumber)

    'The following only applys to Word 2007 EB equations
    If Val(Application.version) >= kWord2007 Then

        ' Move off of (i.e. not adjacent to) EB display equations
        ' (either before or after depending on what side we're on)
        If IsLeftOfEBDisplayEqn(Selection.Range) Then
            ' Make sure we're really not in an EB equation (this should never be true,
            ' but ValidateSectionStyle can have the side effect of putting us there)
            If SelInEBEquation() Then
                Selection.moveLeft
            End If
            
            count = Selection.moveLeft
            
            ' If we're not at the start of the doc ...
            If count > 0 Then
                If inTable Then ' but we were in a table
                    ' If we moved out of the cell we were in, treat same as start of doc
                    If Selection.Information(wdWithInTable) Then
                        If inCol <> Selection.Information(wdEndOfRangeColumnNumber) Or inRow <> Selection.Information(wdEndOfRangeRowNumber) Then
                            Selection.MoveRight
                            count = 0
                        End If
                    Else
                        Selection.MoveRight
                        count = 0
                    End If
                End If
                ' If the move right put ust to the right of a preceeding EB equation then ...
                If IsRightOfEBDisplayEqn(Selection.Range) Then
                    Selection.TypeParagraph
                End If
            End If
            If count = 0 Then ' we're at the start of the document ...
                Selection.TypeParagraph ' we need to add a paragraph and
                Selection.moveLeft      ' move before it
            End If
        ElseIf IsRightOfEBDisplayEqn(Selection.Range) Then
            ' Make sure we're really not in an EB equation (this should never be true,
            ' but ValidateSectionStyle can have the side effect of putting us there)
            If SelInEBEquation() Then
                Selection.MoveRight
            End If
            count = Selection.MoveRight
            
            ' If we're not at the end of the doc ...
            If count > 0 Then
                If inTable Then ' but we were in a table
                    ' If we moved out of the cell we were in treat same as end of doc
                    If Selection.Information(wdWithInTable) Then
                        If inCol <> Selection.Information(wdEndOfRangeColumnNumber) Or inRow <> Selection.Information(wdEndOfRangeRowNumber) Then
                            Selection.moveLeft
                            count = 0
                        End If
                    Else
                        Selection.moveLeft
                        count = 0
                    End If
                End If
                ' If the move right put ust to the left of a following EB equation then ...
                If IsLeftOfEBDisplayEqn(Selection.Range) Then
                    Selection.TypeParagraph
                    Selection.moveLeft
                End If
            End If
            If count = 0 Then ' we're at the end of the document ...
                Selection.TypeParagraph ' we need to add a paragraph and
                If Not inTable Then Selection.MoveRight     ' move after it
            End If
        End If
    End If 'Word 2007 EB equations

    'insert an extra character (must NOT be a space!) so we can manage insertion point properly
    Selection.TypeText "."
    
    'The following only applys to Word 2007 EB equations
    If Val(Application.version) >= kWord2007 Then
        ' If the TypeText above put us in an EB equation (which can happen if the cursor
        ' was positioned to the right of a table at the bottom with an EB eqn just below)
        ' then backup and fix it.
        If SelInEBEquation() Then
            Selection.TypeBackspace
            Selection.moveLeft
            Selection.TypeParagraph
            Selection.moveLeft
            Selection.TypeText "."
        End If
    End If
    
    Selection.Collapse wdCollapseStart
    Selection.moveLeft

    'insert Macrobutton field to run equation section editing macro (MTEditEquationSection)
    Selection.Fields.Add Selection.Range, wdFieldMacroButton, _
        (mtmacro_EDIT_EQUATION_SECTION & mtmacro_EDIT_EQUATION_SECTION_VER), False
    Selection.moveLeft wdCharacter, 1, wdExtend
    If ActiveWindow.View.ShowFieldCodes = False Then
        Selection.Fields.ToggleShowCodes
    End If
    
    Selection.Collapse wdCollapseEnd
    Selection.moveLeft wdCharacter, 1

    'insert chapter/section 'tag' and apply our own style to it
    If break.hasChapter Then
        If break.isExplicitChapterNumber Then
            breakTag = MTLib.GetUserString("!2108Equation Chapter") & " " & break.chapterNumber
        Else
            breakTag = MTLib.GetUserString("!2109Equation Chapter (Next)")
        End If
        If break.isExplicitSectionNumber Then
            breakTag = breakTag & " " & MTLib.GetUserString("!2110Section") & " " & break.sectionNumber
        Else
            breakTag = breakTag & " " & MTLib.GetUserString("!2111Section (Next)")
        End If
    Else
        If break.isExplicitSectionNumber Then
            breakTag = MTLib.GetUserString("!2106Equation Section") & " " & break.sectionNumber
        Else
            breakTag = MTLib.GetUserString("!2107Equation Section (Next)")
        End If
    End If

    Selection.InsertAfter breakTag
    Set tagRange = Selection.Range  ''tag' is now selected, so save the range
    Selection.Collapse wdCollapseEnd

    'insert field to reset eqn numbering
    Selection.Fields.Add Selection.Range, wdFieldSequence, "MTEqn " + Strings.ChrW(&H5C) + "r " + Strings.ChrW(&H5C) + "h"
    
    'insert field to increment/set section number
    If break.isExplicitSectionNumber Then
        Selection.Fields.Add Selection.Range, wdFieldSequence, _
            "MTSec " + Strings.ChrW(&H5C) + "r " + break.sectionNumber + " " + Strings.ChrW(&H5C) + "h"
    Else
        Selection.Fields.Add Selection.Range, wdFieldSequence, "MTSec " + Strings.ChrW(&H5C) + "h"
    End If
    
    'insert field to increment/set chapter number
    If break.hasChapter Then
        If break.isExplicitChapterNumber Then
            Selection.Fields.Add Selection.Range, wdFieldSequence, _
                "MTChap " + Strings.ChrW(&H5C) + "r " + break.chapterNumber + " " + Strings.ChrW(&H5C) + "h"
        Else
            Selection.Fields.Add Selection.Range, wdFieldSequence, "MTChap " + Strings.ChrW(&H5C) + "h"
        End If
    End If

    'delete the extra character we added and leave the cursor in a good postion
   With Selection
        If Doc.ActiveWindow.View.ShowFieldCodes = False Then
            .MoveRight wdCharacter, 2, wdMove
            .moveLeft wdCharacter, 1, wdExtend
            .delete
        Else
            .MoveRight wdCharacter, 1, wdMove
            .MoveRight wdCharacter, 1, wdExtend
            .delete
        End If
    End With

    tagRange.style = Doc.Styles(mtstyle_EQUATION_SECTION)
    
    'select field & update it, hides field codes if necessary
    Selection.moveLeft Unit:=wdCharacter, count:=1, Extend:=wdExtend
    Selection.Fields.update
    Selection.MoveRight Unit:=wdCharacter, count:=1, Extend:=wdMove

    'mark doc as having an equation section, avoids extra checks when
    'an equation number is entered
    MTLib.WriteDocPropString Doc, mtprop_EQUATION_SECTION_CHECKED, "1"
    
    'update existing equation numbers, don't force
    MTUpdateFields.UpdateFields mt_RANGE_DOCUMENT, False
    
    'reset the ShowAll state
    Doc.ActiveWindow.View.ShowAll = saveShowAll
    
    Application.StatusBar = MTLib.GetUserString("!2103New equation section set")
End Function


