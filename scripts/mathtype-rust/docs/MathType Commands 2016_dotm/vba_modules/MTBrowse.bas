Attribute VB_Name = "MTBrowse"
'MTBrowse 5.0
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTBrowse.bas 18    5/06/14 9:54a Jimm $
'=====================================================================

Public Const SelectFloatShape As Long = 0
Public Const SelectInlineShape As Long = 1
Public Const SelectFields As Long = 2
Public Const SelectOMath As Long = 3
Public Const BookMarkGoto As String = "TEMPGOTO"

'-------- entry points -------------
Public Sub BrowseEquationsForward()
    Browse True, 1
End Sub

Public Sub BrowseEquationsBackward()
    Browse False, 1
End Sub

Public Sub BrowseEquationNumberRefForward()
    Browse True, 2
End Sub

Public Sub BrowseEquationNumberRefBackward()
    Browse False, 2
End Sub

Public Sub BrowseChapterSectionForward()
    Browse True, 3
End Sub

Public Sub BrowseChapterSectionBackward()
    Browse False, 3
End Sub

'------- implementation ---------------
'direction -> true==forward, false==backward
Private Sub Browse(forward As Boolean, MTBrowseType As Long)
    'turn off screen updates while searching due to changing the selection
    Application.ScreenUpdating = False
    
    'save the current selection in case no items found
    Dim saveStart As Long
    Dim saveEnd As Long
    saveStart = Selection.start
    saveEnd = Selection.end
    
    'browse to the item
    Dim found As Boolean
    If MTBrowseType = 0 Or MTBrowseType = 1 Then
        ' 0 means no selection (we assume equations)
        ' 1 means equations
        found = BrowseEquations(forward)
    Else
        ' all other commands handled by BrowseGoto
        found = BrowseOther(forward, MTBrowseType)
    End If
    
    'if nothing found, restore the user's selection
    If Not found Then
        Selection.start = saveStart
        Selection.end = saveEnd
    End If
        
    ' since ActiveWindow.ScrollIntoView is buggy, we need to use this
    ' workaround to make sure that selection is in view
    If Selection.Type = wdSelectionShape Then
        ' for shape selections, we need to avoid using the bookmark technique
        ' and hope it works like it supposed to
        #If Win32 Then
        ActiveWindow.ScrollIntoView Selection.Range
        #End If
    Else
        Dim abookmark As Bookmark
        Set abookmark = ActiveDocument.Bookmarks.Add(BookMarkGoto, Selection.Range)
        abookmark.Select
        abookmark.delete
    End If
        
    'update the screen
    Application.ScreenUpdating = True
    Application.ScreenRefresh
End Sub

