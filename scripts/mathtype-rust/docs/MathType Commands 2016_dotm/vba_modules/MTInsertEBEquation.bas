Attribute VB_Name = "MTInsertEBEquation"
'MTInsertEBEquation:
' Functions for inserting normal and numbered Equation Builder (EB)
' equations into the Active Word document
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTInsertEBEquation.bas 39    9/17/12 1:21p Jimm $
'=====================================================================

Option Explicit

' redefinition of wdStyleTypeTable
Private Const MTStyleTypeTable As Long = 3
Const kWDOMathDisplay As Long = 0

'Inserts an Equation Builder (EB) equation at the current insertion point
Public Sub InsertEBEquation()
    Dim title As String
    Dim Doc As Document
    
    Set Doc = ActiveDocument
    title = MTLib.GetUserString2("1104", "1304", "Insert Equation Builder Equation")
    
    'check to see if a document is open
    If MTLib.IsDocumentOpen(title) = False Then GoTo abort
    'check to see if our cursor is in an OK location
    If MTLib.IsCursorPlacedOK(title) = False Then GoTo abort
    'check if we're already in an EB equation
    If SelInEBEquation <> 0 Then GoTo abort
    
    InsertEBEquationEx Doc
abort:
End Sub

Private Sub InsertEBEquationEx(Doc As Document)
    Dim objRange As Object
    Dim objEq As Object
   
    Set objRange = Selection.Range
    Set objRange = SetOMaths(Selection, objRange)
    Set objEq = GetFirstOMath(objRange)
    objEq.BuildUp
End Sub

' used by InsertEBEquationEx to get late binding
Private Function SetOMaths(sel, objRange)
    Set SetOMaths = sel.OMaths.Add(objRange)
End Function
' used by InsertEBEquationEx to get late binding
Private Function GetFirstOMath(objRng)
    Set GetFirstOMath = objRng.OMaths(1)
End Function

'Inserts a left numbered EB equation at the current insertion point
Public Sub InsertEBLeftNumberedDisplayEquation()
    InsertEBNumberedEquationEx Nothing, False
End Sub

'Inserts a right numbered EB equation at the current insertion point
Public Sub InsertEBRightNumberedDisplayEquation()
        InsertEBNumberedEquationEx Nothing, True
End Sub

Public Sub InsertEBNumberedEquationEx( _
    vTheEBEqn As Variant, _
    position As Boolean)    'True for right, False for left

    Dim stat As Long
    Dim distMoved As Long
    Dim isInTable As Boolean
    Dim atStartOfDoc As Boolean
    Dim Doc As Document
    Dim title As String
    Dim cellWidth As Long
    Dim aTable As Table
    Dim inTablePos As Long
    
    Dim theEBEqn
    Set theEBEqn = vTheEBEqn
    
    isInTable = False
    atStartOfDoc = False
    
    title = MTLib.GetUserString2("1104", "1304", "Insert Equation Builder Equation")
    'check to see if a document is open
    If MTLib.IsDocumentOpen(title) = False Then GoTo abort
    'check to see if our cursor is in an OK location
    If MTLib.IsCursorPlacedOK(title) = False Then GoTo abort
    'check to see if our cursor is currently in an EB table
    inTablePos = SelInEBNumberedEqnTable()
    If Not MoveOutOfEBNumberedEqnTable(inTablePos) Then GoTo abort
    
    MTLib.SetScreenUpdate False
   
    Set Doc = ActiveDocument
    With Selection
         'collapse to start (in case selection is not IP)
         .Collapse wdCollapseStart
               
        ' Check if in a table
        isInTable = .Information(wdWithInTable)
        
        ' Check if at start of document
        distMoved = .moveLeft(wdCharacter, 1, wdMove)
        ' did we move at all?
        If distMoved = 0 Then 'no -- at start of document
            atStartOfDoc = True
        Else
            ' move back to where we were
            .MoveRight wdCharacter, 1, wdMove
        End If
        
        'see if section# reqd. and abort if user cancels
        If Not MTEqnNum.CheckSectionNumber(Doc) Then GoTo abort
         
        'if we're not at the start of a new paragraph, enter one
        If Not atStartOfDoc Then
            ' are we in a table?
            If Not isInTable Then
                If Strings.left$(.Text, 1) <> Strings.Chr(13) Then .TypeParagraph
            End If
        End If
        
        ' its possible for the cursor to be at the end of a row, which is not a
        ' acceptible location; just bail out if this happens
        On Error GoTo abort
        Set aTable = InsertEBNumberedEqnTable(Doc, theEBEqn, position)
        On Error GoTo 0
        
        'Put the selection in the EB equation
        aTable.Cell(1, 2).Select
        .Collapse wdCollapseEnd
        .moveLeft
        .moveLeft
    
    End With 'selection

