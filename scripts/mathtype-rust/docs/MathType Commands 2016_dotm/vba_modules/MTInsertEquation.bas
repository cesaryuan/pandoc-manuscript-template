Attribute VB_Name = "MTInsertEquation"
'MTInsertEquation
'  Handles inserting MathType inline, display, and numbered display
'  equations
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTInsertEquation.bas 35    5/06/14 9:54a Jimm $
'=====================================================================

Option Explicit

'Inserts an inline MathType equation
Public Sub InsertInlineEquation()
    Dim title As String
    title = MTLib.GetUserString("!1102Insert MathType Equation")
    If Not MTLib.IsDocumentOpen(title) Then GoTo abort
    'check if this command is allowed in the current view
    If Not MTLib.IsCurrentViewOK(title) Then GoTo abort
    'make sure we're not in an EB eqn
    If Val(Application.version) >= kWord2007 Then
        If MTLib.IsInEBEquation(title) Then GoTo abort
    End If
    ' the intended result of inserting an equation at the end of a row
    ' (i.e. outside of the table but at the end of a row) is
    ' not clear, so don't allow the user to do it
    If Selection.Information(wdAtEndOfRowMarker) Then GoTo abort

    MTLib.WriteLog "Passed Sanity Checks"

    MTIncrementStatisticBy "CInline", 1
    InsertMathTypeEquation ActiveDocument, inline:=True

abort:
End Sub

'Inserts un-numbered MathType display equation
Public Sub InsertDisplayEquation()
    MTIncrementStatisticBy "CDisp", 1
    InsertDisplayEquationEx False, False
End Sub

'Inserts left-numbered display equation
Public Sub InsertLeftNumberedDisplayEquation()
    MTIncrementStatisticBy "CDispL", 1
    InsertDisplayEquationEx True, False
End Sub

'Inserts right-numbered display equation
Public Sub InsertRightNumberedDisplayEquation()
    MTIncrementStatisticBy "CDispR", 1
    InsertDisplayEquationEx True, True
End Sub

'Inserts a centered display equation in a new paragraph.
'if eqnNum is True, also inserts a right-justified equation number.
'Document's current style and any extra font changes are saved & restored.
'If insertion point is in a table cell, center and right tabs are set based on cell width
Public Sub InsertDisplayEquationEx( _
    EqnNum As Boolean, _
    position As Boolean)    'True for right, False for left

    Dim stat As Long
    Dim Doc As Document
    Dim title As String
    Dim cellWidth As Long
    Dim listLevel As WdOutlineLevel
    Dim aView As WdViewType

    title = MTLib.GetUserString("!1102Insert MathType Equation")
    'check to see if a document is open
    If MTLib.IsDocumentOpen(title) = False Then GoTo abort
    'check to see if our cursor is in an OK location
    If MTLib.IsCursorPlacedOK(title) = False Then GoTo abort
    'check if this command is allowed in the current view
    If MTLib.IsCurrentViewOK(title) = False Then GoTo abort
    'make sure we're not in an EB eqn
    If Val(Application.version) >= kWord2007 Then
        If MTLib.IsInEBEquation(title) Then GoTo abort
    End If
    ' the intended result of inserting an equation at the end of a row
    ' (i.e. outside of the table but at the end of a row) is
    ' not clear, so don't allow the user to do it
    If Selection.Information(wdAtEndOfRowMarker) Then GoTo abort

    Set Doc = ActiveDocument

    aView = ActiveWindow.View.Type

    MTLib.SetScreenUpdate False

    ActiveWindow.View.Type = wdPrintView

    With Selection

        'if there is a selection, collapse to start
        If .Type <> wdSelectionIP Then
            .Collapse wdCollapseStart
        End If

        listLevel = Selection.ParagraphFormat.OutlineLevel

        PrepareInsertionPostion

        'if we're inserting an eqn number, see if section# reqd.
        If EqnNum Then
            'abort if user cancels
            If Not MTEqnNum.CheckSectionNumber(Doc) Then GoTo abort
        End If

        MTLib.SetScreenUpdate True

        ApplyDisplayStyle .Range

        'if left numbered...
        If EqnNum And (position = False) Then
            '...insert the equation number
            MTEqnNum.DlgMain
        End If

        'insert tab to move to center tabstop
        .TypeText vbTab

        'insert the equation
        InsertMathTypeEquation Doc, False

        ActiveWindow.View.Type = aView

        'if right numbered...
        If EqnNum And (position = True) Then
            '...insert tab and equation number
            .TypeText vbTab
            MTEqnNum.DlgMain
        End If

        ' PrepareInsertionPosition guarantees there is a trailing
        ' paragraph following the display equation on which to
        ' position the cursor, except when in a table.
        If Not .Information(wdWithInTable) Then
            .MoveRight wdCharacter, 1, wdMove
        End If

    End With 'selection