' browses through the document looking for equations in a specified order
'direction -> true==forward, false==backward
Function BrowseEquations(forward As Boolean) As Boolean
    BrowseEquations = False
    Dim currentStory As WdStoryType
    Dim currentStoryIndex As Long
    
    Dim storyRng As Range
    Dim originalSelectionRng As Range
    Dim modifiedSelectionRng As Range
    Dim storyOrder As Collection
    
    If Documents.count = 0 Then
        Exit Function
    End If
    
    Set originalSelectionRng = Selection.Range
    
    ' collapse the selection
    If forward Then
        Selection.Collapse wdCollapseEnd
    Else
        Selection.Collapse wdCollapseStart
    End If
    
    Set modifiedSelectionRng = Selection.Range
    
    ' setup the order in which we will iterate through
    ' stories in the document. Each entry in the collection
    ' is of the following format:
    '
    ' index i = value: array(index i,story type)
    '           key: story type as string (used for searches)
    '
    ' we do this so we can easily determine the position in our story order
    ' by searching for the key. The key search is done using
    ' the current cursor location.
    Set storyOrder = New Collection
    storyOrder.Add Array(storyOrder.count + 1, WdStoryType.wdMainTextStory), CStr(WdStoryType.wdMainTextStory)
    storyOrder.Add Array(storyOrder.count + 1, WdStoryType.wdFootnotesStory), CStr(WdStoryType.wdFootnotesStory)
    storyOrder.Add Array(storyOrder.count + 1, WdStoryType.wdEndnotesStory), CStr(WdStoryType.wdEndnotesStory)
    storyOrder.Add Array(storyOrder.count + 1, WdStoryType.wdFirstPageHeaderStory), CStr(WdStoryType.wdFirstPageHeaderStory)
    storyOrder.Add Array(storyOrder.count + 1, WdStoryType.wdPrimaryHeaderStory), CStr(WdStoryType.wdPrimaryHeaderStory)
    storyOrder.Add Array(storyOrder.count + 1, WdStoryType.wdEvenPagesHeaderStory), CStr(WdStoryType.wdEvenPagesHeaderStory)
    storyOrder.Add Array(storyOrder.count + 1, WdStoryType.wdFirstPageFooterStory), CStr(WdStoryType.wdFirstPageFooterStory)
    storyOrder.Add Array(storyOrder.count + 1, WdStoryType.wdPrimaryFooterStory), CStr(WdStoryType.wdPrimaryFooterStory)
    storyOrder.Add Array(storyOrder.count + 1, WdStoryType.wdEvenPagesFooterStory), CStr(WdStoryType.wdEvenPagesFooterStory)
    storyOrder.Add Array(storyOrder.count + 1, WdStoryType.wdTextFrameStory), CStr(WdStoryType.wdTextFrameStory)
    storyOrder.Add Array(storyOrder.count + 1, WdStoryType.wdCommentsStory), CStr(WdStoryType.wdCommentsStory)
    
    currentStory = Selection.storyType
    currentStoryIndex = storyOrder(CStr(currentStory))(0)
    
    Dim i As Integer
    If forward Then ' search forward
        For i = currentStoryIndex To storyOrder.count
            currentStory = storyOrder.item(i)(1)
            On Error Resume Next
            Set storyRng = ActiveDocument.StoryRanges(currentStory)
            If err.Number = 5941 Then
                ' this error means the document does not have
                ' the "currentStory" range in this document,
                ' so skip it and move on.
            Else
                If BrowseByStoryRange(storyRng, modifiedSelectionRng, forward) Then
                    BrowseEquations = True
                    Exit Function
                End If
            End If
        Next
    Else ' search backwards
        For i = currentStoryIndex To 1 Step -1
            currentStory = storyOrder.item(i)(1)
            On Error Resume Next
            Set storyRng = ActiveDocument.StoryRanges(currentStory)
            If err.Number = 5941 Then
                ' this error means the document does not have
                ' the "currentStory" range in this document,
                ' so skip it and move on.
            Else
                If BrowseByStoryRange(storyRng, modifiedSelectionRng, forward) Then
                    BrowseEquations = True
                    Exit Function
                End If
            End If
        Next
    End If
End Function