abort:
    MTLib.SetScreenUpdate True
End Sub

'If EB Numbered Equation style is not yet defined, add it to this document.
Private Function GetEBNumberedEqnTableStyle( _
    Doc As Document) As Object
      
    Dim newStyle As Object ' This is a style object
    If Not MTLib.styleExists(Doc, mttstyle_EB_NUMBERED_EQUATION) Then
        Set newStyle = Doc.Styles.Add(mttstyle_EB_NUMBERED_EQUATION, MTStyleTypeTable)
        With newStyle.Table
            .Alignment = wdAlignRowLeft
            .AllowBreakAcrossPage = False
            .AllowPageBreaks = True
            .BottomPadding = 0
            .ColumnStripe = 0
            .leftIndent = 0
            .LeftPadding = 0
            .RightPadding = 0
            .RowStripe = 0
            .Spacing = 0
            .TableDirection = wdTableDirectionLtr
            .TopPadding = 0
            With .Borders
                .DistanceFromBottom = 0
                .DistanceFromLeft = 0
                .DistanceFromRight = 0
                .DistanceFromTop = 0
                .Enable = 0
                .InsideColor = wdColorAutomatic
                .InsideColorIndex = wdNoHighlight
                .InsideLineStyle = wdLineStyleNone
                .InsideLineWidth = 0
                .OutsideColor = wdColorAutomatic
                .OutsideColorIndex = wdNoHighlight
                .OutsideLineStyle = wdLineStyleNone
                .OutsideLineWidth = 0
                .Shadow = False
            End With
        End With
    End If
    Dim eqnStyle As Object
    Set eqnStyle = Doc.Styles(mttstyle_EB_NUMBERED_EQUATION)
    Set GetEBNumberedEqnTableStyle = eqnStyle.Table
End Function