abort:
End Sub

' prepares the insertion point by a) adding paragraphs as necessary
' in order to delegate the propagation of the current style to content
' following the equations to Word's built-in algorithms, and b) guaranteeing
' a trailing paragraph following the display equation on which to
' position the cursor, except when in a table.
Sub PrepareInsertionPostion()
    Dim distMoved As Long

    Dim insertCount As Integer
    Dim moveLeft As Boolean

    Dim atBeginning As Boolean
    Dim atEnd As Boolean
    Dim atDocumentEnd As Boolean

    atBeginning = False
    atEnd = False
    atDocumentEnd = False

    With Selection
        'if we're not at the start of a new paragraph, enter one
        ' or more paragraphs and reposition the cursor
        .Collapse wdCollapseEnd
        distMoved = .moveLeft(wdCharacter, 1, wdMove)
        ' check to see if we moved at all
        If distMoved = 0 Then 'if at start of document
            If Not .Information(wdWithInTable) Then  'not in a table
                .TypeParagraph
                .moveLeft wdCharacter, 1, wdMove
            End If
        Else ' if not at start of document
            ' move back to where we were
            .MoveRight wdCharacter, 1, wdMove

            ' check to see if we are between a tab and a display equation
            If IsCursorRightOfDisplayEquation() Then
                RepositionCursorToRightOfTabOrNumber
            ElseIf IsCursorLeftOfDisplayEquation() Then
                RepositionCursorToLeftOfTabOrNumber
            End If

            ' are we at the end of the document ?
            distMoved = .MoveRight(wdCharacter, 1, wdMove)
            If distMoved = 0 Then
                atDocumentEnd = True
            Else
                ' move back to where we were
                .moveLeft wdCharacter, 1, wdMove
            End If

            ' are we at the end of a paragraph ?
            If Strings.left$(.Text, 1) = Strings.Chr(13) Then
                atEnd = True
            End If

            ' are we at the beginning of a paragraph or top of a column?
            .moveLeft wdCharacter, 1, wdMove
            If Strings.left$(.Text, 1) = Strings.Chr(13) Or Strings.left$(.Text, 1) = Strings.Chr(14) Then
                atBeginning = True
            End If
            .MoveRight wdCharacter, 1, wdMove

            If atDocumentEnd Then
                ' if we are the end of the doc, we need to add a trailing
                ' paragraph to position the cursor on after inserting the
                ' equation.  WARNING: this case is not mutually exclusive
                ' with those that follow so this test must come first in
                ' the if/elseif/elseif... construct.
                If atBeginning Then
                    insertCount = 1
                Else
                    insertCount = 2
                End If
                moveLeft = True
            ElseIf atBeginning And atEnd Then
                ' if we are on a blank line, then just insert
                moveLeft = False
                insertCount = 0
            ElseIf atBeginning Then
                ' we are at the beginning of a para
                insertCount = 1
                moveLeft = True
            ElseIf atEnd Then
                ' we are at the end of a para
                insertCount = 1
                moveLeft = False
            Else
                ' we are somewhere in the middle of a para
                insertCount = 2
                moveLeft = True
            End If

            If insertCount = 1 Then
                .TypeParagraph
            ElseIf insertCount = 2 Then
                .TypeParagraph
                .TypeParagraph
            End If

            ' move back one character (i.e. paragraph) if necessary
            If moveLeft Then
                .moveLeft wdCharacter, 1, wdMove
            End If

        End If
    End With
End Sub