'helper function for browsing equations
'returns true if item found, false otherwise
Function BrowseByStoryRange(storyRng As Range, selRng As Range, forward As Boolean) As Boolean
    BrowseByStoryRange = False
    
    ' there can be more than one range associated with
    ' a story, so we need to check them all
    ' since we can only iterate forward through these stories,
    ' we will place each story in a collection, which will
    ' allow us to iterate forward and backward through the stories
    
    Dim storyRngCollection As Collection
    Set storyRngCollection = New Collection
    
    Dim storyRngIter As Range
    Set storyRngIter = storyRng
    
    Do
        storyRngCollection.Add storyRngIter
        Set storyRngIter = storyRngIter.NextStoryRange
    Loop While Not (storyRngIter Is Nothing)
    
    
    ' this value is used to indicate that the current story range
    ' and all subsequent ranges should be checked
    Dim startChecking As Boolean
    startChecking = False
    
    Dim i As Integer
    Dim newRange As Range
    Dim aRange As Range
    If forward Then
        ' iterate forward though the collection
        For i = 1 To storyRngCollection.count
            Set storyRngIter = storyRngCollection.item(i)
            
            ' recall that we need to search starting with the current selection
            ' location, and then go forward or backwards in the "story order".
            ' We need to handle the case where we have ventured outside the
            ' "starting story" differently
            If storyRngIter.storyType <> selRng.storyType Then
                
                Set newRange = storyRngIter.Duplicate
                newRange.StartOf WdUnits.wdStory
                If SelectEquationInRange(storyRngIter, newRange, forward) Then
                    BrowseByStoryRange = True
                    Exit For
                End If
            Else
                ' we need to see if the selection range is contained
                ' by the story range, which will limit checks
                If selRng.InRange(storyRngIter) Then
                    startChecking = True
                    Set aRange = selRng
                ElseIf startChecking Then
                    ' checking has been enabled in a prior loop,
                    ' and the selection range is not contained by the
                    ' story range so we should create a new range
                    Set aRange = storyRngIter.Duplicate
                    aRange.StartOf WdUnits.wdStory
                End If
                
                If startChecking = True Then
                    ' check this range for equations
                    If SelectEquationInRange(storyRngIter, aRange, forward) Then
                        BrowseByStoryRange = True
                        Exit For
                    End If
                End If
            End If
        Next
    Else
        ' iterate backwards through the collection
        For i = storyRngCollection.count To 1 Step -1
            Set storyRngIter = storyRngCollection.item(i)
            ' recall that we need to search starting with the current selection
            ' location, and then go forward or backwards in the "story order".
            ' We need to handle the case where we have ventured outside the
            ' "starting story" differently
            If storyRngIter.storyType <> selRng.storyType Then
                Set newRange = storyRngIter.Duplicate
                newRange.EndOf WdUnits.wdStory
                If SelectEquationInRange(storyRngIter, newRange, forward) Then
                    BrowseByStoryRange = True
                    Exit For
                End If
            Else
                ' we need to see if the selection range is contained
                ' by the story range, which will limit checks
                If selRng.InRange(storyRngIter) Then
                    startChecking = True
                    Set aRange = selRng
                ElseIf startChecking Then
                    ' checking has been enabled in a prior loop,
                    ' and the selection range is not contained by the
                    ' story range so we should create a new range
                    Set aRange = storyRngIter.Duplicate
                    aRange.EndOf WdUnits.wdStory
                End If
                
                If startChecking = True Then
                    ' check this range for equations
                    If SelectEquationInRange(storyRngIter, aRange, forward) Then
                        BrowseByStoryRange = True
                        Exit For
                    End If
                End If
            End If
        Next
    End If
End Function