Public Function InsertEBNumberedEqnTable( _
    Doc As Document, _
    vTheEBEqn As Variant, _
    position As Boolean) _
    As Table
    
    Dim EBTableStyle As Object ' This is a TableStyle
    Dim docWidth As Long
    Dim colWidth As Long
    Dim c1and3Width As Long
    Dim c2Width As Long
    
    Dim theEBEqn
    Set theEBEqn = vTheEBEqn
    
    Set EBTableStyle = GetEBNumberedEqnTableStyle(Doc)
    
    Selection.TypeText Text:=" "  'work around the problem of table auto merge
    
    ActiveDocument.Tables.Add Range:=Selection.Range, numRows:=1, NumColumns:=3, _
       DefaultTableBehavior:=wdWord9TableBehavior, AutoFitBehavior:= _
        wdAutoFitWindow
    
    Set InsertEBNumberedEqnTable = Selection.Tables(1)
    With GetTable(Selection)
        .style = EBTableStyle.parent
        
        .ApplyStyleHeadingRows = False
        .ApplyStyleLastRow = False
        .ApplyStyleFirstColumn = False
        .ApplyStyleLastColumn = False
        .ApplyStyleRowBands = False
        .ApplyStyleColumnBands = False
        
        .Borders(wdBorderLeft).LineStyle = wdLineStyleNone
        .Borders(wdBorderRight).LineStyle = wdLineStyleNone
        .Borders(wdBorderTop).LineStyle = wdLineStyleNone
        .Borders(wdBorderBottom).LineStyle = wdLineStyleNone
        .Borders(wdBorderVertical).LineStyle = wdLineStyleNone
        .Borders(wdBorderDiagonalDown).LineStyle = wdLineStyleNone
        .Borders(wdBorderDiagonalUp).LineStyle = wdLineStyleNone
        .Borders.Shadow = False
      
        .Rows(1).HeightRule = wdRowHeightAuto
      
        .TopPadding = 3
        .BottomPadding = 3
        .LeftPadding = InchesToPoints(0)
        .RightPadding = InchesToPoints(0)

        .Spacing = 0
        .AllowPageBreaks = True
        .AllowAutoFit = True

        ' Set default column widths
        docWidth = GetDocWidth(Doc)
        c1and3Width = GetDefaultEqnNumberWidth(docWidth)
        c2Width = docWidth - 2 * c1and3Width
        SetEBNumTableColumnWidths aTable:=Selection.Tables(1), c1and3Width:=c1and3Width, c2Width:=c2Width
                          
        ' Format and fill (if necessary) the left number cell
        colWidth = FormatEBNumberCell(aCell:=.Cell(1, 1), insertNumber:=(position = False))
        .Cell(1, 1).Range.ParagraphFormat.Alignment = wdAlignParagraphLeft
        
        ' Format and fill (if necessary) the right number cell
        c1and3Width = FormatEBNumberCell(aCell:=.Cell(1, 3), insertNumber:=(position = True))
        .Cell(1, 3).Range.ParagraphFormat.Alignment = wdAlignParagraphRight
        
        ' Adjust the table
        If c1and3Width = 0 Then c1and3Width = colWidth
        If c1and3Width Then
            c2Width = docWidth - 2 * c1and3Width
            SetEBNumTableColumnWidths aTable:=Selection.Tables(1), c1and3Width:=c1and3Width, c2Width:=c2Width
        End If
        
        ' Format and fill the equation cell (the middle cell)
        With .Cell(1, 2)
            .TopPadding = 3
            .BottomPadding = 3
            .LeftPadding = InchesToPoints(0)
            .RightPadding = InchesToPoints(0)
            .VerticalAlignment = wdCellAlignVerticalCenter
            .Range.ParagraphFormat.Alignment = wdAlignParagraphCenter
            .Range.Cells.VerticalAlignment = wdCellAlignVerticalCenter
            'insert the equation
            If theEBEqn Is Nothing Then
                'Add new equation
                .Select
                Selection.moveLeft Unit:=wdCharacter, count:=1
                AddOMath Selection
            Else
                'Add existing equation
                .Select
                Selection.Paste
                'Done with the clipboard, so clear it out
                Dim MyData As MSForms.DataObject
                Set MyData = New MSForms.DataObject
                MyData.Clear
                ' Wrap in error handler, since failure to clear the clipboard
                ' for whatever reason is not error we want to deal with
                On Error Resume Next
                MyData.PutInClipboard
                On Error GoTo 0
            End If
        End With
        
        ActiveDocument.OMathLeftMargin = .Columns(1).width
        
        'work around the problem of table auto merge
         .Columns(1).Cells(1).Select
         Selection.moveLeft Unit:=wdCharacter, count:=4
         Dim isColumnBreak As Boolean
         If Strings.left$(Selection.Text, 1) <> Strings.Chr(14) Then
            isColumnBreak = False
         Else
            isColumnBreak = True
         End If
         Selection.MoveRight Unit:=wdCharacter, count:=1
         Selection.delete
         'If we're at the top top of a column (other than the first)
         ' then we must not delete the new-line character otherwise the
         ' whole table gets moved to the column on the left (this is
         ' Word's behavior -- it's a bug).
         If Not isColumnBreak Then Selection.delete
        
        .Rows(1).Alignment = wdAlignRowCenter
        .Select 'Select the whole table
    End With
End Function

' used by InsertEBNumberedEqnTable to get late binding
Private Function GetTable(sel)
    Set GetTable = sel.Tables(1)
End Function

' used by InsertEBNumberedEqnTable to get late binding
Private Sub AddOMath(sel)
    sel.OMaths.Add Range:=sel.Range
End Sub

Private Function GetDefaultEqnNumberWidth( _
    docWidth As Long) _
    As Long
    GetDefaultEqnNumberWidth = (docWidth * 25) / 100
End Function