Sub ApplyDisplayStyle(eqnRng As Range)

    Dim cellWidth As Long
    Dim listLevel As WdOutlineLevel
    Dim aView As WdViewType

    listLevel = eqnRng.ParagraphFormat.OutlineLevel

    With eqnRng
        'if we're in a table...
        If .Information(wdWithInTable) Then
            '... set tabs based on cell width
            If Val(Application.version) = kWord97 Then
                cellWidth = .Cells(1).width - .Rows(1).SpaceBetweenColumns
            Else
                cellWidth = GetCellWidth(.Cells(1), .Tables(1))
            End If
            ' outline or master view under office 2000 will cause error when clearing
            ' or setting the tabstops
            aView = ActiveWindow.View.Type
            ActiveWindow.View.Type = wdPrintView
            .ParagraphFormat.TabStops.ClearAll
            If cellWidth > 36 Then 'only set tabstops if cell > 1/2 inch wide
                .ParagraphFormat.TabStops.Add (cellWidth / 2), wdAlignTabCenter, wdTabLeaderSpaces
                .ParagraphFormat.TabStops.Add cellWidth, wdAlignTabRight, wdTabLeaderSpaces
            End If
            ActiveWindow.View.Type = aView
        Else
            '... set tabs using display equation style

            'add our style if it doesn't already exist
            CreateDisplayEquationStyle ActiveDocument

            'select our style and move to the centered Tab
            .style = ActiveDocument.Styles(mtstyle_DISPLAY_EQUATION)

            If listLevel <> wdOutlineLevelBodyText Then
                .ParagraphFormat.OutlineLevel = listLevel
            End If
        End If
    End With

End Sub

' Detects if cursor is to the right of a display equation.
' The selection must be collapsed prior to calling this.
' The cursor position is modified, but returned to its original position
Private Function IsCursorRightOfDisplayEquation() As Boolean
    Dim result As Boolean
    result = True
    With Selection
        If .Information(wdWithInTable) Then
            ' special handling for tables
            Dim cellRng As Range
            ' grab the range of the current cell
            Set cellRng = GetCellRange()

            ' since we want to extend the selection to the left, we
            ' need to check if we are about to move outside of the cell
            If .Range.start = cellRng.start Then
                ' moving to the left will move us outside the cell, so we
                ' clearly are not to the right of a display equation
                result = False
            Else
                ' extend the selection to the left one character
                .moveLeft wdCharacter, 1, wdExtend
                ' see if selection contains an equation
                result = result And IsEquationInSelection()
                ' move back to original position
                .MoveRight wdCharacter, 1, wdMove
                ' now look at the character to the right of the selection
                result = result And (Strings.left(.Text, 1) = Strings.Chr(13) Or Strings.left(.Text, 1) = Strings.Chr(9))
            End If
        Else ' handling for outside of tables
            ' extend the selection to the left one character
            .moveLeft wdCharacter, 1, wdExtend
            If .Information(wdWithInTable) Then
                ' we just selected an entire table, so we
                ' clearly are not to the right of a display equation
                result = False
                ' move back to original position
                .MoveRight wdCharacter, 1, wdMove
            Else
                ' see if selection contains an equation
                result = result And IsEquationInSelection()
                ' move back to original position
                .MoveRight wdCharacter, 1, wdMove
                ' now look at the character to the left of the selection
                result = result And (Strings.left(.Text, 1) = Strings.Chr(13) Or Strings.left(.Text, 1) = Strings.Chr(9))
            End If
        End If
    End With
    IsCursorRightOfDisplayEquation = result
End Function

' Detects if cursor is to the left of a display equation
' The selection must be collapsed prior to calling this.
' The cursor position is modified, but returned to its original position
Private Function IsCursorLeftOfDisplayEquation() As Boolean
    Dim result As Boolean
    result = True
    With Selection
        If .Information(wdWithInTable) Then
            ' special handling for tables
            Dim cellRng As Range
            ' grab the range of the current cell
            Set cellRng = GetCellRange()

            ' since we want to extend the selection to the right, we
            ' need to check if we are about to move outside of the cell
            If .Range.end = cellRng.end - 1 Then
                ' moving to the right will move us outside the cell, so we
                ' clearly are not to the left of a display equation
                result = False
            Else
                ' extend the selection to the right one character
                .MoveRight wdCharacter, 1, wdExtend
                ' see if selection contains an equation
                result = result And IsEquationInSelection()
                ' move back to original position
                .moveLeft wdCharacter, 1, wdMove
                ' since we want to extend the selection to the left, we
                ' need to check if we are about to move outside of the cell
                If .Range.start = cellRng.start Then
                    ' moving to the left will move us outside the cell, so we
                    ' clearly are not to the right of a display equation
                    result = False
                Else
                    ' now extend the selection to the left by one character
                    .moveLeft wdCharacter, 1, wdExtend
                    ' if selection contains a carriage return or tab
                    result = result And (.Text = Strings.Chr(13) Or .Text = Strings.Chr(9))
                    ' move back to original position
                    .MoveRight wdCharacter, 1, wdMove
                End If
            End If
        Else
            ' extend the selection to the right one character
            .MoveRight wdCharacter, 1, wdExtend
            ' see if selection contains an equation
            result = result And IsEquationInSelection()
            ' move back to original position
            .moveLeft wdCharacter, 1, wdMove
            ' now extend the selection to the left by one character
            .moveLeft wdCharacter, 1, wdExtend
            ' if selection contains a carriage return or tab
            result = result And (.Text = Strings.Chr(13) Or .Text = Strings.Chr(9))
            ' move back to original position
            .MoveRight wdCharacter, 1, wdMove
        End If
    End With
    IsCursorLeftOfDisplayEquation = result