'helper function for browsing equations
' storyRng         represents the range for the entire story
' searchFromRng    represents the range of the current selection, or a range that has been
'                  set to the start or end of a story
' forward          true for browse forward, false for browse backward
Function SelectEquationInRange(storyRng As Range, searchFromRng As Range, forward As Boolean) As Boolean
    Dim ebEQPos As Long ' used to mark either the start or end position of a EB equation
    Dim mtEQFloatingShapesPos As Long ' used to mark either the start or end position of a MT equation
    Dim mtEQInlineShapesPos As Long ' used to mark either the start or end position of a MT equation
    Dim mtEQFieldsPos As Long ' used to mark either the start or end position of a MT equation
    
    Dim ebEQIndex As Long ' the index number of the found EB equation
    Dim mtEQFloatingShapesIndex As Long ' the index number of the found MT equation
    Dim mtEQInlineShapesIndex As Long ' the index number of the found MT equation
    Dim mtEQFieldsIndex As Long ' the index number of the found MT equation
    
    ' intialize to -1 to indicate nothing is found
    ebEQPos = -1
    mtEQFloatingShapesPos = -1
    mtEQInlineShapesPos = -1
    mtEQFieldsPos = -1
    
    ' used as index during iteration
    Dim i As Integer
    
    ''' OLE1, OLE2, non-OLE MathType equation
    ' floating shapes
    ' the shape collection is not always available, and will give
    ' a "not available" error.
    On Error Resume Next
    Dim tmpRng As ShapeRange
    Set tmpRng = storyRng.ShapeRange
    If err.Number = 0 Then
        With tmpRng
            If forward Then
                If .count <> 0 Then
                    For i = 1 To .count
                        If MTBrowse.IsShapeEquation(.item(i)) Then
                            If searchFromRng.start <= .item(i).Anchor.start Then
                                ' store the position and index of the equation
                                mtEQFloatingShapesPos = .item(i).Anchor.start
                                mtEQFloatingShapesIndex = i
                                Exit For
                            End If
                        End If
                    Next
                End If
            Else
                If .count <> 0 Then
                    For i = .count To 1 Step -1
                        If MTBrowse.IsShapeEquation(.item(i)) Then
                            If searchFromRng.end >= .item(i).Anchor.end Then
                                ' store the position and index of the equation
                                mtEQFloatingShapesPos = .item(i).Anchor.end
                                mtEQFloatingShapesIndex = i
                                Exit For
                            End If
                        End If
                    Next
                End If
            End If
        End With
    End If
    
    ' inlineshapes.
    With storyRng.InlineShapes
        If forward Then
            If .count <> 0 Then
                For i = 1 To .count
                    If IsInlineShapeEquation(.item(i)) Then
                        If searchFromRng.start <= .item(i).Range.start Then
                            ' store the position and index of the equation
                            mtEQInlineShapesPos = .item(i).Range.start
                            mtEQInlineShapesIndex = i
                            Exit For
                        End If
                    End If
                Next
            End If
        Else
            If .count <> 0 Then
                For i = .count To 1 Step -1
                    If IsInlineShapeEquation(.item(i)) Then
                        If searchFromRng.end >= .item(i).Range.end Then
                            ' store the position and index of the equation
                            mtEQInlineShapesPos = .item(i).Range.end
                            mtEQInlineShapesIndex = i
                            Exit For
                        End If
                    End If
                Next
            End If
        End If
    End With
    
    ''' MathType 1.x macro equations or MS word formula fields
    With storyRng.Fields
        If forward Then
            If .count <> 0 Then
                For i = 1 To .count
                    If IsFieldEquation(.item(i)) Then
                        If searchFromRng.start <= .item(i).Code.start Then
                            ' store the position and index of the equation
                            mtEQFieldsPos = .item(i).Code.start
                            mtEQFieldsIndex = i
                            Exit For
                        End If
                    End If
                Next
            End If
        Else
            If .count <> 0 Then
                For i = .count To 1 Step -1
                    If IsFieldEquation(.item(i)) Then
                        If searchFromRng.end >= .item(i).Code.end Then
                            ' store the position and index of the equation
                            mtEQFieldsPos = .item(i).Code.end
                            mtEQFieldsIndex = i
                            Exit For
                        End If
                    End If
                Next
            End If
        End If
    End With
    ' Word 2007 Equation Builder equations
    If Val(Application.version) >= 12 Then
        ' iterate through all of the EB equations
        Dim ebEQ
        Set ebEQ = GetOMathCol(storyRng)
        With ebEQ
            If .count > 0 Then
                If forward Then ' if browsing forward
                    For i = 1 To .count
                        ' check the position against this particaular equation
                        If searchFromRng.start <= .item(i).Range.start Then
                            ' store the position and index of the equation
                            ebEQPos = .item(i).Range.start
                            ebEQIndex = i
                            Exit For
                        End If
                    Next
                Else ' if browsing backward
                    For i = .count To 1 Step -1
                        ' check the position against this particaular equation
                        If searchFromRng.end >= .item(i).Range.end Then
                            ' store the position and index of the equation
                            ebEQPos = .item(i).Range.end
                            ebEQIndex = i
                            Exit For
                        End If
                    Next
                End If
            Else ' no EB equations
                ebEQPos = -1
            End If
        End With
    End If
    
    ' now analyze the results and act accordingly
    Dim arr(1 To 2, 1 To 4) As Long
    arr(1, 1) = ebEQPos
    arr(1, 2) = mtEQFieldsPos
    arr(1, 3) = mtEQFloatingShapesPos
    arr(1, 4) = mtEQInlineShapesPos
    arr(2, 1) = SelectOMath
    arr(2, 2) = SelectFields
    arr(2, 3) = SelectFloatShape
    arr(2, 4) = SelectInlineShape
    
    ' now find the equation that is closest to where we started from
 
    ' used to keep track of the smallest distance from where we started to
    ' an equation
    Dim minimumVal As Long
    ' where we started from
    Dim comparison As Long
    ' the "type" of equation that we should select
    Dim eqType As Long
    ' initialize variables
    If forward Then
        comparison = searchFromRng.start
    Else
        comparison = searchFromRng.end
    End If
    minimumVal = -1
    ' now loop through and find the closest equation to where we started from
    For i = 1 To 4
        If arr(1, i) > -1 Then
            If minimumVal = -1 Then
                minimumVal = Math.Abs(comparison - arr(1, i))
                eqType = arr(2, i)
            ElseIf Math.Abs(comparison - arr(1, i)) < minimumVal Then
                minimumVal = Math.Abs(comparison - arr(1, i))
                eqType = arr(2, i)
            End If
            
            SelectEquationInRange = True
        End If
    Next
    ' since we initialized minimumVal to -1, make sure its not still set to that value
    If minimumVal <> -1 Then
        ' now make our selection
        Select Case eqType
            Case SelectOMath
                If SelectRange(storyRng, eqType, ebEQIndex) = False Then
                    SelectEquationInRange = False
                End If
            Case SelectFields
                If SelectRange(storyRng, eqType, mtEQFieldsIndex) = False Then
                    SelectEquationInRange = False
                End If
            Case SelectFloatShape
                If SelectRange(storyRng, eqType, mtEQFloatingShapesIndex) = False Then
                    SelectEquationInRange = False
                End If
            Case SelectInlineShape
                If SelectRange(storyRng, eqType, mtEQInlineShapesIndex) = False Then
                    SelectEquationInRange = False
                End If
        End Select
    End If
    