' Determine if the current selection is inside an EB equation
' (Assumes Selection is IP, i.e. collapased)
Public Function SelInEBEquation() As Boolean
    Dim count As Long
    Dim start As Long
    Dim mathCount As Long
    MTLib.WriteLog "entering SelInEBEquation"
    SelInEBEquation = False
    If Val(Application.version) >= kWord2007 Then
        With GetSel(Selection)
            If .OMaths.count > 0 Then   'if selection has OMath then it might be in one
                Dim mathRange As Range
                Set mathRange = .OMaths(1).Range
                ' remember the start of this OMath
                start = mathRange.start
                
                ' Is the selection in the range of the OMath eqn?
                If .Range.InRange(mathRange) Then
                    ' Yes, so we think we're in the eqn, but ...
                    ' In this case we could be outside the eqn immediately to the left
                    count = .moveLeft ' try moving away from the eqn
                    
                    ' Did we really move away? If so mathCount will be zero
                    mathCount = .OMaths.count
                    If mathCount > 0 Then
                        If start <> .OMaths(1).Range.start Then
                            mathCount = 0
                        End If
                    End If
                        
                    If mathCount = 0 Then 'if selection no longer has OMath then it
                                          ' was just to the left of one, but not in
                        .MoveRight  ' restore selection
                        SelInEBEquation = False
                        Exit Function
                    Else  ' other wise we're in an EB eqn ...
                        If count = 0 Then 'unless we're at the beginning of the document
                            SelInEBEquation = False
                        Else
                            .MoveRight  ' restore selection
                            SelInEBEquation = True
                        End If
                        Exit Function
                    End If
                Else
                    ' No, so we think we're out of the eqn, but ...
                    ' In this case we could be inside the eqn at the far right
                    count = .MoveRight ' try moving out of the eqn
                    
                    ' Did we really move away? If so mathCount will be zero
                    mathCount = .OMaths.count
                    If mathCount > 0 Then
                        If start <> .OMaths(1).Range.start Then
                            mathCount = 0
                        End If
                    End If
                    
                    If mathCount = 0 Then 'if selection no longer has OMath then it
                                          ' was just to the right of one, but not in
                        .moveLeft  ' restore selection
                        SelInEBEquation = False
                        Exit Function
                    Else ' other wise we're in an EB eqn ...
                        If count = 0 Then 'unless we're at the end of the document
                            SelInEBEquation = False
                        Else
                            .moveLeft  ' restore selection
                            SelInEBEquation = True
                        End If
                        Exit Function
                    End If
                End If
            End If
            SelInEBEquation = False
        End With
    End If
    MTLib.WriteLog "exiting SelInEBEquation"
End Function

' used by SelInEBEquation and NumberExistingEBEqn to get late binding
Private Function GetSel(sel)
    Set GetSel = sel
End Function

' Determine if the current selection is inside an EB numbered equation table
' (Assumes Selection is IP, i.e. collapased)
' Returns position in EB numbered eqn table:
'   0 for Not in EB table
'   1 for in leftmost cell
'   2 for left of the EB equation in the middle cell
'   3 for inside the EB equation in the middle cell
'   4 for right of the EB equation in the middle cell
'   5 for in rightmost cell
Public Function SelInEBNumberedEqnTable() As Long
    Dim left As Long
    Dim right As Long
    With Selection
        SelInEBNumberedEqnTable = 0
        If Not .Information(wdWithInTable) Then Exit Function
        
        Dim aTable As Object ' This is a Table
        Set aTable = .Tables(1)
        Dim aStyle As style
        Set aStyle = aTable.style
        If aStyle.NameLocal <> mttstyle_EB_NUMBERED_EQUATION Then Exit Function
            
        With .Cells(1)
            ' In column 1
            If .ColumnIndex = 1 Then
                SelInEBNumberedEqnTable = 1
                Exit Function
            End If
            ' In column 2
            If .ColumnIndex = 2 Then
                If SelInEBEquation() Then
                    SelInEBNumberedEqnTable = 3
                    Exit Function
                End If
                
                Selection.moveLeft
                If SelInEBEquation() Then
                    Selection.MoveRight
                    SelInEBNumberedEqnTable = 4
                    Exit Function
                End If
                Selection.MoveRight
                
                Selection.MoveRight
                If SelInEBEquation() Then
                    Selection.moveLeft
                    SelInEBNumberedEqnTable = 2
                    Exit Function
                End If
                Selection.moveLeft
                ' Should not get here!
                SelInEBNumberedEqnTable = 3
                Exit Function
            End If
            ' In column 3
            If .ColumnIndex = 3 Then
                SelInEBNumberedEqnTable = 5
                Exit Function
            End If
            
            SelInEBNumberedEqnTable = 2 'Should not happen
            Exit Function
        End With
    End With
End Function