End Function

Private Sub RepositionCursorToRightOfTabOrNumber()
    With Selection
        ' move cursor to right by one character
        If Not AttemptToExtendRight Then
            ' can't extend to right, must be in a table cell
            .Collapse wdCollapseStart
            Exit Sub
        End If

        If .Text = Strings.Chr(13) Then
            ' we found a return character, so collapse the selection and get out
            .Collapse wdCollapseStart
            Exit Sub
        ElseIf .Text = Strings.Chr(9) Then
            .Collapse wdCollapseEnd
        End If

        If Not AttemptToExtendRight Then
            ' can't extend to right, must be in a table cell
            .Collapse wdCollapseStart
            Exit Sub
        End If

        ' we expect an equation number
        If .Fields.count > 0 Then
            If InStr(1, Strings.LCase$(.Fields(1).Code.Text), "mtplaceref", vbBinaryCompare) Or .Text = Strings.Chr(13) Then    ' "MTPlaceRef"
                ' collapse
                .Collapse wdCollapseEnd
            Else
                ' expected a equation number... fall back
                .Collapse wdCollapseStart
            End If
        Else
            .Collapse wdCollapseStart
        End If
    End With
End Sub

' should only be called after a successfull call to IsCursorLeftOfDisplayEquation
Private Sub RepositionCursorToLeftOfTabOrNumber()
    With Selection
        If Not AttemptToExtendLeft Then
            .Collapse wdCollapseEnd
        End If

        ' we expect a tab charcter
        If .Text = Strings.Chr(9) Then
            .Collapse wdCollapseStart
        Else
            ' no tab character found... fall back and bail
            .Collapse wdCollapseEnd
            Exit Sub
        End If

        If Not AttemptToExtendLeft Then
            ' failed to move left, bail out
            .Collapse wdCollapseEnd
            Exit Sub
        End If

        ' we expect an equation number
        If .Fields.count > 0 Then
            If InStr(1, Strings.LCase$(.Fields(1).Code.Text), "mtplaceref", vbBinaryCompare) Or .Text = Strings.Chr(13) Then    ' "MTPlaceRef"
                ' collapse
                .Collapse wdCollapseStart
            Else
                ' expected a equation number... fall back
                .Collapse wdCollapseEnd
            End If
        Else
            .Collapse wdCollapseEnd
        End If
    End With
End Sub

' This attempts to EXTEND the selection to the left
' returns true if success or false otherwise
Private Function AttemptToExtendLeft()
    Dim result As Boolean
    result = True
    With Selection
        If .Information(wdWithInTable) Then
            ' special handling for tables
            Dim cellRng As Range
            ' grab the range of the current cell
            Set cellRng = GetCellRange()

            ' since we want to extend the selection to the left, we
            ' need to check if we are about to move outside of the cell
            If .Range.start = cellRng.start Then
                result = False
                Exit Function
            Else
                ' move cursor to left by one character
                .moveLeft WdUnits.wdCharacter, 1, wdExtend
            End If
        Else
            ' move cursor to left by one character
            .moveLeft WdUnits.wdCharacter, 1, wdExtend
        End If
    End With
    AttemptToExtendLeft = result
End Function

' This attempts to EXTEND the selection to the left
' returns true if success or false otherwise
Private Function AttemptToExtendRight()
    Dim result As Boolean
    result = True
    With Selection
        If .Information(wdWithInTable) Then
            ' special handling for tables
            Dim cellRng As Range
            ' grab the range of the current cell
            Set cellRng = GetCellRange()

            ' since we want to extend the selection to the right, we
            ' need to check if we are about to move outside of the cell
            If .Range.end = cellRng.end - 1 Then
                result = False
                Exit Function
            Else
                ' move cursor to right by one character
                .MoveRight WdUnits.wdCharacter, 1, wdExtend
            End If
        Else
            ' move cursor to left by one character
            .MoveRight WdUnits.wdCharacter, 1, wdExtend
        End If
    End With
    AttemptToExtendRight = result