End Function

' This function exists to fetch the OMaths object from a range.
' It allows us to prevent compile problems in versions of office
' prior to office 2007
Function GetOMathCol(storyRng)
    Set GetOMathCol = storyRng.OMaths
End Function

'helper function for browsing equations
Sub Sort(arr() As Long)

    Dim TempPos As Double
    Dim TempConst As Long
    Dim i As Long
    Dim j As Long
    
    For j = 2 To UBound(arr, 2)
        TempPos = arr(1, j)
        TempConst = arr(2, j)
        For i = j - 1 To 1 Step -1
            If (arr(1, i) <= TempPos) Then GoTo skip
            arr(1, i + 1) = arr(1, i) ' position
            arr(2, i + 1) = arr(2, i) ' const
        Next i
        i = 0
skip:   arr(1, i + 1) = TempPos
        arr(2, i + 1) = TempConst
    Next j
    
End Sub


' helper function for browsing equations
' This function will select an equation using the following values:
' storyRange = the storyRange that contains the item we want to select
' selItem = a constant "SelectXXX" constant (SelectFloatShape, SelectInlineShape, SelectFields, SelectOMath)
' index = index in the appropriate collection
Function SelectRange(storyRange As Range, selItem As Long, index As Long) As Boolean
    SelectRange = True
    Dim aWindow As Window
    Set aWindow = ActiveDocument.ActiveWindow
    ' change the view to print view
    aWindow.View = wdPrintView
    
    ' put word into the correct view state given the type of story range
    Select Case storyRange.storyType
        Case WdStoryType.wdMainTextStory
            aWindow.View.SeekView = wdSeekMainDocument
        Case WdStoryType.wdFootnotesStory
            aWindow.View.SeekView = wdSeekFootnotes
        Case WdStoryType.wdEndnotesStory
            aWindow.View.SeekView = wdSeekEndnotes
        Case WdStoryType.wdFirstPageHeaderStory
            aWindow.View.SeekView = wdSeekFirstPageHeader
        Case WdStoryType.wdPrimaryHeaderStory
            aWindow.View.SeekView = wdSeekPrimaryHeader
        Case WdStoryType.wdEvenPagesHeaderStory
            aWindow.View.SeekView = wdSeekEvenPagesHeader
        Case WdStoryType.wdFirstPageFooterStory
            aWindow.View.SeekView = wdSeekFirstPageFooter
        Case WdStoryType.wdPrimaryFooterStory
            aWindow.View.SeekView = wdSeekPrimaryFooter
        Case WdStoryType.wdEvenPagesFooterStory
            aWindow.View.SeekView = wdSeekEvenPagesFooter
        Case WdStoryType.wdTextFrameStory
            aWindow.View.SeekView = wdSeekMainDocument
        Case WdStoryType.wdCommentsStory
            aWindow.View.SeekView = wdSeekMainDocument
    End Select
    
    ' now finally make the selection of the item
    Select Case selItem
        Case SelectFloatShape
            storyRange.ShapeRange(index).Select
        Case SelectInlineShape
            storyRange.InlineShapes(index).Range.Select
        Case SelectFields
            storyRange.Fields(index).Select
        Case SelectOMath
            Dim ebEQ
            Set ebEQ = GetOMathCol(storyRange)
            ebEQ(index).Range.Select
    End Select
    