Private Sub SetEBNumTableColumnWidths( _
    aTable As Table, _
    c1and3Width As Long, _
    c2Width As Long)

    Dim width As Long
    With aTable
        .PreferredWidthType = wdPreferredWidthPercent
        .PreferredWidth = 100
            
        .Columns.item(1).PreferredWidthType = wdPreferredWidthPoints
        .Columns.item(1).PreferredWidth = c1and3Width
        
        .Columns(3).PreferredWidthType = wdPreferredWidthPoints
        .Columns(3).PreferredWidth = c1and3Width

        .Columns(2).PreferredWidthType = wdPreferredWidthPoints
        .Columns(2).PreferredWidth = c2Width
    End With
End Sub

Private Function FormatEBNumberCell( _
    aCell As Cell, _
    insertNumber As Boolean _
  ) As Long
    
    FormatEBNumberCell = 0
    With aCell
        .TopPadding = 3
        .BottomPadding = 3
        .LeftPadding = InchesToPoints(0)
        .RightPadding = InchesToPoints(0)
        .VerticalAlignment = wdCellAlignVerticalCenter
        .Range.Cells.VerticalAlignment = wdCellAlignVerticalCenter
        If insertNumber Then
            .Select
            MTEqnNum.InsertEqnNum
            .Column.AutoFit ' Size the column to the number
            FormatEBNumberCell = .width
        End If
    End With
End Function

Private Function MoveOutOfEBNumberedEqnTable( _
    position As Long _
  ) As Boolean

    MoveOutOfEBNumberedEqnTable = False
    With Selection
    If position > 0 Then
        If (position = 1) Or (position = 2) Then
            While Selection.Information(wdWithInTable)
                If .moveLeft = 0 Then Exit Function
            Wend
        End If
        If (position = 4) Or (position = 5) Then
            While Selection.Information(wdWithInTable)
                If .MoveRight = 0 Then Exit Function
            Wend
        End If
    End If
    End With
    MoveOutOfEBNumberedEqnTable = True
End Function

Function NumberExistingEBEqn() As Boolean
    NumberExistingEBEqn = False
     
    Dim title As String
    title = MTLib.GetUserString2("1105", "1305", "Number Equation Builder Equation")
    
    If IsInEBEquationTable(title) Then
        NumberExistingEBEqn = True
        GoTo abort
    End If
    
    Dim savedSelRange As Range
    Set savedSelRange = Selection.Range
    
    Dim dir As Boolean
    Dim isNextToEBDisplayEqn As Boolean
    Dim isInTable As Boolean
    Dim inRow As Long
    Dim inCol As Long
    Dim doDelete As Boolean
   
    isNextToEBDisplayEqn = False
    If IsSelLeftOfEBDisplayEqn() Then
        dir = False
        isNextToEBDisplayEqn = True
    ElseIf IsSelRightOfEBDisplayEqn() Then
        dir = True
        isNextToEBDisplayEqn = True
    End If
    If isNextToEBDisplayEqn Then
        ' Disallow EB numbering in Outline view
        If IsInOutlineView(title) Then
            NumberExistingEBEqn = True
            GoTo abort
        End If
    
        With GetSel(Selection)
            Dim numTable As Table
            Dim theEBEqn
            Set theEBEqn = .OMaths(1)
            .OMaths(1).Range.Select
            .Cut
            isInTable = .Information(wdWithInTable)
            inCol = Selection.Information(wdEndOfRangeColumnNumber)
            inRow = Selection.Information(wdEndOfRangeRowNumber)
            
            ' Note: WrapEBEqnWithTableAndNumber adds an extra paragraph after the
            ' inserted table -- we get rid of it below
            Set numTable = WrapEBEqnWithTableAndNumber(theEBEqn, dir)
            If numTable Is Nothing Then ' if the user cancelled
                .Paste
                If dir Then
                    .EndKey
                Else
                    .HomeKey
                    .moveLeft
                End If
                NumberExistingEBEqn = True
                GoTo abort
            Else
                MTIncrementStatisticBy "CEBEqNum", 1
                NumberExistingEBEqn = True
            End If
        End With
    End If
    
    'Restore the selection if we didn't do anything
    If Not NumberExistingEBEqn Then
        savedSelRange.Select
    Else
        Selection.MoveDown 'Move out of the EB number table (just below)
        'Place the cursor based on whether we're in a table or not
        If isInTable Then
            doDelete = True
            Selection.MoveRight
            If isInTable <> Selection.Information(wdWithInTable) Then doDelete = False
            If inCol <> Selection.Information(wdEndOfRangeColumnNumber) Then doDelete = False
            If inRow <> Selection.Information(wdEndOfRangeRowNumber) Then doDelete = False
            Selection.moveLeft
            If doDelete And Asc(Selection.Text) = 13 Then
                Selection.delete
                If SelInEBEquation Then ' If we're inside an EB eqn edit window ...
                    Selection.moveLeft  '  move out of it
                End If
            End If
        Else
            Selection.delete 'Delete extra paragraph (this does nothing if we're at the end of the document)
            If SelInEBEquation Then ' If we're inside an EB eqn edit window ...
                Selection.moveLeft  '  move out of it
            End If
        End If
    End If