End Function

'This function assumes that it the caller has checked to make sure
'that the selection is really within a table
Private Function GetCellRange() As Range
    Dim row As Long
    Dim col As Long

    With Selection
        ' return the range of the current cell
        row = .Information(wdStartOfRangeRowNumber)
        col = .Information(wdStartOfRangeColumnNumber)
        Set GetCellRange = .Tables(1).Rows(row).Cells(col).Range
    End With
End Function

Private Function IsEquationInSelection() As Boolean
    With Selection
        Dim fld As Field
        For Each fld In .Fields
            If IsFieldEquation(fld) Then
                IsEquationInSelection = True
                Exit Function
            End If
        Next

        ' the shape collection is not always available, and will give
        ' a "not available" error.
        On Error Resume Next
        Dim tmpRng As ShapeRange
        Set tmpRng = .ShapeRange
        If err.Number = 0 Then
            Dim shp As Shape
            For Each shp In tmpRng
                If MTLib.IsShapeEquation(shp) Then
                    IsEquationInSelection = True
                    Exit Function
                End If
            Next
        End If

        Dim inlineshp As InlineShape
        For Each inlineshp In .InlineShapes
            If IsInlineShapeEquation(inlineshp) Then
                IsEquationInSelection = True
                Exit Function
            End If
        Next

    End With
    IsEquationInSelection = False
End Function

'Inserts an equation at the current insertion point.
'If the document has own eqn settings they're sent to MathType first.
'If inline is True, the eqn is internally marked as inline
'Also marks document with platform.
Private Sub InsertMathTypeEquation( _
    Doc As Document, _
    inline As Boolean)

    Dim eqnSettings As String
    Dim propName As String

    On Error Resume Next

    'if there is a selection, collapse to start
    If Selection.Type <> wdSelectionIP Then
        Selection.Collapse wdCollapseStart
    End If

    MTLib.WriteLog "collapsed selection"

    'add property marking doc as having platform's eqns
    MTLib.AddEqnPlatformProperty Doc

    MTLib.WriteLog "added platform property"

    eqnSettings = ""
    'if document uses its own settings for new equations, send them to MathType
    If MTLib.DocUsesEquationSettings(Doc) Then
        eqnSettings = MTLib.GetPrefsFromDoc$
    End If
    MTLib.SetPrefsForNextEqn eqnSettings, inline

    MTLib.WriteLog "set prefs for next equation"
    
    'save current window position - fix for http://valor:8080/browse/MT-2217
#If Win32 Then
    Dim curWindow As Long
    curWindow = GetForegroundWindow()

    Dim retval As Boolean
    Dim saveWinRect As RECT
    retval = GetWindowRect(curWindow, saveWinRect)
#End If
    
    'insert the equation
    InsertNewEquation
    
    'http://valor:8080/browse/MT-1014
    'Word baseline alignment shifts after insterting an equation object into Word 2007
    If (inline = True) And (GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_WORD_NO_SPACE_AFTER_INLINE) <> 1) Then
        'Only for Word 2007 because on 2016 for right-to-left cause errors avoiding edit late
        If (Val(Application.version) = 12) Then
            Selection.InsertAfter " "
            Selection.MoveRight 1
            Selection.Collapse wdCollapseEnd
        End If
    End If
        
#If Win32 Then
    'restore window position
    retval = SetWindowPos(curWindow, HWND_TOP, saveWinRect.left, saveWinRect.top, saveWinRect.right - saveWinRect.left, saveWinRect.bottom - saveWinRect.top, SWP_SHOWWINDOW)
#End If
        
End Sub

'inserts a new equation (and opens it in MT for editing)
'returns True if ok
Private Function InsertNewEquation() As Boolean
    InsertNewEquation = False
    On Error Resume Next
    Selection.InlineShapes.AddOLEObject ClassType:=mtole_PROGID, fileName:=""
    If err.Number = 0 Then
        InsertNewEquation = True
    ElseIf err.Number = 4198 Then
#If Mac Then
        'for Mac Office 11, try our alternative approach
        If Val(Application.version) >= kWord2004 Then
            With Dialogs(wdDialogInsertObject)
                .Class = mtole_PROGID
                .floating = False
                .Execute
            End With
           InsertNewEquation = True
        End If