End Function

' helper function for browsing equations
' returns true if aShape represents an MT equation, false otherwise
Public Function IsShapeEquation(aShape As Shape) As Boolean
    IsShapeEquation = False
    Dim isEquation As Long
    If aShape.Type = msoEmbeddedOLEObject Then
        'accessing ProgID may cause an error 5825 on corrupt objects, so use ClassType instead
        isEquation = IsEquationProgID(aShape.OLEFormat.ClassType)
    End If
    If isEquation = 0 And (aShape.Type = msoPicture) Then
        'select the object & copy to clipboard
        aShape.Select
        On Error Resume Next
        Selection.Copy 'may fail with error 4198
        If err.Number = 0 Then
            If IsPictureEquation Then
                isEquation = 2
            End If
        End If
    End If
    If isEquation > 0 Then
        IsShapeEquation = True
    End If
End Function

' helper function for browsing equations
' returns true if aShape represents an MT equation, false otherwise
Public Function IsInlineShapeEquation(aShape As InlineShape) As Boolean
    IsInlineShapeEquation = False
    Dim isEquation As Long
    If aShape.Type = wdInlineShapeEmbeddedOLEObject Then
        'accessing ProgID may cause an error 5825 on corrupt objects, so use ClassType instead
        isEquation = IsEquationProgID(aShape.OLEFormat.ClassType)
    End If
    If isEquation = 0 And (aShape.Type = wdInlineShapePicture Or _
                           aShape.Type = wdInlineShapeLinkedPicture) Then
        'select the object & copy to clipboard
        aShape.Select
        On Error Resume Next
        Selection.Copy 'may fail with error 4198
        If err.Number = 0 Then
            If IsPictureEquation Then
                isEquation = 2
            End If
        End If
    End If
    If isEquation > 0 Then
        IsInlineShapeEquation = True
    End If
End Function

' helper function for browsing equations
' returns true if aField represents an MT equation, false otherwise
Public Function IsFieldEquation(aField As Field) As Boolean
    IsFieldEquation = False
    Dim fieldText As String
    fieldText = Strings.LCase$(aField.Code.Text)
    If (aField.Type = wdFieldFormula) Or _
        (Len(fieldText) > 1 And _
        (InStr(1, fieldText, "edittexteqn", vbBinaryCompare) <> 0 Or _
        InStr(1, fieldText, "editdispeqn", vbBinaryCompare) <> 0)) Then
        ' "EditTextEqn", "EditDispEqn"
        IsFieldEquation = True
    End If
End Function

' helper function for browsing equations
' returns true if there is a picture on the clipboard and it contains
' MTEF, false otherwise
Function IsPictureEquation() As Boolean
    IsPictureEquation = True
    Dim result As Long
    On Error GoTo bye
   
    'Use API call to check clipboard contents first
    'Fails to detect WMFs with MTEF after Word 2000.  See MT-1131
    result = MTEquationOnClipboard()
    If result = mtNOT_EQUATION Then
        IsPictureEquation = False
    End If
    Exit Function