abort:
End Function

Function WrapEBEqnWithTableAndNumber( _
    vTheEBEqn As Variant, _
    dir As Boolean _
  ) As Table
    
    Dim numTable As Table
    
    Dim theEBEqn
    Set theEBEqn = vTheEBEqn
    
    MTCommandsMain.InsertEBNumberedEquationEx theEBEqn, dir
    If Selection.Tables.count = 1 Then
        Set numTable = Selection.Tables(1)
        Set WrapEBEqnWithTableAndNumber = numTable
    Else
        Set WrapEBEqnWithTableAndNumber = Nothing
    End If
End Function

Function IsSelLeftOfEBDisplayEqn() As Boolean
    Dim selRange As Range
    IsSelLeftOfEBDisplayEqn = False
    If Val(Application.version) >= kWord2007 Then
        With Selection
            Set selRange = .Range
            .Collapse wdCollapseStart
            IsSelLeftOfEBDisplayEqn = IsLeftOfEBDisplayEqn(.Range)
            If Not IsSelLeftOfEBDisplayEqn Then
                selRange.Select 'Restore the selection
            End If
        End With
    End If
End Function

Function IsLeftOfEBDisplayEqn( _
    theRange As Range _
  ) As Boolean
  
    Dim count As Long
    Dim moved As Long
    IsLeftOfEBDisplayEqn = False
    If Val(Application.version) >= kWord2007 Then
        With GetRng(theRange)
            If .OMaths.count = 1 Then
                If .OMaths(1).Type <> kWDOMathDisplay Then Exit Function
                moved = .MoveStart(wdCharacter, -1)
                count = .OMaths.count
                .MoveStart wdCharacter, 1    'restore to original position
                If (moved = 0) Or (count = 0) Then
                    'moved = 0 => start of document (so left of the OMath)
                    'count = 0 => we were at the start of the OMath before the move
                    IsLeftOfEBDisplayEqn = True
                End If
            End If
        End With
    End If
End Function
' called by IsLeftOfEBDisplayEqn and IsRightOfEBDisplayEqn to get late binding
Private Function GetRng(rng)
    Set GetRng = rng
End Function

Function IsSelRightOfEBDisplayEqn() As Boolean
    Dim selRange As Range
    IsSelRightOfEBDisplayEqn = False
    If Val(Application.version) >= kWord2007 Then
        With Selection
            Set selRange = .Range
            .Collapse wdCollapseEnd
            IsSelRightOfEBDisplayEqn = IsRightOfEBDisplayEqn(.Range)
            If Not IsSelRightOfEBDisplayEqn Then selRange.Select
        End With
    End If
End Function

Function IsRightOfEBDisplayEqn( _
    theRange As Range _
  ) As Boolean
  
    Dim count As Long
    Dim moved As Long
    IsRightOfEBDisplayEqn = False
    If Val(Application.version) >= kWord2007 Then
        With GetRng(theRange)
            If .OMaths.count = 1 Then
                If .OMaths(1).Type <> kWDOMathDisplay Then Exit Function
                moved = .MoveEnd(wdCharacter, 1)
                count = .OMaths.count
                .MoveEnd wdCharacter, -1
                'moved = 0 => end of document (so right of the OMath)
                'count = 0 => we were at the end of the OMath before the move
                If (moved = 0) Or (count = 0) Then
                    IsRightOfEBDisplayEqn = True
                End If
            End If
        End With
    End If
End Function