#End If
    End If
End Function

'If MT display equation style is not yet defined, add it to this document.
'(Para. style based on cur style, has center & right tabs based on doc's width,
'takes cols into account.
'NOTE: switch to Normal view if in Outline/Master Doc view & we're creating style
'      .ParagraphFormat.TabStops can't be modified in Outline view
'      .PageFormat properties not available in Reading view, but we disable most of our commands in Reading view
Public Sub CreateDisplayEquationStyle(Doc As Document)
    Dim docWidth As Long
    Dim newStyle As style
    Dim center As Long
    Dim indent As Long

    If Not MTLib.styleExists(Doc, mtstyle_DISPLAY_EQUATION) Then
        Dim oldViewType As Variant 'WdViewType not supported in W97
        Dim changedView As Boolean

        'can't clear/set tabs in Outline view, so switch to Normal
        changedView = False
        With Doc.ActiveWindow
            If (.ActivePane.View.Type = wdOutlineView) Or _
                (.ActivePane.View.Type = wdMasterView) Then
                changedView = True
                If .View.SplitSpecial = wdPaneNone Then
                    oldViewType = .ActivePane.View.Type
                    .ActivePane.View.Type = wdNormalView
                Else
                    oldViewType = .View.Type
                    .View.Type = wdNormalView
                End If
            End If
        End With

        'get the width of the document for calculating tabstops
        docWidth = GetDocWidth(Doc)

        'create MTDisplay equation style based on current style
        'add centered tab for equation and right tab for equation numbers
        'set next paragraph style to Normal in case user hits enter key
        Set newStyle = Doc.Styles.Add(mtstyle_DISPLAY_EQUATION, wdStyleTypeParagraph)
        With newStyle
            On Error Resume Next
            ' this can fail for certain types of styles
            .BaseStyle = Selection.style
            If err.Number <> 0 Then
                ' fall back to using normal style
                .BaseStyle = wdStyleNormal
            End If
            .ParagraphFormat.TabStops.ClearAll 'not available in Outline/Master view
            indent = Selection.ParagraphFormat.leftIndent
            center = indent + ((docWidth - indent) / 2)
            .ParagraphFormat.TabStops.Add position:=center, Alignment:=wdAlignTabCenter, _
                Leader:=wdTabLeaderSpaces
            .ParagraphFormat.TabStops.Add position:=docWidth, Alignment:=wdAlignTabRight, _
                Leader:=wdTabLeaderSpaces
                .NextParagraphStyle = wdStyleNormal
        End With

        'if we changed the view, change it back
        If changedView Then
            Doc.ActiveWindow.View.Type = oldViewType
        End If
    End If
End Sub

'Returns width of document containing insertion point
'Handles multiple columns
Public Function GetDocWidth(Doc As Document)
    Dim pos As Long
    Dim colWidth As Long
    Dim prevColWidth As Long
    Dim numCols As Long
    Dim i As Long

    If Selection.storyType <> wdTextFrameStory Then
        numCols = Selection.PageSetup.TextColumns.count
        If (numCols > 1) Then
            prevColWidth = Selection.Range.PageSetup.LeftMargin
            pos = Selection.Information(wdHorizontalPositionRelativeToPage)
            For i = 1 To numCols
                Dim col As TextColumn
                Set col = Selection.PageSetup.TextColumns(i)
                colWidth = prevColWidth + col.width
                If i < numCols Then
                    colWidth = colWidth + col.SpaceAfter
                End If
                If (pos < colWidth) Then
                    GetDocWidth = col.width
                    Exit For
                Else
                    prevColWidth = colWidth
                End If
            Next i
        Else
            With Selection.PageSetup
                GetDocWidth = (.PageWidth - (.LeftMargin + .RightMargin))
            End With
        End If
    Else
        GetDocWidth = 0
    End If
End Function

'compute cell width less padding (Word 2000 or later)
Function GetCellWidth(curCell As Cell, curTable As Table) As Long
    With curCell
        GetCellWidth = .width - .LeftPadding - .RightPadding - curTable.Spacing
        'adjust width if column is first, last, or both
        On Error GoTo done
        If .Column.IsFirst Then
            GetCellWidth = GetCellWidth - (curTable.Spacing / 2)
        End If
        If .Column.IsLast Then
            GetCellWidth = GetCellWidth - (curTable.Spacing / 2)
        End If
done:
    End With
End Function