bye:
    IsPictureEquation = False
End Function



'browse through equation numbers or references as well as chapter or section breaks
Private Function BrowseOther(forward As Boolean, MTBrowseType As Long) As Boolean
    Dim found As Boolean ' indicates if we have found the item we are looking for
    Dim originalStart As Long ' marks the start point of the current cursor location
    Dim originalEnd As Long ' marks the end point of the current cursor location
    Dim posStart As Long ' marks the start point of the current cursor location after we move it
    Dim posEnd As Long ' marks the end point of the current cursor location after we move it
    Dim i As Integer
    
    found = False
    
    ' keep the original selection position
    originalStart = Selection.start
    originalEnd = Selection.end

    ' collapse the selection
    If forward Then
        Selection.Collapse wdCollapseEnd
    Else
        Selection.Collapse wdCollapseStart
    End If
    
    ' mark the new position after selection collapse
    posStart = Selection.start
    posEnd = Selection.end
    
    'iterate through all of the fields
    With ActiveDocument.Fields
        If .count > 0 Then
            If forward Then ' traverse forward
                For i = 1 To .count
                    ' check the position against this particaular field and see if it is an equation
                    If posStart <= .item(i).Code.start Then
                        If MTBrowseType = 2 Then 'equation number or reference
                            ' check to make sure the field is what we expect
                            If .item(i).Type = wdFieldMacroButton And _
                                InStr(1, .item(i).Code.Text, "MTPlaceRef", 1) Then
                                ' OK to make selection
                                .item(i).Select
                                found = True
                                Exit For
                            ElseIf .item(i).Type = wdFieldGoToButton And _
                                InStr(1, .item(i).Code.Text, "ZEqnNum", 1) Then
                                ' OK to make selection
                                .item(i).Select
                                found = True
                                Exit For
                            End If
                        Else 'chapter or section break
                            ShowSectionStyle
                            ' check to make sure the field is what we expect
                            If .item(i).Type = wdFieldMacroButton Then
                                If InStr(1, .item(i).Code.Text, "MTEditEquationSection", 1) Or _
                                       InStr(1, .item(i).Code.Text, "Chapter", 1) Then
                                    
                                    .item(i).Select
                                    found = True
                                    Exit For
                                End If
                            End If
                        End If
                    End If
                Next
            Else ' traverse backwards
                For i = .count To 1 Step -1
                    If posEnd >= .item(i).Code.end Then
                        If MTBrowseType = 2 Then 'equation number or reference
                            ' check to make sure the field is what we expect
                            If .item(i).Type = wdFieldMacroButton And _
                                InStr(1, .item(i).Code.Text, "MTPlaceRef", 1) Then
                                ' OK to make selection
                                .item(i).Select
                                found = True
                                Exit For
                            ElseIf .item(i).Type = wdFieldGoToButton And _
                                InStr(1, .item(i).Code.Text, "ZEqnNum", 1) Then
                                ' OK to make selection
                                .item(i).Select
                                found = True
                                Exit For
                            End If
                        Else 'chapter or section break
                            ShowSectionStyle
                            ' check to make sure the field is what we expect
                            If .item(i).Type = wdFieldMacroButton Then
                                If InStr(1, .item(i).Code.Text, "MTEditEquationSection", 1) Or _
                                       InStr(1, .item(i).Code.Text, "Chapter", 1) Then
                                    ' OK to make selection
                                    .item(i).Select
                                    found = True
                                    Exit For
                                End If
                            End If
                        End If
                    End If
                Next
            End If
        End If
    End With
    
    'if nothing found, restore the user's selection
    If Not found Then
        Selection.start = originalStart
        Selection.end = originalEnd
        Selection.Select
    End If
    
    ' return the result
    BrowseOther = found
    
End Function


'If MTEquationSection style exists, make it not hidden
Private Sub ShowSectionStyle()
    If MTLib.styleExists(ActiveDocument, mtstyle_EQUATION_SECTION) Then
        ActiveDocument.Styles(mtstyle_EQUATION_SECTION).font.Hidden = False
    End If
End Sub
